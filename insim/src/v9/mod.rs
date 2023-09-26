//! Insim and Insim Relay Packet definitions
use insim_core::identifiers::RequestId;
use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Insim packet definitions
pub mod insim;

const VERSION: u8 = 9;

use crate::{codec::Frame, relay};

#[derive(InsimEncode, InsimDecode, Debug, Clone, from_variants::FromVariants)]
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

impl Frame for Packet {
    type Init = insim::Isi;

    fn maybe_pong(&self) -> Option<Self> {
        use self::insim::TinyType;

        match self {
            Packet::Tiny(insim::Tiny {
                subt: TinyType::None,
                reqi: RequestId(0),
            }) => Some(Self::Tiny(insim::Tiny {
                reqi: RequestId(0),
                subt: insim::TinyType::None,
            })),
            _ => None,
        }
    }

    fn maybe_verify_version(&self) -> crate::result::Result<bool> {
        match self {
            Packet::Version(insim::Version { insimver, .. }) => {
                if *insimver != VERSION {
                    return Err(crate::error::Error::IncompatibleVersion(*insimver));
                }

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    // The majority of packet conversions are tested through insim_core.
    // Any packets implementing their own InsimEncode InsimDecode should have their own test
    // We could test every packet, but at a certain point we're just testing insim_core and
    // insim_derive all over again.
}
