//! [`Service`]: one update function over its own state.

use std::{any::Any, collections::HashMap, time::Instant};

use insim::{Packet, identifiers::ConnectionId};

use crate::{
    kitcar::effects::Effects,
    ui::{Key, Node, Screen, Ui, id_pool::ClickIdPool},
    world::{World, WorldEvent},
};

/// A game, idle screen or long-lived service run by a
/// [`Host`](crate::kitcar::Host).
///
/// Unlike tower's `Service`, this isn't request/response. It sees every input
/// on the connection, one at a time, with `&mut self`, so state needs no
/// locks. `update` does no I/O: it returns [`Effects`] for the host to carry
/// out. Slow work goes in [`Effects::task`] and comes back as a message.
///
/// Reusable pieces are plain structs held as fields, combined with
/// [`Effects::and`] and [`Effects::map`].
///
/// [`view`](Self::view) draws buttons straight from state, so nothing needs
/// syncing. The host redraws after each update and sends only the changes.
/// Per-player UI state (an open dialog, a scroll position) lives in
/// [`Component`](crate::ui::Component)s placed with
/// [`Ui::component`](crate::ui::Ui::component), which the host keeps per
/// player.
pub trait Service: Send + 'static {
    /// Timer payloads, task results and button clicks.
    type Msg: Clone + Send + 'static;

    /// React to one input.
    fn update(&mut self, input: Input<Self::Msg>, cx: &Context<'_>) -> Effects<Self::Msg>;

    /// Draw one player's buttons. Clicks arrive as [`Input::Msg`]. Never
    /// called for the local connection (UCID 0).
    fn view(
        &self,
        ucid: ConnectionId,
        ui: &mut Ui<'_, Self::Msg>,
        cx: &Context<'_>,
    ) -> Option<Node<Self::Msg>> {
        let _ = (ucid, ui, cx);
        None
    }
}

/// One input to a [`Service`], in the order the host received it.
#[derive(Debug)]
#[non_exhaustive]
pub enum Input<M> {
    /// The first input. Arm timers, attach streams, set up the track.
    Started,
    /// A packet from LFS. The world already reflects it.
    Packet(Packet),
    /// A world change caused by the preceding packet, or
    /// [`WorldEvent::Ready`] once the world has caught up after connecting.
    World(WorldEvent),
    /// One of the service's own messages.
    Msg(M),
    /// The last input. Packets returned now are still sent; timers and tasks
    /// are dropped.
    Stopping,
}

impl<M> Input<M> {
    pub(crate) fn map<N>(self, f: impl FnOnce(M) -> N) -> Input<N> {
        match self {
            Self::Started => Input::Started,
            Self::Packet(packet) => Input::Packet(packet),
            Self::World(event) => Input::World(event),
            Self::Msg(msg) => Input::Msg(f(msg)),
            Self::Stopping => Input::Stopping,
        }
    }
    /// The packet, if any.
    pub fn packet(&self) -> Option<&Packet> {
        match self {
            Self::Packet(packet) => Some(packet),
            _ => None,
        }
    }
    /// The world event, if any.
    pub fn world(&self) -> Option<&WorldEvent> {
        match self {
            Self::World(event) => Some(event),
            _ => None,
        }
    }
}

/// Read-only context for one [`Service::update`].
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    world: &'a World,
    now: Instant,
}

impl<'a> Context<'a> {
    /// For calling `update` directly, as tests do.
    pub fn new(world: &'a World, now: Instant) -> Self {
        Self { world, now }
    }
    /// The server mirror, already updated for this input.
    pub fn world(&self) -> &'a World {
        self.world
    }
    /// The time of this input. Use it over `Instant::now()` to keep updates
    /// deterministic.
    pub fn now(&self) -> Instant {
        self.now
    }
}

pub(crate) type AnyMsg = Box<dyn Any + Send>;
/// One click-id pool per connection, shared by every service drawing on it.
pub(crate) type Pools = HashMap<ConnectionId, ClickIdPool>;

