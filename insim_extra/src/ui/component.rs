//! Components that keep their own per-player state, and the [`Screen`] that
//! draws one connection's buttons.
//!
//! A view is rendered afresh each time. Components placed with [`Ui::component`]
//! or [`Ui::child`] aren't stored by their parent: the screen keeps one instance
//! per player, identified by its parent, call site and [`Key`] (if any). It lives
//! only while it keeps being placed. Clicks on its buttons go straight to it, and
//! [`Child::on`] turns what it reports ([`Component::Out`]) into a message for
//! its parent.
//!
//! A component can also start work of its own, returned as [`Effects`] from
//! [`Component::mounted`] and [`Component::update`]: timers, tasks and streams
//! whose messages come back to it. They run on Tokio, report through the
//! screen's [`Outbox`], and are aborted when the instance unmounts.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
    panic::Location,
    sync::Arc,
    time::{Duration, Instant},
};

use insim::{
    Packet,
    identifiers::ConnectionId,
    insim::{BfnType, Btc, Btt},
};
use tokio::sync::mpsc;

use super::{Action, Canvas, CanvasDiff, Effects, Node, id_pool::ClickIdPool};
use crate::timers::{self, Job, JobId, Timers};

/// A piece of UI with its own state, kept per player.
///
/// ```ignore
/// struct Dialog { open: bool, lines: Vec<String> }
///
/// impl Component for Dialog {
///     type Msg = DialogMsg;     // its buttons' messages
///     type Out = ();            // nothing to report to its parent
///     type Props<'a> = ();      // no data from its parent
///     fn update(&mut self, msg: DialogMsg) -> Effects<DialogMsg> {
///         self.open = matches!(msg, DialogMsg::Show);
///         Effects::none()
///     }
///     fn render(&self, _: (), _: &mut Ui<'_, DialogMsg>) -> Node<DialogMsg> { /* ... */ }
/// }
/// ```
pub trait Component: Send + 'static {
    /// What its buttons produce, delivered to [`update`](Self::update).
    type Msg: Clone + Send + Sync + 'static;
    /// What it reports to its parent (see [`Child::on`]), or `()`.
    type Out: Send + 'static;
    /// Data from the parent on every render, possibly borrowed (see
    /// [`Child::with_props`]), or `()`.
    type Props<'a>;

    /// Runs once, before the first render, with the first props. `init` never
    /// sees props, so set up from them here, and start any loading: a task
    /// returned here delivers its result to [`update`](Self::update). Placed
    /// again after being dropped, it mounts afresh.
    fn mounted(&mut self, props: &Self::Props<'_>) -> Effects<Self::Msg, Self::Out> {
        let _ = props;
        Effects::none()
    }

    /// Runs once when it is dropped: not placed in a render, replaced by a
    /// different component, or its screen cleared (the player left, or the
    /// service stopped). Its timers and tasks are aborted.
    fn unmounted(&mut self) {}

    /// Handle one of its messages: a click on its buttons, or the result of a
    /// timer or task it started. Returns more work, and anything to report to
    /// the parent with [`Effects::emit`].
    fn update(&mut self, msg: Self::Msg) -> Effects<Self::Msg, Self::Out>;

    /// Draw it, placing any nested components with `ui`.
    fn render(&self, props: Self::Props<'_>, ui: &mut Ui<'_, Self::Msg>) -> Node<Self::Msg>;
}

/// Tells apart components placed by one call site (in a loop, say), and
/// addresses them from outside. Implemented for any `Hash + PartialEq + Debug +
/// Send + Sync + 'static` type, such as `&'static str` or `PlayerId`.
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

/// A timer, task or stream result from a component, on its way back to the
/// [`Screen`] that holds it. Hand it to [`Screen::deliver`].
pub struct Delivery {
    ucid: ConnectionId,
    path: u64,
    job: JobId,
    msg: AnyMsg,
}

impl fmt::Debug for Delivery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Delivery")
            .field("ucid", &self.ucid)
            .field("path", &self.path)
            .field("job", &self.job)
            .finish_non_exhaustive()
    }
}

impl Delivery {
    /// The connection whose screen it is for.
    pub fn ucid(&self) -> ConnectionId {
        self.ucid
    }
}

