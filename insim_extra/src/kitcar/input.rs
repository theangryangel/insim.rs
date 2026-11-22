//! [`FromInput`]: pick one typed value out of an [`Input`].

use insim::Packet;

use crate::{
    kitcar::Input,
    world::{
        AllowedCarsChanged, AllowedModsChanged, Connected, ConnectionDetails, Disconnected,
        LayoutChanged, LobbyEntered, LobbyNotReady, LobbyReady, MultiplayerJoined, MultiplayerLeft,
        PlayerJoined, PlayerLeft, PlayerTeleportedToPits, RaceEvent, Renamed, SessionEnded,
        SessionStarted, TakingOver, TrackChanged, VehicleSelected, VersionReceived, WorldEvent,
        WorldReady,
    },
};

/// A value an [`Input`] can carry: a packet such as [`Uco`](insim::insim::Uco),
/// a world event such as [`Connected`], or a marker ([`Started`],
/// [`Stopping`]).
///
/// Powers [`Input::get`] and
/// [`Router::on`](crate::kitcar::services::router::Router::on). Implemented
/// for [`Packet`] and every packet type, [`WorldEvent`] and its payloads, and
/// [`RaceEvent`].
pub trait FromInput {
    /// The value, if present.
    fn from_input<M>(input: &Input<M>) -> Option<&Self>;
}

/// Every packet type, and [`Packet`].
impl<T> FromInput for T
where
    for<'a> &'a T: TryFrom<&'a Packet>,
{
    fn from_input<M>(input: &Input<M>) -> Option<&Self> {
        match input {
            Input::Packet(packet) => <&T>::try_from(packet).ok(),
            _ => None,
        }
    }
}

/// [`Input::Started`], as a type: `router.on(|state, _: &Started, cx| ..)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Started;

impl FromInput for Started {
    fn from_input<M>(input: &Input<M>) -> Option<&Self> {
        matches!(input, Input::Started).then_some(&Started)
    }
}

/// [`Input::Stopping`], as a type: `router.on(|state, _: &Stopping, cx| ..)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stopping;

impl FromInput for Stopping {
    fn from_input<M>(input: &Input<M>) -> Option<&Self> {
        matches!(input, Input::Stopping).then_some(&Stopping)
    }
}

impl FromInput for WorldEvent {
    fn from_input<M>(input: &Input<M>) -> Option<&Self> {
        input.world()
    }
}

macro_rules! world_events {
    ($($variant:ident($ty:ty)),* $(,)?) => {
        $(
            impl FromInput for $ty {
                fn from_input<M>(input: &Input<M>) -> Option<&Self> {
                    match input {
                        Input::World(WorldEvent::$variant(event)) => Some(event),
                        _ => None,
                    }
                }
            }
        )*
    };
}

world_events! {
    Connected(Connected),
    Disconnected(Disconnected),
    ConnectionDetails(ConnectionDetails),
    VehicleSelected(VehicleSelected),
    Renamed(Renamed),
    PlayerJoined(PlayerJoined),
    PlayerLeft(PlayerLeft),
    TakingOver(TakingOver),
    PlayerTeleportedToPits(PlayerTeleportedToPits),
    SessionStarted(SessionStarted),
    SessionEnded(SessionEnded),
    LobbyEntered(LobbyEntered),
    LobbyReady(LobbyReady),
    LobbyNotReady(LobbyNotReady),
    TrackChanged(TrackChanged),
    LayoutChanged(LayoutChanged),
    MultiplayerJoined(MultiplayerJoined),
    MultiplayerLeft(MultiplayerLeft),
    AllowedCarsChanged(AllowedCarsChanged),
    AllowedModsChanged(AllowedModsChanged),
    VersionReceived(VersionReceived),
    Race(RaceEvent),
    Ready(WorldReady),
}

impl<M> Input<M> {
    /// The `T` this input carries: `input.get::<Uco>()`.
    pub fn get<T: FromInput + ?Sized>(&self) -> Option<&T> {
        T::from_input(self)
    }
}

#[cfg(test)]
mod tests {
    use insim::insim::{Mso, Tiny};

    use super::*;

    #[test]
    fn picks_packets_and_markers() {
        let input = Input::<()>::Packet(Mso::default().into());
        assert!(input.get::<Mso>().is_some());
        assert!(input.get::<Packet>().is_some());
        assert!(input.get::<Tiny>().is_none());
        assert!(input.get::<Started>().is_none());

        assert!(Input::<()>::Started.get::<Started>().is_some());
        assert!(Input::<()>::Stopping.get::<Stopping>().is_some());
        assert!(Input::<()>::Stopping.get::<Packet>().is_none());
    }
}
