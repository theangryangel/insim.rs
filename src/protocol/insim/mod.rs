//! Definitions for Insim Game Server packets.

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
mod init;
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
mod state;
mod tiny;
mod toc;
mod ttc;
mod uco;
mod ver;
mod vtn;

pub const VERSION: u8 = 8;

pub use acr::{Acr, AcrResult};
pub use axi::Axi;
pub use axm::{Axm, ObjectInfo, PmoAction};
pub use axo::Axo;
pub use btn::{Bfn, BfnType, Btc, Btn, Btt};
pub use cch::Cch;
pub use cim::{Cim, CimMode};
pub use cnl::{Cnl, CnlReason};
pub use con::{Con, ConInfo};
pub use cpp::Cpp;
pub use cpr::Cpr;
pub use crs::Crs;
pub use csc::{Csc, CscAction};
pub use fin::Fin;
pub use flg::{Flg, FlgType};
pub use hcp::{Hcp, HcpCarHandicap};
pub use hlv::{Hlv, Hlvc};
pub use iii::Iii;
pub use init::{Init, InitFlags};
pub use ism::Ism;
pub use jrr::{Jrr, JrrAction};
pub use lap::Lap;
pub use mal::Mal;
pub use mci::Mci;
pub use mode::Mode;
pub use msl::{Msl, MslSoundType};
pub use mso::{Mso, MsoUserType};
pub use mst::Mst;
pub use msx::Msx;
pub use mtc::Mtc;
pub use nci::Nci;
pub use ncn::Ncn;
pub use nlp::{Nlp, NodeLapInfo};
pub use npl::{Npl, PlayerFlags};
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
pub use rst::Rst;
pub use scc::Scc;
pub use sch::Sch;
pub use sfp::Sfp;
pub use slc::Slc;
pub use small::{Small, SmallType};
pub use spx::Spx;
pub use ssh::{Ssh, SshError};
pub use state::Sta;
pub use tiny::{Tiny, TinyType};
pub use toc::Toc;
pub use ttc::{Ttc, TtcType};
pub use uco::{Uco, UcoAction};
pub use ver::Version;
pub use vtn::Vtn;
