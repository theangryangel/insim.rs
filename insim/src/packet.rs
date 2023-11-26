//! Insim v9 Packet definitions

use insim_core::identifiers::RequestId;
use insim_core::prelude::*;

use crate::insim::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::relay;

#[derive(InsimEncode, InsimDecode, Debug, Clone, from_variants::FromVariants)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
#[repr(u8)]
/// Enum representing all possible packets receivable via an Insim connection
pub enum Packet {
    Init(Isi) = 1,
    Version(Version) = 2,
    Tiny(Tiny) = 3,
    Small(Small) = 4,
    State(Sta) = 5,
    SingleCharacter(Sch) = 6,
    StateFlagsPack(Sfp) = 7,
    SetCarCam(Scc) = 8,
    CamPosPack(Cpp) = 9,
    MultiPlayerNotification(Ism) = 10,
    MessageOut(Mso) = 11,
    InsimInfo(Iii) = 12,
    MessageType(Mst) = 13,
    MessageToConnection(Mtc) = 14,
    ScreenMode(Mod) = 15,
    VoteNotification(Vtn) = 16,
    RaceStart(Rst) = 17,
    NewConnection(Ncn) = 18,
    ConnectionLeave(Cnl) = 19,
    ConnectionPlayerRenamed(Cpr) = 20,
    NewPlayer(Npl) = 21,
    PlayerPits(Plp) = 22,
    PlayerLeave(Pll) = 23,
    Lap(Lap) = 24,
    SplitX(Spx) = 25,
    PitStopStart(Pit) = 26,
    PitStopFinish(Psf) = 27,
    PitLane(Pla) = 28,
    CameraChange(Cch) = 29,
    Penalty(Pen) = 30,
    TakeOverCar(Toc) = 31,
    Flag(Flg) = 32,
    PlayerFlags(Pfl) = 33,
    Finished(Fin) = 34,
    Result(Res) = 35,
    Reorder(Reo) = 36,
    NodeLap(Nlp) = 37,
    MultiCarInfo(Mci) = 38,
    MesssageExtended(Msx) = 39,
    MessageLocal(Msl) = 40,
    CarReset(Crs) = 41,
    ButtonFunction(Bfn) = 42,
    AutoXInfo(Axi) = 43,
    AutoXObject(Axo) = 44,
    Button(Btn) = 45,
    ButtonClick(Btc) = 46,
    ButtonType(Btt) = 47,
    ReplayInformation(Rip) = 48,
    ScreenShot(Ssh) = 49,
    Contact(Con) = 50,
    ObjectHit(Obh) = 51,
    HotLapValidity(Hlv) = 52,
    PlayerAllowedCars(Plc) = 53,
    AutoXMultipleObjects(Axm) = 54,
    AdminCommandReport(Acr) = 55,
    Handicaps(Hcp) = 56,
    Nci(Nci) = 57,
    Jrr(Jrr) = 58,
    UserControlObject(Uco) = 59,
    ObjectControl(Oco) = 60,
    TargetToConnection(Ttc) = 61,
    SelectedVehicle(Slc) = 62,
    VehicleStateChanged(Csc) = 63,
    ConnectionInterfaceMode(Cim) = 64,
    ModsAllowed(Mal) = 65,

    RelayAdminRequest(relay::AdminRequest) = 250,
    RelayAdminResponse(relay::AdminResponse) = 251,
    RelayHostListRequest(relay::HostListRequest) = 252,
    RelayHostList(relay::HostList) = 253,
    RelayHostSelect(relay::HostSelect) = 254,
    RelayError(relay::RelayError) = 255,
}

impl Default for Packet {
    fn default() -> Self {
        Self::Tiny(Tiny::default())
    }
}

impl Packet {
    #[tracing::instrument]
    pub fn maybe_pong(&self) -> Option<Self> {
        match self {
            Packet::Tiny(Tiny {
                subt: TinyType::None,
                reqi: RequestId(0),
            }) => Some(Self::Tiny(Tiny {
                reqi: RequestId(0),
                subt: TinyType::None,
            })),
            _ => None,
        }
    }

    #[tracing::instrument]
    pub fn maybe_verify_version(&self) -> crate::result::Result<bool> {
        match self {
            Packet::Version(Version { insimver, .. }) => {
                if *insimver != crate::VERSION {
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