/// Where components' timers and tasks send their results, for whoever drives
/// the screens to pass back with [`Screen::deliver`]. kitcar's `Host` gives
/// every screen one. Driving screens yourself, use [`Outbox::channel`].
#[derive(Clone)]
pub struct Outbox(Arc<dyn Fn(Delivery) -> bool + Send + Sync>);

impl fmt::Debug for Outbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Outbox")
    }
}

impl Outbox {
    /// Send each delivery through `send`, which returns false once nobody is
    /// listening.
    pub fn new(send: impl Fn(Delivery) -> bool + Send + Sync + 'static) -> Self {
        Self(Arc::new(send))
    }
    /// An outbox and the receiver its deliveries arrive on.
    pub fn channel() -> (Self, mpsc::UnboundedReceiver<Delivery>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self::new(move |delivery| tx.send(delivery).is_ok()), rx)
    }
}

/// A component's effects with its types erased.
struct Work {
    jobs: Vec<Job<AnyMsg>>,
    emits: Vec<AnyMsg>,
}

impl Work {
    fn erase<C: Component>(effects: Effects<C::Msg, C::Out>) -> Self {
        let boxed: Arc<dyn Fn(C::Msg) -> AnyMsg + Send + Sync> =
            Arc::new(|msg| Box::new(msg) as AnyMsg);
        Self {
            jobs: effects
                .jobs
                .into_iter()
                .map(|job| job.map(boxed.clone()))
                .collect(),
            emits: effects
                .emits
                .into_iter()
                .map(|out| Box::new(out) as AnyMsg)
                .collect(),
        }
    }
}

/// How to place a component: its constructor, props, key and parent mapping.
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
    /// A component given `props` on every render.
    pub fn with_props(props: C::Props<'p>, init: impl FnOnce() -> C + 'p) -> Self {
        Self {
            props,
            init: Box::new(init),
            key: None,
            on: None,
            _parent: PhantomData,
        }
    }
    /// Tell this instance apart from others placed by the same line, and
    /// address it from outside.
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
    /// `None` if `msg` isn't one of its messages.
    fn update_any(&mut self, msg: &dyn Any) -> Option<Work>;
    fn unmounted(&mut self);
}
impl<C: Component> Erased for C {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn unmounted(&mut self) {
        Component::unmounted(self);
    }
    fn update_any(&mut self, msg: &dyn Any) -> Option<Work> {
        let msg = msg.downcast_ref::<C::Msg>()?.clone();
        Some(Work::erase::<C>(self.update(msg)))
    }
}

struct Instance {
    component: Box<dyn Erased>,
    parent: u64,
    key: Option<Box<dyn Key>>,
    on: Option<OnOut>,
    seen: u64,
    /// Started on its first timer or task.
    timers: Option<Timers<AnyMsg>>,
}

/// Every unmount ends here: unplaced, replaced, or its screen cleared or
/// dropped. Its timers and tasks are aborted with it.
impl Drop for Instance {
    fn drop(&mut self) {
        self.component.unmounted();
    }
}

/// The view itself, rather than a component.
const ROOT: u64 = 0;

struct Store {
    ucid: ConnectionId,
    outbox: Option<Outbox>,
    instances: HashMap<u64, Instance>,
    generation: u64,
    /// The earliest redraw asked for during the last render.
    redraw_at: Option<Instant>,
    /// What components returned from `mounted` during this render, carried
    /// out once it's done and every instance is in place.
    mounting: Vec<(u64, Work)>,
}

impl Store {
    fn new(ucid: ConnectionId, outbox: Option<Outbox>) -> Self {
        Self {
            ucid,
            outbox,
            instances: HashMap::new(),
            generation: 0,
            redraw_at: None,
            mounting: Vec::new(),
        }
    }

    /// Deliver `msg` to an instance and carry out its effects. Returns the
    /// messages that reach the view.
    fn deliver(&mut self, path: u64, msg: &dyn Any) -> Vec<AnyMsg> {
        let Some(work) = self
            .instances
            .get_mut(&path)
            .and_then(|instance| instance.component.update_any(msg))
        else {
            return Vec::new();
        };
        self.apply(path, work)
    }

