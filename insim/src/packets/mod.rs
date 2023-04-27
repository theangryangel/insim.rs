//! Insim and Insim Relay Packet definitions
use futures::{Sink, Stream};
use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;
mod macros;

/// Insim packet definitions
pub mod insim;

#[cfg(feature = "relay")]
/// Relay packet definitions
pub mod relay;

pub trait PacketSinkStream:
    Sink<Packet, Error = crate::error::Error>
    + Stream<Item = crate::result::Result<Packet>>
    + std::marker::Unpin
    + Send
{
}

pub trait RequestIdentifiable {
    fn request_identifier(&self) -> RequestId;

    fn set_request_identifier(&mut self, reqi: RequestId);
}

/// This Insim protocol version number
pub const VERSION: u8 = 9;

#[derive(InsimEncode, InsimDecode, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
#[repr(u8)]
/// Enum representing all possible packets receivable via an Insim connection
pub enum Packet {
    Init(insim::Isi) = 1,
    Version(insim::Version) = 2,
    Tiny(insim::Tiny) = 3,
    Small(insim::Small) = 4,
    State(insim::Sta) = 5,
    SingleCharacter(insim::Sch) = 6,
    StateFlagsPack(insim::Sfp) = 7,
    SetCarCam(insim::Scc) = 8,
    CamPosPack(insim::Cpp) = 9,
    MultiPlayerNotification(insim::Ism) = 10,
    MessageOut(insim::Mso) = 11,
    InsimInfo(insim::Iii) = 12,
    MessageType(insim::Mst) = 13,
    MessageToConnection(insim::Mtc) = 14,
    ScreenMode(insim::Mod) = 15,
    VoteNotification(insim::Vtn) = 16,
    RaceStart(insim::Rst) = 17,
    NewConnection(insim::Ncn) = 18,
    ConnectionLeave(insim::Cnl) = 19,
    ConnectionPlayerRenamed(insim::Cpr) = 20,
    NewPlayer(insim::Npl) = 21,
    PlayerPits(insim::Plp) = 22,
    PlayerLeave(insim::Pll) = 23,
    Lap(insim::Lap) = 24,
    SplitX(insim::Spx) = 25,
    PitStopStart(insim::Pit) = 26,
    PitStopFinish(insim::Psf) = 27,
    PitLane(insim::Pla) = 28,
    CameraChange(insim::Cch) = 29,
    Penalty(insim::Pen) = 30,
    TakeOverCar(insim::Toc) = 31,
    Flag(insim::Flg) = 32,
    PlayerFlags(insim::Pfl) = 33,
    Finished(insim::Fin) = 34,
    Result(insim::Res) = 35,
    Reorder(insim::Reo) = 36,
    NodeLap(insim::Nlp) = 37,
    MultiCarInfo(insim::Mci) = 38,
    MesssageExtended(insim::Msx) = 39,
    MessageLocal(insim::Msl) = 40,
    CarReset(insim::Crs) = 41,
    ButtonFunction(insim::Bfn) = 42,
    AutoXInfo(insim::Axi) = 43,
    AutoXObject(insim::Axo) = 44,
    Button(insim::Btn) = 45,
    ButtonClick(insim::Btc) = 46,
    ButtonType(insim::Btt) = 47,
    ReplayInformation(insim::Rip) = 48,
    ScreenShot(insim::Ssh) = 49,
    Contact(insim::Con) = 50,
    ObjectHit(insim::Obh) = 51,
    HotLapValidity(insim::Hlv) = 52,
    PlayerAllowedCars(insim::Plc) = 53,
    AutoXMultipleObjects(insim::Axm) = 54,
    AdminCommandReport(insim::Acr) = 55,
    Handicaps(insim::Hcp) = 56,
    Nci(insim::Nci) = 57,
    Jrr(insim::Jrr) = 58,
    UserControlObject(insim::Uco) = 59,
    ObjectControl(insim::Oco) = 60,
    TargetToConnection(insim::Ttc) = 61,
    SelectedVehicle(insim::Slc) = 62,
    VehicleStateChanged(insim::Csc) = 63,
    ConnectionInterfaceMode(insim::Cim) = 64,
    ModsAllowed(insim::Mal) = 65,

    #[cfg(feature = "relay")]
    RelayAdminRequest(relay::AdminRequest) = 250,

    #[cfg(feature = "relay")]
    RelayAdminResponse(relay::AdminResponse) = 251,

    #[cfg(feature = "relay")]
    RelayHostListRequest(relay::HostListRequest) = 252,

    #[cfg(feature = "relay")]
    RelayHostList(relay::HostList) = 253,

    #[cfg(feature = "relay")]
    RelayHostSelect(relay::HostSelect) = 254,