/// A service with its message type erased, so the host can hold several, plus
/// its per-player screens.
pub(crate) trait ErasedService: Send {
    fn update(
        &mut self,
        input: Input<AnyMsg>,
        cx: &Context<'_>,
        pools: &mut Pools,
    ) -> Effects<AnyMsg>;
    /// Deliver a message to the components placed with `key`, for one player
    /// or all.
    fn send(
        &mut self,
        ucid: Option<ConnectionId>,
        key: &dyn Key,
        msg: &dyn Any,
        cx: &Context<'_>,
    ) -> Effects<AnyMsg>;
    fn needs_render(&self) -> bool;
    fn render(&mut self, cx: &Context<'_>, pools: &mut Pools) -> Vec<Packet>;
    /// Remove every button this service has drawn.
    fn clear(&mut self, pools: &mut Pools) -> Vec<Packet>;
}

pub(crate) struct Typed<S: Service> {
    service: S,
    screens: HashMap<ConnectionId, Screen<S::Msg>>,
    // State changed, so every screen needs redrawing.
    dirty: bool,
}

impl<S: Service> Typed<S> {
    pub(crate) fn new(service: S) -> Self {
        Self {
            service,
            screens: HashMap::new(),
            dirty: true,
        }
    }
    fn run(&mut self, input: Input<S::Msg>, cx: &Context<'_>) -> Effects<AnyMsg> {
        self.dirty = true;
        self.service
            .update(input, cx)
            .map(|msg| Box::new(msg) as AnyMsg)
    }
    fn run_all(&mut self, msgs: Vec<S::Msg>, cx: &Context<'_>) -> Effects<AnyMsg> {
        msgs.into_iter().fold(Effects::new(), |effects, msg| {
            effects.and(self.run(Input::Msg(msg), cx))
        })
    }
}

impl<S: Service> ErasedService for Typed<S> {
    fn update(
        &mut self,
        input: Input<AnyMsg>,
        cx: &Context<'_>,
        pools: &mut Pools,
    ) -> Effects<AnyMsg> {
        let input = input.map(|msg| {
            *msg.downcast::<S::Msg>()
                .expect("the host only delivers a service its own messages")
        });
        // Button packets also become messages from the view or its components.
        let clicked = match &input {
            Input::Packet(
                packet @ (Packet::Btc(insim::insim::Btc { ucid, .. })
                | Packet::Btt(insim::insim::Btt { ucid, .. })
                | Packet::Bfn(insim::insim::Bfn { ucid, .. })),
            ) => match (self.screens.get_mut(ucid), pools.get_mut(ucid)) {
                (Some(screen), Some(pool)) => screen.handle(packet, pool),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        let effects = self.run(input, cx);
        effects.and(self.run_all(clicked, cx))
    }

    fn send(
        &mut self,
        ucid: Option<ConnectionId>,
        key: &dyn Key,
        msg: &dyn Any,
        cx: &Context<'_>,
    ) -> Effects<AnyMsg> {
        let mut msgs = Vec::new();
        for (screen_ucid, screen) in &mut self.screens {
            if ucid.is_none_or(|ucid| ucid == *screen_ucid) {
                msgs.extend(screen.send(key, msg));
            }
        }
        self.run_all(msgs, cx)
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.screens.values().any(Screen::is_dirty)
    }

    fn render(&mut self, cx: &Context<'_>, pools: &mut Pools) -> Vec<Packet> {
        // Excludes the server's own connection (UCID 0).
        let connections: Vec<ConnectionId> = cx.world().connections().map(|c| c.ucid).collect();
        // Players who left take their buttons with them.
        self.screens.retain(|ucid, _| connections.contains(ucid));
        let mut packets = Vec::new();
        for ucid in connections {
            let screen = self
                .screens
                .entry(ucid)
                .or_insert_with(|| Screen::new(ucid));
            if !(self.dirty || screen.is_dirty()) {
                continue;
            }
            let pool = pools.entry(ucid).or_insert_with(ClickIdPool::new);
            let service = &self.service;
            packets.extend(screen.render(pool, cx.now(), |ui| service.view(ucid, ui, cx)));
        }
        self.dirty = false;
        packets
    }

    fn clear(&mut self, pools: &mut Pools) -> Vec<Packet> {
        let mut packets = Vec::new();
        for (ucid, mut screen) in self.screens.drain() {
            if let Some(pool) = pools.get_mut(&ucid) {
                packets.extend(screen.clear(pool));
            }
        }
        packets
    }
}