    /// Start an instance's work, and pass what it reports up to its parent,
    /// whose own update may report further. Returns the messages that reach
    /// the view.
    fn apply(&mut self, path: u64, work: Work) -> Vec<AnyMsg> {
        let Some(instance) = self.instances.get_mut(&path) else {
            return Vec::new();
        };
        if !work.jobs.is_empty() {
            match (&mut instance.timers, &self.outbox) {
                (Some(timers), _) => Self::run(timers, work.jobs),
                (None, Some(outbox)) => {
                    let (outbox, ucid) = (outbox.clone(), self.ucid);
                    let timers = instance
                        .timers
                        .insert(Timers::new(Arc::new(move |job, msg| {
                            (outbox.0)(Delivery {
                                ucid,
                                path,
                                job,
                                msg,
                            })
                        })));
                    Self::run(timers, work.jobs);
                },
                (None, None) => tracing::warn!(
                    ucid = %self.ucid,
                    "a component started a timer or task on a screen with no outbox; dropped"
                ),
            }
        }
        let (parent, on) = (instance.parent, instance.on.clone());
        let mut root = Vec::new();
        for out in work.emits {
            let Some(mapped) = on.as_ref().and_then(|on| on(out)) else {
                continue;
            };
            if parent == ROOT {
                root.push(mapped);
            } else {
                root.extend(self.deliver(parent, &*mapped));
            }
        }
        root
    }