    #[cfg(feature = "relay")]
    RelayError(relay::RelayError) = 255,
}

impl Default for Packet {
    fn default() -> Self {
        Self::Tiny(insim::Tiny::default())
    }
}

impl RequestIdentifiable for Packet {
    fn request_identifier(&self) -> RequestId {
        match self {
            Packet::Init(i) => i.request_identifier(),
            Packet::Version(i) => i.request_identifier(),
            Packet::Tiny(i) => i.request_identifier(),
            Packet::Small(i) => i.request_identifier(),
            Packet::State(i) => i.request_identifier(),
            Packet::SingleCharacter(i) => i.request_identifier(),
            Packet::StateFlagsPack(i) => i.request_identifier(),
            Packet::SetCarCam(i) => i.request_identifier(),
            Packet::CamPosPack(i) => i.request_identifier(),
            Packet::MultiPlayerNotification(i) => i.request_identifier(),
            Packet::MessageOut(i) => i.request_identifier(),
            Packet::InsimInfo(i) => i.request_identifier(),
            Packet::MessageType(i) => i.request_identifier(),
            Packet::MessageToConnection(i) => i.request_identifier(),
            Packet::ScreenMode(i) => i.request_identifier(),
            Packet::VoteNotification(i) => i.request_identifier(),
            Packet::RaceStart(i) => i.request_identifier(),
            Packet::NewConnection(i) => i.request_identifier(),
            Packet::ConnectionLeave(i) => i.request_identifier(),
            Packet::ConnectionPlayerRenamed(i) => i.request_identifier(),
            Packet::NewPlayer(i) => i.request_identifier(),
            Packet::PlayerPits(i) => i.request_identifier(),
            Packet::PlayerLeave(i) => i.request_identifier(),
            Packet::Lap(i) => i.request_identifier(),
            Packet::SplitX(i) => i.request_identifier(),
            Packet::PitStopStart(i) => i.request_identifier(),
            Packet::PitStopFinish(i) => i.request_identifier(),
            Packet::PitLane(i) => i.request_identifier(),
            Packet::CameraChange(i) => i.request_identifier(),
            Packet::Penalty(i) => i.request_identifier(),
            Packet::TakeOverCar(i) => i.request_identifier(),
            Packet::Flag(i) => i.request_identifier(),
            Packet::PlayerFlags(i) => i.request_identifier(),
            Packet::Finished(i) => i.request_identifier(),
            Packet::Result(i) => i.request_identifier(),
            Packet::Reorder(i) => i.request_identifier(),
            Packet::NodeLap(i) => i.request_identifier(),
            Packet::MultiCarInfo(i) => i.request_identifier(),
            Packet::MesssageExtended(i) => i.request_identifier(),
            Packet::MessageLocal(i) => i.request_identifier(),
            Packet::CarReset(i) => i.request_identifier(),
            Packet::ButtonFunction(i) => i.request_identifier(),
            Packet::AutoXInfo(i) => i.request_identifier(),
            Packet::AutoXObject(i) => i.request_identifier(),
            Packet::Button(i) => i.request_identifier(),
            Packet::ButtonClick(i) => i.request_identifier(),
            Packet::ButtonType(i) => i.request_identifier(),
            Packet::ReplayInformation(i) => i.request_identifier(),
            Packet::ScreenShot(i) => i.request_identifier(),
            Packet::Contact(i) => i.request_identifier(),
            Packet::ObjectHit(i) => i.request_identifier(),
            Packet::HotLapValidity(i) => i.request_identifier(),
            Packet::PlayerAllowedCars(i) => i.request_identifier(),
            Packet::AutoXMultipleObjects(i) => i.request_identifier(),
            Packet::AdminCommandReport(i) => i.request_identifier(),
            Packet::Handicaps(i) => i.request_identifier(),
            Packet::Nci(i) => i.request_identifier(),
            Packet::Jrr(i) => i.request_identifier(),
            Packet::UserControlObject(i) => i.request_identifier(),
            Packet::ObjectControl(i) => i.request_identifier(),
            Packet::TargetToConnection(i) => i.request_identifier(),
            Packet::SelectedVehicle(i) => i.request_identifier(),
            Packet::VehicleStateChanged(i) => i.request_identifier(),
            Packet::ConnectionInterfaceMode(i) => i.request_identifier(),
            Packet::ModsAllowed(i) => i.request_identifier(),

            #[cfg(feature = "relay")]
            Packet::RelayAdminRequest(i) => i.request_identifier(),
            #[cfg(feature = "relay")]
            Packet::RelayAdminResponse(i) => i.request_identifier(),
            #[cfg(feature = "relay")]
            Packet::RelayHostListRequest(i) => i.request_identifier(),
            #[cfg(feature = "relay")]
            Packet::RelayHostList(i) => i.request_identifier(),
            #[cfg(feature = "relay")]
            Packet::RelayHostSelect(i) => i.request_identifier(),
            #[cfg(feature = "relay")]
            Packet::RelayError(i) => i.request_identifier(),
        }
    }

