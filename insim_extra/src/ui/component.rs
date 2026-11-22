//! Components that keep their own per-player state, and the [`Screen`] that
//! draws one connection's buttons.
//!
//! A view is rendered from scratch each time. Components placed with
//! [`Ui::component`] or [`Ui::child`] are not stored by their parent; the screen
//! keeps one instance per player, identified by where it is placed (the call
//! site, plus a [`Key`] if given). Clicks on a component's buttons go straight to
//! its instance, and what it reports back ([`Component::Out`]) can be turned into
//! its parent's message with [`Child::on`].

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
    panic::Location,
    sync::Arc,
    time::Instant,
};

use insim::{
    Packet,
    identifiers::ConnectionId,
    insim::{BfnType, Btc, Btt},
};

use super::{Action, Canvas, CanvasDiff, Node, id_pool::ClickIdPool};

/// A piece of UI with its own state, kept per player.
///
/// ```ignore
/// struct Dialog { open: bool, lines: Vec<String> }
///
/// impl Component for Dialog {
///     type Msg = DialogMsg;     // its buttons' messages
///     type Out = ();            // nothing to report to its parent
///     type Props<'a> = ();      // no data from its parent
///     fn update(&mut self, msg: DialogMsg) -> Option<()> {
///         self.open = matches!(msg, DialogMsg::Show);
///         None
///     }
///     fn render(&self, _: (), _: &mut Ui<'_, DialogMsg>) -> Node<DialogMsg> { /* ... */ }
/// }
/// ```
pub trait Component: Send + 'static {
    /// What this component's buttons produce. Delivered to [`update`](Self::update).
    type Msg: Clone + Send + Sync + 'static;
    /// What it reports to its parent, if anything; see [`Child::on`]. Use `()`
    /// when it reports nothing.
    type Out: Send + 'static;
    /// Data passed in by the parent on every render, which may borrow from it;
    /// see [`Child::with_props`]. Use `()` when there is none.
    type Props<'a>;

    /// Handle one of its own messages, optionally reporting to the parent.
    fn update(&mut self, msg: Self::Msg) -> Option<Self::Out>;

    /// Draw it. `ui` places nested components.
    fn render(&self, props: Self::Props<'_>, ui: &mut Ui<'_, Self::Msg>) -> Node<Self::Msg>;
}

/// Names a component so it can be told apart from others placed at the same
/// call site (such as in a loop), and addressed from outside. Implemented for
/// any `Hash + PartialEq + Debug + Send + Sync + 'static` type, including
/// `&'static str` and ids such as `PlayerId`.
pub trait Key: Any + Send + Sync + fmt::Debug {
    #[doc(hidden)]
    fn same(&self, other: &dyn Key) -> bool;
    #[doc(hidden)]
    fn id(&self) -> u64;
    #[doc(hidden)]
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any + Send + Sync + fmt::Debug + Hash + PartialEq> Key for T {
    fn same(&self, other: &dyn Key) -> bool {
        other.as_any().downcast_ref::<T>() == Some(self)
    }
    fn id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        self.hash(&mut hasher);
        hasher.finish()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

type AnyMsg = Box<dyn Any + Send>;
type OnOut = Arc<dyn Fn(AnyMsg) -> Option<AnyMsg> + Send + Sync>;

/// How to place a component: what creates it, its props, key and parent mapping.
#[must_use = "place it with Ui::child"]
pub struct Child<'p, C: Component, M> {
    props: C::Props<'p>,
    init: Box<dyn FnOnce() -> C + 'p>,
    key: Option<Box<dyn Key>>,
    on: Option<OnOut>,
    _parent: PhantomData<fn() -> M>,
}

impl<C: Component, M> fmt::Debug for Child<'_, C, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Child")
            .field("key", &self.key)
            .field("on", &self.on.is_some())
            .finish_non_exhaustive()
    }
}

impl<'p, C, M> Child<'p, C, M>
where
    C: for<'a> Component<Props<'a> = ()>,
{
    /// A component without props. `init` runs only when the instance is created.
    pub fn new(init: impl FnOnce() -> C + 'p) -> Self {
        Self::with_props((), init)
    }
}

