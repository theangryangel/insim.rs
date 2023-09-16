//! Insim and Insim Relay Packet definitions
use insim_core::identifiers::RequestId;
use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Insim packet definitions
pub mod insim;

const VERSION: u8 = 9;

use crate::relay;

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

    RelayAdminRequest(relay::AdminRequest) = 250,
    RelayAdminResponse(relay::AdminResponse) = 251,
    RelayHostListRequest(relay::HostListRequest) = 252,
    RelayHostList(relay::HostList) = 253,
    RelayHostSelect(relay::HostSelect) = 254,
    RelayError(relay::RelayError) = 255,
}

impl Default for Packet {
    fn default() -> Self {
        Self::Tiny(insim::Tiny::default())
    }
}

crate::impl_packet_from! {
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

    relay::AdminRequest => RelayAdminRequest,
    relay::AdminResponse => RelayAdminResponse,
    relay::HostListRequest => RelayHostListRequest,
    relay::HostList => RelayHostList,
    relay::HostSelect => RelayHostSelect,
    relay::RelayError => RelayError,
}

impl Packet {
    pub fn is_error(&self) -> bool {
        matches!(self, Self::RelayError(_))
    }
}

use crate::codec;

impl codec::Packets for Packet {
    fn is_ping(&self) -> bool {
        matches!(self, Packet::Tiny(_))
    }

    fn pong(reqi: Option<RequestId>) -> Self {
        insim::Tiny {
            reqi: reqi.unwrap_or(RequestId(0)),
            subt: insim::TinyType::None,
        }.into()
    }

    fn maybe_verify_version(&self) -> crate::result::Result<bool> {
        match self {
            Packet::Version(insim::Version { insimver, .. }) => {
                if *insimver != VERSION {
                    return Err(crate::error::Error::IncompatibleVersion(*insimver));
                }

                Ok(true)
            }, 
            _ => {
                return Ok(false)
            }
        }
    }
}

pub struct Codec {
    pub mode: codec::Mode,
}

impl codec::Codec for Codec {
    type Item = Packet;

    fn mode(&self) -> codec::Mode {
        self.mode
    }
}

#[cfg(test)]
mod tests {
    // The majority of packet conversions are tested through insim_core.
    // Any packets implementing their own InsimEncode InsimDecode should have their own test
    // We could test every packet, but at a certain point we're just testing insim_core and
    // insim_derive all over again.
}