    fn run(timers: &mut Timers<AnyMsg>, jobs: Vec<Job<AnyMsg>>) {
        let now = timers::now();
        for job in jobs {
            timers.run(job, now);
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

    /// Draw again at `at`, even if nothing changes. Views are otherwise only
    /// redrawn after an update, so anything drawn from the clock (a countdown,
    /// an animation) asks for its next frame here. The earliest request in a
    /// render wins; each render starts with none.
    pub fn redraw_at(&mut self, at: Instant) {
        let at = at.max(self.now);
        self.store.redraw_at = Some(self.store.redraw_at.map_or(at, |earlier| earlier.min(at)));
    }

    /// Like [`redraw_at`](Self::redraw_at), `after` from now.
    pub fn redraw_after(&mut self, after: Duration) {
        self.redraw_at(self.now + after);
    }

    /// Whole seconds left until `deadline`, rounded up, and a redraw for when
    /// the number changes. Zero once it has passed.
    ///
    /// ```ignore
    /// if let Some(at) = cx.director().and_then(|d| d.restart_at) {
    ///     let secs = ui.countdown(at);
    ///     ui::text(format!("Next race in {secs}s"), style)
    /// }
    /// ```
    pub fn countdown(&mut self, deadline: Instant) -> u64 {
        let left = deadline.saturating_duration_since(self.now);
        if left.is_zero() {
            return 0;
        }
        let secs = left.as_secs() + u64::from(left.subsec_nanos() > 0);
        // The number drops when `secs - 1` whole seconds are left.
        self.redraw_at(deadline - Duration::from_secs(secs - 1));
        secs
    }

    /// Place a component with no props. It is created the first time this line
    /// places it for a player, and kept while it keeps being placed. For props,
    /// its reports, or a [`Child::key`] (needed to place several from one line),
    /// use [`child`](Self::child).
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
                let mut component = (child.init)();
                let work = Work::erase::<C>(component.mounted(&child.props));
                if !work.jobs.is_empty() || !work.emits.is_empty() {
                    self.store.mounting.push((id, work));
                }
                Instance {
                    component: Box::new(component),
                    parent: self.owner,
                    key: None,
                    on: None,
                    seen: generation,
                    timers: None,
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

/// One connection's buttons for one view: what is drawn, the components behind
/// it, and whether the player has hidden them.
///
/// Click ids come from a pool shared by every screen on the connection, so
/// several views can draw on it without colliding.
pub struct Screen<M: Clone + 'static> {
    ucid: ConnectionId,
    canvas: Canvas<M>,
    store: Store,
    hidden: bool,
    dirty: bool,
    /// Messages for the view from components reporting as they mounted.
    raised: Vec<M>,
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
    /// A screen for `ucid` with nothing drawn yet, and no outbox: components
    /// on it can't start timers or tasks. See [`with_outbox`](Self::with_outbox).
    pub fn new(ucid: ConnectionId) -> Self {
        Self::build(ucid, None)
    }

    /// A screen whose components' timers and tasks report to `outbox`. Pass
    /// each [`Delivery`] back with [`deliver`](Self::deliver).
    pub fn with_outbox(ucid: ConnectionId, outbox: Outbox) -> Self {
        Self::build(ucid, Some(outbox))
    }

    fn build(ucid: ConnectionId, outbox: Option<Outbox>) -> Self {
        Self {
            ucid,
            canvas: Canvas::new(ucid),
            store: Store::new(ucid, outbox),
            hidden: false,
            dirty: true,
            raised: Vec::new(),
        }
    }

    /// Whether a component changed, or the player asked for the buttons back,
    /// since the last render.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// When the last render asked to be drawn again, if it did. See
    /// [`Ui::redraw_at`].
    pub fn redraw_at(&self) -> Option<Instant> {
        if self.hidden {
            None
        } else {
            self.store.redraw_at
        }
    }

    /// Draw `view` and return the packets that bring the player's buttons up to
    /// date. Unplaced components are dropped. Draws nothing while the player has
    /// hidden the buttons.
    pub fn render(
        &mut self,
        pool: &mut ClickIdPool,
        now: Instant,
        view: impl FnOnce(&mut Ui<'_, M>) -> Option<Node<M>>,
    ) -> Vec<Packet> {
        self.dirty = false;
        self.store.redraw_at = None;
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
        for (path, work) in std::mem::take(&mut self.store.mounting) {
            // Reporting may change a parent's state, so draw again.
            self.dirty |= !work.emits.is_empty();
            let raised = Self::root(self.store.apply(path, work));
            self.raised.extend(raised);
        }
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

    /// Hand a component the result of one of its timers or tasks, and return
    /// any messages for the view. Dropped if the component has since
    /// unmounted, or replaced or cancelled the work that sent it.
    pub fn deliver(&mut self, delivery: Delivery) -> Vec<M> {
        let accepted = delivery.ucid == self.ucid
            && self
                .store
                .instances
                .get_mut(&delivery.path)
                .and_then(|instance| instance.timers.as_mut())
                .is_some_and(|timers| timers.accept(delivery.job));
        if !accepted {
            return Vec::new();
        }
        self.dirty = true;
        Self::root(self.store.deliver(delivery.path, &*delivery.msg))
    }

    /// Messages for the view from components that reported as they mounted
    /// during the last render.
    pub fn take_messages(&mut self) -> Vec<M> {
        std::mem::take(&mut self.raised)
    }

    /// Send `msg` to every component placed with `key`, as if from one of its
    /// buttons, and return any messages for the view. If none is placed, `msg` is
    /// dropped with a warning.
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

    fn root(msgs: Vec<AnyMsg>) -> Vec<M> {
        msgs.into_iter()
            .filter_map(|msg| msg.downcast::<M>().ok().map(|msg| *msg))
            .collect()
    }

    /// Remove every button and component, releasing their click ids. Returns the
    /// packets that delete the buttons.
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
        fn update(&mut self, msg: ConfirmMsg) -> Effects<ConfirmMsg, bool> {
            match msg {
                ConfirmMsg::Toggle => {
                    self.open = !self.open;
                    Effects::none()
                },
                ConfirmMsg::Yes => Effects::new().emit(true),
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
        fn update(&mut self, PanelMsg::Confirmed: PanelMsg) -> Effects<PanelMsg, &'static str> {
            Effects::new().emit("cleared")
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

    /// Remembers the props it was mounted with, and counts its mounts and
    /// unmounts.
    struct Snapshot {
        taken: u32,
        mounts: std::sync::Arc<std::sync::atomic::AtomicU32>,
        unmounts: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }
    impl Component for Snapshot {
        type Msg = ();
        type Out = ();
        type Props<'a> = u32;
        fn mounted(&mut self, props: &u32) -> Effects<()> {
            self.taken = *props;
            let _ = self
                .mounts
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Effects::none()
        }
        fn unmounted(&mut self) {
            let _ = self
                .unmounts
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        fn update(&mut self, (): ()) -> Effects<()> {
            Effects::none()
        }
        fn render(&self, live: u32, _: &mut Ui<'_, ()>) -> Node<()> {
            Node::text(format!("{} {live}", self.taken), BtnStyle::default())
                .w(20)
                .h(5)
        }
    }

    #[test]
    fn components_mount_and_unmount_once_per_placement() {
        use std::sync::{
            Arc,
            atomic::{AtomicU32, Ordering},
        };

        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<()>::new(ConnectionId(3));
        let now = Instant::now();
        let mounts = Arc::new(AtomicU32::new(0));
        let unmounts = Arc::new(AtomicU32::new(0));
        let view = |live: u32, shown: bool| {
            let (mounts, unmounts) = (mounts.clone(), unmounts.clone());
            move |ui: &mut Ui<'_, ()>| {
                shown.then(|| {
                    ui.child(Child::with_props(live, move || Snapshot {
                        taken: 0,
                        mounts,
                        unmounts,
                    }))
                })
            }
        };

        let first = texts(&screen.render(&mut pool, now, view(1, true)));
        assert_eq!(first[0].1, "1 1");
        // Later renders see new props, but the mount-time snapshot stays.
        let second = texts(&screen.render(&mut pool, now, view(2, true)));
        assert_eq!(second[0].1, "1 2");
        assert_eq!(mounts.load(Ordering::Relaxed), 1);
        assert_eq!(unmounts.load(Ordering::Relaxed), 0);

        // Not placed: it unmounts. Placed again: it mounts afresh.
        let _ = screen.render(&mut pool, now, view(3, false));
        assert_eq!(unmounts.load(Ordering::Relaxed), 1);
        let again = texts(&screen.render(&mut pool, now, view(4, true)));
        assert_eq!(again[0].1, "4 4");
        assert_eq!(mounts.load(Ordering::Relaxed), 2);

        // Clearing the screen (the player left, or the service stopped) unmounts too.
        let _ = screen.clear(&mut pool);
        assert_eq!(unmounts.load(Ordering::Relaxed), 2);
    }

    /// Loads on mount, then closes itself a while after.
    struct Board {
        rows: Option<u32>,
    }
    #[derive(Clone, Debug, PartialEq)]
    enum BoardMsg {
        Loaded(u32),
        Close,
    }
    #[derive(Debug, PartialEq)]
    struct CloseLater;
    impl Component for Board {
        type Msg = BoardMsg;
        type Out = &'static str;
        type Props<'a> = ();
        fn mounted(&mut self, (): &()) -> Effects<BoardMsg, &'static str> {
            Effects::new().task(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                BoardMsg::Loaded(7)
            })
        }
        fn update(&mut self, msg: BoardMsg) -> Effects<BoardMsg, &'static str> {
            match msg {
                BoardMsg::Loaded(rows) => {
                    self.rows = Some(rows);
                    Effects::new().schedule_after(
                        CloseLater,
                        Duration::from_secs(5),
                        BoardMsg::Close,
                    )
                },
                BoardMsg::Close => Effects::new().emit("closed"),
            }
        }
        fn render(&self, (): (), _: &mut Ui<'_, BoardMsg>) -> Node<BoardMsg> {
            let text = match self.rows {
                Some(rows) => format!("{rows} rows"),
                None => "loading".to_string(),
            };
            Node::text(text, BtnStyle::default()).w(20).h(5)
        }
    }

    fn board(shown: bool) -> impl FnOnce(&mut Ui<'_, Root>) -> Option<Node<Root>> {
        move |ui| shown.then(|| ui.child(Child::new(|| Board { rows: None }).on(Root::Done)))
    }

    /// The next delivery, waiting as long as it takes on the paused clock.
    async fn delivery(rx: &mut mpsc::UnboundedReceiver<Delivery>) -> Delivery {
        rx.recv().await.expect("the screen holds the outbox")
    }

    #[tokio::test(start_paused = true)]
    async fn components_load_on_mount_and_act_on_their_own_timers() {
        let (outbox, mut rx) = Outbox::channel();
        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<Root>::with_outbox(ConnectionId(3), outbox);

        let drawn = texts(&screen.render(&mut pool, timers::now(), board(true)));
        assert_eq!(drawn[0].1, "loading");

        // The task's result comes back to the component, which redraws.
        let loaded = delivery(&mut rx).await;
        assert!(screen.deliver(loaded).is_empty());
        assert!(screen.is_dirty());
        let redrawn = texts(&screen.render(&mut pool, timers::now(), board(true)));
        assert_eq!(redrawn[0].1, "7 rows");

        // Its timer fires five seconds later, and what it reports reaches the
        // view.
        let started = tokio::time::Instant::now();
        let close = delivery(&mut rx).await;
        assert_eq!(started.elapsed(), Duration::from_secs(5));
        assert_eq!(screen.deliver(close), [Root::Done("closed")]);
    }

    #[tokio::test(start_paused = true)]
    async fn unmounting_aborts_a_components_work() {
        let (outbox, mut rx) = Outbox::channel();
        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<Root>::with_outbox(ConnectionId(3), outbox);

        let _ = screen.render(&mut pool, timers::now(), board(true));
        // Unplaced before its load finishes: the load is aborted.
        let _ = screen.render(&mut pool, timers::now(), board(false));
        let waited = tokio::time::timeout(Duration::from_secs(60), rx.recv()).await;
        assert!(waited.is_err(), "an unmounted component's task reported");

        // Placed again, it mounts afresh and loads again.
        let _ = screen.render(&mut pool, timers::now(), board(true));
        let loaded = delivery(&mut rx).await;
        assert!(screen.deliver(loaded).is_empty());
        let redrawn = texts(&screen.render(&mut pool, timers::now(), board(true)));
        assert_eq!(redrawn[0].1, "7 rows");

        // A result for an instance that has since gone is dropped.
        let close = delivery(&mut rx).await;
        let _ = screen.render(&mut pool, timers::now(), board(false));
        assert!(screen.deliver(close).is_empty());
    }

    /// Reports as soon as it mounts.
    struct Greeter;
    impl Component for Greeter {
        type Msg = ();
        type Out = &'static str;
        type Props<'a> = ();
        fn mounted(&mut self, (): &()) -> Effects<(), &'static str> {
            Effects::new().emit("hello")
        }
        fn update(&mut self, (): ()) -> Effects<(), &'static str> {
            Effects::none()
        }
        fn render(&self, (): (), _: &mut Ui<'_, ()>) -> Node<()> {
            Node::empty()
        }
    }

    #[test]
    fn reports_from_mounting_reach_the_view_after_the_render() {
        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<Root>::new(ConnectionId(3));
        let view = |ui: &mut Ui<'_, Root>| Some(ui.child(Child::new(|| Greeter).on(Root::Done)));
        let _ = screen.render(&mut pool, Instant::now(), view);
        assert_eq!(screen.take_messages(), [Root::Done("hello")]);
        assert!(screen.is_dirty());
        // Only once: it's already mounted.
        let _ = screen.render(&mut pool, Instant::now(), view);
        assert!(screen.take_messages().is_empty());
    }

    #[test]
    fn countdowns_ask_to_be_redrawn_when_the_number_changes() {
        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<()>::new(ConnectionId(1));
        let now = Instant::now();
        let deadline = now + Duration::from_millis(2_500);

        let mut secs = None;
        let _ = screen.render(&mut pool, now, |ui| {
            secs = Some(ui.countdown(deadline));
            None
        });
        assert_eq!(secs, Some(3));
        assert_eq!(screen.redraw_at(), Some(now + Duration::from_millis(500)));

        // Each render starts with no request; a passed deadline makes none.
        let _ = screen.render(&mut pool, deadline, |ui| {
            secs = Some(ui.countdown(deadline));
            None
        });
        assert_eq!(secs, Some(0));
        assert_eq!(screen.redraw_at(), None);

        // The earliest request wins.
        let _ = screen.render(&mut pool, now, |ui| {
            ui.redraw_after(Duration::from_secs(5));
            ui.redraw_after(Duration::from_secs(1));
            None
        });
        assert_eq!(screen.redraw_at(), Some(now + Duration::from_secs(1)));
    }
}
