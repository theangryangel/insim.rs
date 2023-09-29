//! Insim v9 Packet definitions

use insim_core::identifiers::RequestId;
use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

const VERSION: u8 = 9;

use crate::{codec::Frame, relay};

mod acr;
mod axi;
mod axm;
mod axo;
mod btn;
mod cch;
mod cim;
mod cnl;
mod con;
mod cpp;
mod cpr;
mod crs;
mod csc;
mod fin;
mod flg;
mod hcp;
mod hlv;
mod iii;
mod isi;
mod ism;
mod jrr;
mod lap;
mod mal;
mod mci;
mod mode;
mod msl;
mod mso;
mod mst;
mod msx;
mod mtc;
mod nci;
mod ncn;
mod nlp;
mod npl;
mod obh;
mod oco;
mod pen;
mod pfl;
mod pit;
mod plc;
mod pll;
mod plp;
mod reo;
mod res;
mod rip;
mod rst;
mod scc;
mod sch;
mod sfp;
mod slc;
mod small;
mod spx;
mod ssh;
mod sta;
mod tiny;
mod toc;
mod ttc;
mod uco;
mod ver;
mod vtn;

pub use acr::{Acr, AcrResult};
pub use axi::Axi;
pub use axm::{Axm, ObjectInfo, PmoAction};
pub use axo::Axo;
pub use btn::{Bfn, BfnType, Btc, Btn, Btt};
pub use cch::{CameraView, Cch};
pub use cim::{Cim, CimMode};
pub use cnl::{Cnl, CnlReason};
pub use con::{Con, ConInfo};
pub use cpp::Cpp;
pub use cpr::Cpr;
pub use crs::Crs;
pub use csc::{Csc, CscAction};
pub use fin::{Fin, RaceResultFlags};
pub use flg::{Flg, FlgType};
pub use hcp::{Hcp, HcpCarHandicap};
pub use hlv::{Hlv, Hlvc};
pub use iii::Iii;
pub use isi::{Isi, IsiFlags};
pub use ism::Ism;
pub use jrr::{Jrr, JrrAction};
pub use lap::{Fuel, Fuel200, Lap};
pub use mal::Mal;
pub use mci::{CompCar, CompCarInfo, Mci};
pub use mode::Mod;
pub use msl::{Msl, SoundType};
pub use mso::{Mso, MsoUserType};
pub use mst::Mst;
pub use msx::Msx;
pub use mtc::Mtc;
pub use nci::Nci;
pub use ncn::Ncn;
pub use nlp::{Nlp, NodeLapInfo};
pub use npl::{Npl, PlayerFlags, TyreCompound};
pub use obh::{CarContact, Obh, ObhFlags};
pub use oco::{Oco, OcoAction, OcoIndex, OcoLights};
pub use pen::{Pen, PenaltyInfo, PenaltyReason};
pub use pfl::Pfl;
pub use pit::{Pit, PitLaneFact, Pla, Psf};
pub use plc::{Plc, PlcAllowedCars};
pub use pll::Pll;
pub use plp::Plp;
pub use reo::Reo;
pub use res::Res;
pub use rip::{Rip, RipError};
pub use rst::{HostFacts, Rst};
pub use scc::Scc;
pub use sch::{Sch, SchFlags};
pub use sfp::Sfp;
pub use slc::Slc;
pub use small::{Small, SmallType};
pub use spx::Spx;
pub use ssh::{Ssh, SshError};
pub use sta::{Sta, StaFlags, StaRacing};
pub use tiny::{Tiny, TinyType};
pub use toc::Toc;
pub use ttc::{Ttc, TtcType};
pub use uco::{Uco, UcoAction};
pub use ver::Version;
pub use vtn::{Vtn, VtnAction};

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

impl Frame for Packet {
    type Isi = Isi;

    fn maybe_pong(&self) -> Option<Self> {
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

    fn maybe_verify_version(&self) -> crate::result::Result<bool> {
        match self {
            Packet::Version(Version { insimver, .. }) => {
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