    fn set_request_identifier(&mut self, reqi: RequestId) {
        match self {
            Packet::Init(i) => i.set_request_identifier(reqi),
            Packet::Version(i) => i.set_request_identifier(reqi),
            Packet::Tiny(i) => i.set_request_identifier(reqi),
            Packet::Small(i) => i.set_request_identifier(reqi),
            Packet::State(i) => i.set_request_identifier(reqi),
            Packet::SingleCharacter(i) => i.set_request_identifier(reqi),
            Packet::StateFlagsPack(i) => i.set_request_identifier(reqi),
            Packet::SetCarCam(i) => i.set_request_identifier(reqi),
            Packet::CamPosPack(i) => i.set_request_identifier(reqi),
            Packet::MultiPlayerNotification(i) => i.set_request_identifier(reqi),
            Packet::MessageOut(i) => i.set_request_identifier(reqi),
            Packet::InsimInfo(i) => i.set_request_identifier(reqi),
            Packet::MessageType(i) => i.set_request_identifier(reqi),
            Packet::MessageToConnection(i) => i.set_request_identifier(reqi),
            Packet::ScreenMode(i) => i.set_request_identifier(reqi),
            Packet::VoteNotification(i) => i.set_request_identifier(reqi),
            Packet::RaceStart(i) => i.set_request_identifier(reqi),
            Packet::NewConnection(i) => i.set_request_identifier(reqi),
            Packet::ConnectionLeave(i) => i.set_request_identifier(reqi),
            Packet::ConnectionPlayerRenamed(i) => i.set_request_identifier(reqi),
            Packet::NewPlayer(i) => i.set_request_identifier(reqi),
            Packet::PlayerPits(i) => i.set_request_identifier(reqi),
            Packet::PlayerLeave(i) => i.set_request_identifier(reqi),
            Packet::Lap(i) => i.set_request_identifier(reqi),
            Packet::SplitX(i) => i.set_request_identifier(reqi),
            Packet::PitStopStart(i) => i.set_request_identifier(reqi),
            Packet::PitStopFinish(i) => i.set_request_identifier(reqi),
            Packet::PitLane(i) => i.set_request_identifier(reqi),
            Packet::CameraChange(i) => i.set_request_identifier(reqi),
            Packet::Penalty(i) => i.set_request_identifier(reqi),
            Packet::TakeOverCar(i) => i.set_request_identifier(reqi),
            Packet::Flag(i) => i.set_request_identifier(reqi),
            Packet::PlayerFlags(i) => i.set_request_identifier(reqi),
            Packet::Finished(i) => i.set_request_identifier(reqi),
            Packet::Result(i) => i.set_request_identifier(reqi),
            Packet::Reorder(i) => i.set_request_identifier(reqi),
            Packet::NodeLap(i) => i.set_request_identifier(reqi),
            Packet::MultiCarInfo(i) => i.set_request_identifier(reqi),
            Packet::MesssageExtended(i) => i.set_request_identifier(reqi),
            Packet::MessageLocal(i) => i.set_request_identifier(reqi),
            Packet::CarReset(i) => i.set_request_identifier(reqi),
            Packet::ButtonFunction(i) => i.set_request_identifier(reqi),
            Packet::AutoXInfo(i) => i.set_request_identifier(reqi),
            Packet::AutoXObject(i) => i.set_request_identifier(reqi),
            Packet::Button(i) => i.set_request_identifier(reqi),
            Packet::ButtonClick(i) => i.set_request_identifier(reqi),
            Packet::ButtonType(i) => i.set_request_identifier(reqi),
            Packet::ReplayInformation(i) => i.set_request_identifier(reqi),
            Packet::ScreenShot(i) => i.set_request_identifier(reqi),
            Packet::Contact(i) => i.set_request_identifier(reqi),
            Packet::ObjectHit(i) => i.set_request_identifier(reqi),
            Packet::HotLapValidity(i) => i.set_request_identifier(reqi),
            Packet::PlayerAllowedCars(i) => i.set_request_identifier(reqi),
            Packet::AutoXMultipleObjects(i) => i.set_request_identifier(reqi),
            Packet::AdminCommandReport(i) => i.set_request_identifier(reqi),
            Packet::Handicaps(i) => i.set_request_identifier(reqi),
            Packet::Nci(i) => i.set_request_identifier(reqi),
            Packet::Jrr(i) => i.set_request_identifier(reqi),
            Packet::UserControlObject(i) => i.set_request_identifier(reqi),
            Packet::ObjectControl(i) => i.set_request_identifier(reqi),
            Packet::TargetToConnection(i) => i.set_request_identifier(reqi),
            Packet::SelectedVehicle(i) => i.set_request_identifier(reqi),
            Packet::VehicleStateChanged(i) => i.set_request_identifier(reqi),
            Packet::ConnectionInterfaceMode(i) => i.set_request_identifier(reqi),
            Packet::ModsAllowed(i) => i.set_request_identifier(reqi),

            #[cfg(feature = "relay")]
            Packet::RelayAdminRequest(i) => i.set_request_identifier(reqi),
            #[cfg(feature = "relay")]
            Packet::RelayAdminResponse(i) => i.set_request_identifier(reqi),
            #[cfg(feature = "relay")]
            Packet::RelayHostListRequest(i) => i.set_request_identifier(reqi),
            #[cfg(feature = "relay")]
            Packet::RelayHostList(i) => i.set_request_identifier(reqi),
            #[cfg(feature = "relay")]
            Packet::RelayHostSelect(i) => i.set_request_identifier(reqi),
            #[cfg(feature = "relay")]
            Packet::RelayError(i) => i.set_request_identifier(reqi),
        }
    }
}