impl<'p, C: Component, M> Child<'p, C, M> {
    /// A component that is given `props` on every render.
    pub fn with_props(props: C::Props<'p>, init: impl FnOnce() -> C + 'p) -> Self {
        Self {
            props,
            init: Box::new(init),
            key: None,
            on: None,
            _parent: PhantomData,
        }
    }
    /// Tell this instance apart from others placed by the same line, and let
    /// it be addressed from outside.
    pub fn key(mut self, key: impl Key) -> Self {
        self.key = Some(Box::new(key));
        self
    }
    /// Turn what the component reports into a message for the parent.
    pub fn on(mut self, f: impl Fn(C::Out) -> M + Send + Sync + 'static) -> Self
    where
        M: Send + 'static,
    {
        self.on = Some(Arc::new(move |out: AnyMsg| {
            out.downcast::<C::Out>()
                .ok()
                .map(|out| Box::new(f(*out)) as AnyMsg)
        }));
        self
    }
}

trait Erased: Send {
    fn as_any(&self) -> &dyn Any;
    fn update_any(&mut self, msg: &dyn Any) -> Option<AnyMsg>;
}
impl<C: Component> Erased for C {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn update_any(&mut self, msg: &dyn Any) -> Option<AnyMsg> {
        let msg = msg.downcast_ref::<C::Msg>()?.clone();
        self.update(msg).map(|out| Box::new(out) as AnyMsg)
    }
}

struct Instance {
    component: Box<dyn Erased>,
    parent: u64,
    key: Option<Box<dyn Key>>,
    on: Option<OnOut>,
    seen: u64,
}

/// The view's own messages come from the root.
const ROOT: u64 = 0;

#[derive(Default)]
struct Store {
    instances: HashMap<u64, Instance>,
    generation: u64,
}

impl Store {
    /// Deliver a message to an instance and pass what it reports up through its
    /// parents. Returns the root's message, if it gets that far.
    fn deliver(&mut self, mut path: u64, msg: &dyn Any) -> Option<AnyMsg> {
        let mut out = self.instances.get_mut(&path)?.component.update_any(msg)?;
        loop {
            let instance = self.instances.get(&path)?;
            let mapped = (instance.on.as_ref()?)(out)?;
            if instance.parent == ROOT {
                return Some(mapped);
            }
            path = instance.parent;
            out = self
                .instances
                .get_mut(&path)?
                .component
                .update_any(&*mapped)?;
        }
    }
}

/// Where a view places its buttons and components. Passed to a view, and to
/// each component's [`render`](Component::render) for nested components.
pub struct Ui<'a, M> {
    store: &'a mut Store,
    owner: u64,
    now: Instant,
    _msg: PhantomData<fn() -> M>,
}

impl<M> fmt::Debug for Ui<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ui")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

impl<M> Ui<'_, M> {
    /// The time of this render, for countdowns and animation.
    pub fn now(&self) -> Instant {
        self.now
    }

    /// Place a self-contained component with no props. Its instance is created
    /// the first time this line places it for a player, then kept across renders
    /// until it is no longer placed. A loop that places several from one line
    /// needs [`Child::key`]; use [`child`](Self::child) for that, for props, or
    /// to receive what it reports.
    #[track_caller]
    pub fn component<C>(&mut self, init: impl FnOnce() -> C) -> Node<M>
    where
        C: for<'a> Component<Props<'a> = ()>,
    {
        self.place(Child::new(init), Location::caller())
    }

    /// Place a component described by a [`Child`].
    #[track_caller]
    pub fn child<C: Component>(&mut self, child: Child<'_, C, M>) -> Node<M> {
        self.place(child, Location::caller())
    }

    fn place<C: Component>(&mut self, child: Child<'_, C, M>, at: &Location<'_>) -> Node<M> {
        let mut hasher = DefaultHasher::new();
        self.owner.hash(&mut hasher);
        (at.file(), at.line(), at.column()).hash(&mut hasher);
        child.key.as_ref().map(|key| key.id()).hash(&mut hasher);
        // ROOT is reserved for the view itself.
        let id = hasher.finish().max(1);

        let generation = self.store.generation;
        let mut instance = match self.store.instances.remove(&id) {
            Some(existing) if existing.component.as_any().is::<C>() => {
                if existing.seen == generation {
                    tracing::warn!(
                        file = at.file(),
                        line = at.line(),
                        "two components placed by one line share an instance; give each a key"
                    );
                }
                existing
            },
            replaced => {
                if replaced.is_some() {
                    tracing::warn!(
                        file = at.file(),
                        line = at.line(),
                        "a different component was placed where another was; it starts afresh"
                    );
                }
                Instance {
                    component: Box::new((child.init)()),
                    parent: self.owner,
                    key: None,
                    on: None,
                    seen: generation,
                }
            },
        };
        instance.seen = generation;
        instance.key = child.key;
        instance.on = child.on;

        let node = {
            let component = instance
                .component
                .as_any()
                .downcast_ref::<C>()
                .expect("the instance was checked or created as C");
            let mut ui = Ui::<C::Msg> {
                store: &mut *self.store,
                owner: id,
                now: self.now,
                _msg: PhantomData,
            };
            component.render(child.props, &mut ui)
        };
        let _ = self.store.instances.insert(id, instance);
        node.route(id)
    }
}

