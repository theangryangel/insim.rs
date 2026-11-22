//! [`Service`]: a game or service run by a [`Host`](crate::kitcar::Host), written as one
//! update function over its own state.

use std::{any::Any, collections::HashMap, time::Instant};

use insim::{Packet, identifiers::ConnectionId};

use crate::{
    kitcar::Effects,
    ui::{Key, Node, Screen, Ui, id_pool::ClickIdPool},
    world::{World, WorldEvent},
};

/// Something a [`Host`](crate::kitcar::Host) runs on a connection: a game such as Bomb,
/// the idle screen, or a long-lived service such as licensing.
///
/// Unlike tower's `Service`, this is not request/response: it is a stateful
/// program that sees every input on the connection, in order.
///
/// The host calls [`update`](Self::update) for every input, one at a time and
/// in order, with `&mut self`: state needs no locks. `update` is synchronous and
/// does no I/O. It returns [`Effects`], which the host carries out; slow work
/// such as database queries goes in [`Effects::task`] and comes back as a
/// message.
///
/// Reusable pieces (setup, restrictions, penalty clearing) are plain structs
/// with their own update methods, held as fields and combined with
/// [`Effects::and`] and [`Effects::map`].
///
/// A service with buttons implements [`view`](Self::view): it draws each
/// player's buttons straight from the service's state, so there is nothing to
/// keep in sync. The host redraws after every update and sends only what
/// changed. Per-player UI state (an open dialog, a scroll position) lives in
/// [`Component`](crate::ui::Component)s placed with
/// [`Ui::component`](crate::ui::Ui::component), which the host keeps for
/// each player.
pub trait Service: Send + 'static {
    /// The service's own messages: timer payloads, task results, button clicks.
    type Msg: Clone + Send + 'static;

    /// React to one input.
    fn update(&mut self, input: Input<Self::Msg>, cx: &Context<'_>) -> Effects<Self::Msg>;

    /// Draw one player's buttons, or `None` for none. Clicks on the view's own
    /// buttons arrive as [`Input::Msg`]. The host never draws for the local
    /// connection (UCID 0).
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
    /// The service has been started. Arm timers, attach streams, set up the track.
    Started,
    /// A packet from LFS. The world already reflects it.
    Packet(Packet),
    /// A world change caused by the preceding packet, or
    /// [`WorldEvent::Ready`] once the world has caught up after connecting.
    World(WorldEvent),
    /// One of the service's own messages.
    Msg(M),
    /// The service is being stopped. Packets returned now are still sent; timers and
    /// tasks are then dropped.
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
    /// The packet, if this input is one.
    pub fn packet(&self) -> Option<&Packet> {
        match self {
            Self::Packet(packet) => Some(packet),
            _ => None,
        }
    }
    /// The world event, if this input is one.
    pub fn world(&self) -> Option<&WorldEvent> {
        match self {
            Self::World(event) => Some(event),
            _ => None,
        }
    }
}

/// Read-only context for one [`Service::update`] call.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    world: &'a World,
    now: Instant,
}

impl<'a> Context<'a> {
    /// Context for calling `update` directly, as tests do.
    pub fn new(world: &'a World, now: Instant) -> Self {
        Self { world, now }
    }
    /// The server mirror, already updated for the current input.
    pub fn world(&self) -> &'a World {
        self.world
    }
    /// The time of this input. Use it instead of `Instant::now()` so that
    /// updates stay deterministic and testable.
    pub fn now(&self) -> Instant {
        self.now
    }
}

pub(crate) type AnyMsg = Box<dyn Any + Send>;
/// One click-id pool per connection, shared by every service drawing on it.
pub(crate) type Pools = HashMap<ConnectionId, ClickIdPool>;

/// A service with its message type erased, so the host can hold several. It
/// also owns the service's per-player screens.
pub(crate) trait ErasedService: Send {
    fn update(
        &mut self,
        input: Input<AnyMsg>,
        cx: &Context<'_>,
        pools: &mut Pools,
    ) -> Effects<AnyMsg>;
    /// Deliver a message to the components placed with `key` for one player, or
    /// every player.
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
    // The service's state may have changed, so every screen needs redrawing.
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
        // Players' connections only: never the server's own (UCID 0).
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