crate::impl_packet_traits! {
    insim::Isi => Init,
    insim::Version => Version,
    insim::Tiny => Tiny,
    insim::Small => Small,
    insim::Sta => State,
    insim::Sch => SingleCharacter,
    insim::Sfp => StateFlagsPack,
    insim::Scc => SetCarCam,
    insim::Cpp => CamPosPack,
    insim::Ism => MultiPlayerNotification,
    insim::Mso => MessageOut,
    insim::Iii => InsimInfo,
    insim::Mst => MessageType,
    insim::Mtc => MessageToConnection,
    insim::Mod => ScreenMode,
    insim::Vtn => VoteNotification,
    insim::Rst => RaceStart,
    insim::Ncn => NewConnection,
    insim::Cnl => ConnectionLeave,
    insim::Cpr => ConnectionPlayerRenamed,
    insim::Npl => NewPlayer,
    insim::Plp => PlayerPits,
    insim::Pll => PlayerLeave,
    insim::Lap => Lap,
    insim::Spx => SplitX,
    insim::Pit => PitStopStart,
    insim::Psf => PitStopFinish,
    insim::Pla => PitLane,
    insim::Cch => CameraChange,
    insim::Pen => Penalty,
    insim::Toc => TakeOverCar,
    insim::Flg => Flag,
    insim::Pfl => PlayerFlags,
    insim::Fin => Finished,
    insim::Res => Result,
    insim::Reo => Reorder,
    insim::Nlp => NodeLap,
    insim::Mci => MultiCarInfo,
    insim::Msx => MesssageExtended,
    insim::Msl => MessageLocal,
    insim::Crs => CarReset,
    insim::Bfn => ButtonFunction,
    insim::Axi => AutoXInfo,
    insim::Axo => AutoXObject,
    insim::Btn => Button,
    insim::Btc => ButtonClick,
    insim::Btt => ButtonType,
    insim::Rip => ReplayInformation,
    insim::Ssh => ScreenShot,
    insim::Con => Contact,
    insim::Obh => ObjectHit,
    insim::Hlv => HotLapValidity,
    insim::Plc => PlayerAllowedCars,
    insim::Axm => AutoXMultipleObjects,
    insim::Acr => AdminCommandReport,
    insim::Hcp => Handicaps,
    insim::Nci => Nci,
    insim::Jrr => Jrr,
    insim::Uco => UserControlObject,
    insim::Oco => ObjectControl,
    insim::Ttc => TargetToConnection,
    insim::Slc => SelectedVehicle,
    insim::Csc => VehicleStateChanged,
    insim::Cim => ConnectionInterfaceMode,
    insim::Mal => ModsAllowed,
}

#[cfg(feature = "relay")]
crate::impl_packet_traits! {
    relay::AdminRequest => RelayAdminRequest,
    relay::AdminResponse => RelayAdminResponse,
    relay::HostListRequest => RelayHostListRequest,
    relay::HostList => RelayHostList,
    relay::HostSelect => RelayHostSelect,
    relay::RelayError => RelayError,
}

#[cfg(test)]
mod tests {
    // The majority of packet conversions are tested through insim_core.
    // Any packets implementing their own InsimEncode InsimDecode should have their own test
    // We could test every packet, but at a certain point we're just testing insim_core and
    // insim_derive all over again.
}