/// One connection's buttons for one view: what is drawn, the component instances
/// behind it, and whether the player has hidden the buttons.
///
/// Click ids are leased from a pool shared by every screen on the connection, so
/// several views can draw on one connection without colliding.
pub struct Screen<M: Clone + 'static> {
    ucid: ConnectionId,
    canvas: Canvas<M>,
    store: Store,
    hidden: bool,
    dirty: bool,
}

impl<M: Clone + 'static> fmt::Debug for Screen<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Screen")
            .field("ucid", &self.ucid)
            .field("components", &self.store.instances.len())
            .field("hidden", &self.hidden)
            .finish_non_exhaustive()
    }
}

impl<M: Clone + Send + 'static> Screen<M> {
    /// A screen for `ucid` with nothing drawn yet.
    pub fn new(ucid: ConnectionId) -> Self {
        Self {
            ucid,
            canvas: Canvas::new(ucid),
            store: Store::default(),
            hidden: false,
            dirty: true,
        }
    }

    /// Whether a component changed, or the player asked for the buttons back,
    /// since the last render.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Draw `view` and return the packets that bring the player's buttons up to
    /// date. Components no longer placed are dropped. Draws nothing while the
    /// player has hidden the buttons.
    pub fn render(
        &mut self,
        pool: &mut ClickIdPool,
        now: Instant,
        view: impl FnOnce(&mut Ui<'_, M>) -> Option<Node<M>>,
    ) -> Vec<Packet> {
        self.dirty = false;
        if self.hidden {
            return Vec::new();
        }
        self.store.generation += 1;
        let node = view(&mut Ui {
            store: &mut self.store,
            owner: ROOT,
            now,
            _msg: PhantomData,
        });
        let generation = self.store.generation;
        self.store
            .instances
            .retain(|_, instance| instance.seen == generation);
        self.canvas
            .reconcile_in(node.unwrap_or_else(Node::empty), pool)
            .map(CanvasDiff::merge)
            .unwrap_or_default()
    }

    /// Handle a button packet. Clicks and type-ins on a component go to its
    /// instance; the view's own messages are returned. `Bfn` hides or restores
    /// the buttons when the player clears or requests them.
    pub fn handle(&mut self, packet: &Packet, pool: &mut ClickIdPool) -> Vec<M> {
        let action = match packet {
            Packet::Btc(Btc { ucid, clickid, .. }) if *ucid == self.ucid => {
                self.canvas.translate_clickid(clickid)
            },
            Packet::Btt(Btt {
                ucid,
                clickid,
                text,
                ..
            }) if *ucid == self.ucid => self.canvas.translate_typein_clickid(clickid, text.clone()),
            Packet::Bfn(bfn) if bfn.ucid == self.ucid => {
                match bfn.subt {
                    BfnType::UserClear | BfnType::Clear => {
                        self.hidden = true;
                        self.canvas.clear_in(pool);
                    },
                    BfnType::BtnRequest => {
                        self.hidden = false;
                        self.dirty = true;
                    },
                    _ => {},
                }
                None
            },
            _ => None,
        };
        match action {
            None => Vec::new(),
            Some(Action::Own(msg)) => vec![msg],
            Some(Action::Routed(routed)) => {
                self.dirty = true;
                Self::root(self.store.deliver(routed.path, &*routed.msg))
            },
        }
    }

    /// Send `msg` to every component placed with `key`, as if it came from one of
    /// its buttons. Returns the view's own messages that result.
    pub fn send(&mut self, key: &dyn Key, msg: &dyn Any) -> Vec<M> {
        let targets: Vec<u64> = self
            .store
            .instances
            .iter()
            .filter(|(_, instance)| instance.key.as_deref().is_some_and(|k| k.same(key)))
            .map(|(id, _)| *id)
            .collect();
        if targets.is_empty() {
            tracing::warn!(?key, ucid = %self.ucid, "no component has this key");
        }
        let mut msgs = Vec::new();
        for id in targets {
            self.dirty = true;
            msgs.extend(Self::root(self.store.deliver(id, msg)));
        }
        msgs
    }

    fn root(msg: Option<AnyMsg>) -> Vec<M> {
        msg.and_then(|msg| msg.downcast::<M>().ok())
            .map(|msg| vec![*msg])
            .unwrap_or_default()
    }

    /// Remove every button and component, returning the packets that delete the
    /// buttons and releasing their click ids.
    pub fn clear(&mut self, pool: &mut ClickIdPool) -> Vec<Packet> {
        self.store.instances.clear();
        self.canvas
            .reconcile_in(Node::empty(), pool)
            .map(CanvasDiff::merge)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use insim::{
        identifiers::ClickId,
        insim::{Btn, BtnStyle},
    };

    use super::*;

    /// Opens and closes itself; reports when confirmed.
    struct Confirm {
        open: bool,
    }
    #[derive(Clone, Debug, PartialEq)]
    enum ConfirmMsg {
        Toggle,
        Yes,
    }
    impl Component for Confirm {
        type Msg = ConfirmMsg;
        type Out = bool;
        type Props<'a> = &'a str;
        fn update(&mut self, msg: ConfirmMsg) -> Option<bool> {
            match msg {
                ConfirmMsg::Toggle => {
                    self.open = !self.open;
                    None
                },
                ConfirmMsg::Yes => Some(true),
            }
        }
        fn render(&self, label: &str, _: &mut Ui<'_, ConfirmMsg>) -> Node<ConfirmMsg> {
            let state = if self.open { "open" } else { "closed" };
            Node::container().with_children([
                Node::clickable(
                    format!("{label} {state}"),
                    BtnStyle::default(),
                    ConfirmMsg::Toggle,
                )
                .w(20)
                .h(5),
                Node::clickable("yes", BtnStyle::default(), ConfirmMsg::Yes)
                    .w(20)
                    .h(5),
            ])
        }
    }

    /// Holds a nested Confirm, and passes its answer on.
    struct Panel;
    #[derive(Clone, Debug, PartialEq)]
    enum PanelMsg {
        Confirmed,
    }
    impl Component for Panel {
        type Msg = PanelMsg;
        type Out = &'static str;
        type Props<'a> = ();
        fn update(&mut self, PanelMsg::Confirmed: PanelMsg) -> Option<&'static str> {
            Some("cleared")
        }
        fn render(&self, (): (), ui: &mut Ui<'_, PanelMsg>) -> Node<PanelMsg> {
            ui.child(
                Child::with_props("nested", || Confirm { open: false })
                    .key("confirm")
                    .on(|_yes| PanelMsg::Confirmed),
            )
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Root {
        Done(&'static str),
    }

    fn texts(packets: &[Packet]) -> Vec<(ClickId, String)> {
        packets
            .iter()
            .filter_map(|p| match p {
                Packet::Btn(Btn { clickid, text, .. }) => Some((*clickid, text.clone())),
                _ => None,
            })
            .collect()
    }
    fn click(clickid: ClickId) -> Packet {
        Btc {
            ucid: ConnectionId(3),
            clickid,
            ..Default::default()
        }
        .into()
    }

    fn view(ui: &mut Ui<'_, Root>) -> Option<Node<Root>> {
        Some(Node::container().with_child(ui.child(Child::new(|| Panel).on(Root::Done))))
    }

    #[test]
    fn nested_components_keep_state_and_report_up() {
        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<Root>::new(ConnectionId(3));
        let now = Instant::now();

        let drawn = texts(&screen.render(&mut pool, now, view));
        assert_eq!(drawn[0].1, "nested closed");
        let (toggle, yes) = (drawn[0].0, drawn[1].0);

        // A click on the nested component changes only its own state.
        assert!(screen.handle(&click(toggle), &mut pool).is_empty());
        assert!(screen.is_dirty());
        let redrawn = texts(&screen.render(&mut pool, now, view));
        assert_eq!(redrawn, [(toggle, "nested open".to_string())]);

        // Its answer passes through the panel to the view.
        assert_eq!(
            screen.handle(&click(yes), &mut pool),
            [Root::Done("cleared")]
        );

        // It can be addressed by key from outside.
        assert!(screen.send(&"confirm", &ConfirmMsg::Toggle).is_empty());
        let closed = texts(&screen.render(&mut pool, now, view));
        assert_eq!(closed, [(toggle, "nested closed".to_string())]);
    }

    #[test]
    fn unplaced_components_are_dropped_and_screens_share_one_pool() {
        let mut pool = ClickIdPool::new();
        let mut a = Screen::<Root>::new(ConnectionId(3));
        let mut b = Screen::<Root>::new(ConnectionId(3));
        let now = Instant::now();
        let ids_a: Vec<_> = texts(&a.render(&mut pool, now, view))
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        let ids_b: Vec<_> = texts(&b.render(&mut pool, now, view))
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        assert!(
            ids_a.iter().all(|id| !ids_b.contains(id)),
            "click ids collide"
        );

        // Toggle, then stop placing it: the next placement starts afresh.
        let _ = a.handle(&click(ids_a[0]), &mut pool);
        let _ = a.render(&mut pool, now, |_| None);
        let again = texts(&a.render(&mut pool, now, view));
        assert_eq!(again[0].1, "nested closed");

        // Clearing one screen frees only its own ids.
        let removed = b.clear(&mut pool);
        assert_eq!(removed.len(), 2);
        assert_eq!(pool.available_count(), 239 - 2);
    }
}
