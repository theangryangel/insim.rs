//! Insim v9 Packet definitions

use insim_core::{identifiers::RequestId, binrw::{self, binrw}};

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::insim::*;
use crate::relay;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, from_variants::FromVariants)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
#[repr(u8)]
/// Enum representing all possible packets receivable via an Insim connection
pub enum Packet {
    #[brw(magic = 1u8)] Init(Isi) = 1,
	#[brw(magic = 2u8)] Version(Version) = 2,
	#[brw(magic = 3u8)] Tiny(Tiny) = 3,
	#[brw(magic = 4u8)] Small(Small) = 4,
	#[brw(magic = 5u8)] State(Sta) = 5,
	#[brw(magic = 6u8)] SingleCharacter(Sch) = 6,
	#[brw(magic = 7u8)] StateFlagsPack(Sfp) = 7,
	#[brw(magic = 8u8)] SetCarCam(Scc) = 8,
	#[brw(magic = 9u8)] CamPosPack(Cpp) = 9,
	#[brw(magic = 10u8)] MultiPlayerNotification(Ism) = 10,
	#[brw(magic = 11u8)] MessageOut(Mso) = 11,
	#[brw(magic = 12u8)] InsimInfo(Iii) = 12,
	#[brw(magic = 13u8)] MessageType(Mst) = 13,
	#[brw(magic = 14u8)] MessageToConnection(Mtc) = 14,
	#[brw(magic = 15u8)] ScreenMode(Mod) = 15,
	#[brw(magic = 16u8)] VoteNotification(Vtn) = 16,
	#[brw(magic = 17u8)] RaceStart(Rst) = 17,
	#[brw(magic = 18u8)] NewConnection(Ncn) = 18,
	#[brw(magic = 19u8)] ConnectionLeave(Cnl) = 19,
	#[brw(magic = 20u8)] ConnectionPlayerRenamed(Cpr) = 20,
	#[brw(magic = 21u8)] NewPlayer(Npl) = 21,
	#[brw(magic = 22u8)] PlayerPits(Plp) = 22,
	#[brw(magic = 23u8)] PlayerLeave(Pll) = 23,
	#[brw(magic = 24u8)] Lap(Lap) = 24,
	#[brw(magic = 25u8)] SplitX(Spx) = 25,
	#[brw(magic = 26u8)] PitStopStart(Pit) = 26,
	#[brw(magic = 27u8)] PitStopFinish(Psf) = 27,
	#[brw(magic = 28u8)] PitLane(Pla) = 28,
	#[brw(magic = 29u8)] CameraChange(Cch) = 29,
	#[brw(magic = 30u8)] Penalty(Pen) = 30,
	#[brw(magic = 31u8)] TakeOverCar(Toc) = 31,
	#[brw(magic = 32u8)] Flag(Flg) = 32,
	#[brw(magic = 33u8)] PlayerFlags(Pfl) = 33,
	#[brw(magic = 34u8)] Finished(Fin) = 34,
	#[brw(magic = 35u8)] Result(Res) = 35,
	#[brw(magic = 36u8)] Reorder(Reo) = 36,
	#[brw(magic = 37u8)] NodeLap(Nlp) = 37,
	#[brw(magic = 38u8)] MultiCarInfo(Mci) = 38,
	#[brw(magic = 39u8)] MesssageExtended(Msx) = 39,
	#[brw(magic = 40u8)] MessageLocal(Msl) = 40,
	#[brw(magic = 41u8)] CarReset(Crs) = 41,
	#[brw(magic = 42u8)] ButtonFunction(Bfn) = 42,
	#[brw(magic = 43u8)] AutoXInfo(Axi) = 43,
	#[brw(magic = 44u8)] AutoXObject(Axo) = 44,
	#[brw(magic = 45u8)] Button(Btn) = 45,
	#[brw(magic = 46u8)] ButtonClick(Btc) = 46,
	#[brw(magic = 47u8)] ButtonType(Btt) = 47,
	#[brw(magic = 48u8)] ReplayInformation(Rip) = 48,
	#[brw(magic = 49u8)] ScreenShot(Ssh) = 49,
	#[brw(magic = 50u8)] Contact(Con) = 50,
	#[brw(magic = 51u8)] ObjectHit(Obh) = 51,
	#[brw(magic = 52u8)] HotLapValidity(Hlv) = 52,
	#[brw(magic = 53u8)] PlayerAllowedCars(Plc) = 53,
	#[brw(magic = 54u8)] AutoXMultipleObjects(Axm) = 54,
	#[brw(magic = 55u8)] AdminCommandReport(Acr) = 55,
	#[brw(magic = 56u8)] Handicaps(Hcp) = 56,
	#[brw(magic = 57u8)] Nci(Nci) = 57,
	#[brw(magic = 58u8)] Jrr(Jrr) = 58,
	#[brw(magic = 59u8)] UserControlObject(Uco) = 59,
	#[brw(magic = 60u8)] ObjectControl(Oco) = 60,
	#[brw(magic = 61u8)] TargetToConnection(Ttc) = 61,
	#[brw(magic = 62u8)] SelectedVehicle(Slc) = 62,
	#[brw(magic = 63u8)] VehicleStateChanged(Csc) = 63,
	#[brw(magic = 64u8)] ConnectionInterfaceMode(Cim) = 64,
	#[brw(magic = 65u8)] ModsAllowed(Mal) = 65,

    #[brw(magic = 250u8)] RelayAdminRequest(relay::AdminRequest) = 250,
    #[brw(magic = 251u8)] RelayAdminResponse(relay::AdminResponse) = 251,
    #[brw(magic = 252u8)] RelayHostListRequest(relay::HostListRequest) = 252,
    #[brw(magic = 253u8)] RelayHostList(relay::HostList) = 253,
    #[brw(magic = 254u8)] RelayHostSelect(relay::HostSelect) = 254,
    #[brw(magic = 255u8)] RelayError(relay::RelayError) = 255,
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
