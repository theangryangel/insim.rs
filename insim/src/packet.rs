//! Insim v9 Packet definitions

use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
};

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
/// Enum representing all possible packets receivable via an Insim connection
pub enum Packet {
    #[brw(magic = 1u8)]
    Init(Isi),
    #[brw(magic = 2u8)]
    Version(Version),
    #[brw(magic = 3u8)]
    Tiny(Tiny),
    #[brw(magic = 4u8)]
    Small(Small),
    #[brw(magic = 5u8)]
    State(Sta),
    #[brw(magic = 6u8)]
    SingleCharacter(Sch),
    #[brw(magic = 7u8)]
    StateFlagsPack(Sfp),
    #[brw(magic = 8u8)]
    SetCarCam(Scc),
    #[brw(magic = 9u8)]
    CamPosPack(Cpp),
    #[brw(magic = 10u8)]
    MultiPlayerNotification(Ism),
    #[brw(magic = 11u8)]
    MessageOut(Mso),
    #[brw(magic = 12u8)]
    InsimInfo(Iii),
    #[brw(magic = 13u8)]
    MessageType(Mst),
    #[brw(magic = 14u8)]
    MessageToConnection(Mtc),
    #[brw(magic = 15u8)]
    ScreenMode(Mod),
    #[brw(magic = 16u8)]
    VoteNotification(Vtn),
    #[brw(magic = 17u8)]
    RaceStart(Rst),
    #[brw(magic = 18u8)]
    NewConnection(Ncn),
    #[brw(magic = 19u8)]
    ConnectionLeave(Cnl),
    #[brw(magic = 20u8)]
    ConnectionPlayerRenamed(Cpr),
    #[brw(magic = 21u8)]
    NewPlayer(Npl),
    #[brw(magic = 22u8)]
    PlayerPits(Plp),
    #[brw(magic = 23u8)]
    PlayerLeave(Pll),
    #[brw(magic = 24u8)]
    Lap(Lap),
    #[brw(magic = 25u8)]
    SplitX(Spx),
    #[brw(magic = 26u8)]
    PitStopStart(Pit),
    #[brw(magic = 27u8)]
    PitStopFinish(Psf),
    #[brw(magic = 28u8)]
    PitLane(Pla),
    #[brw(magic = 29u8)]
    CameraChange(Cch),
    #[brw(magic = 30u8)]
    Penalty(Pen),
    #[brw(magic = 31u8)]
    TakeOverCar(Toc),
    #[brw(magic = 32u8)]
    Flag(Flg),
    #[brw(magic = 33u8)]
    PlayerFlags(Pfl),
    #[brw(magic = 34u8)]
    Finished(Fin),
    #[brw(magic = 35u8)]
    Result(Res),
    #[brw(magic = 36u8)]
    Reorder(Reo),
    #[brw(magic = 37u8)]
    NodeLap(Nlp),
    #[brw(magic = 38u8)]
    MultiCarInfo(Mci),
    #[brw(magic = 39u8)]
    MesssageExtended(Msx),
    #[brw(magic = 40u8)]
    MessageLocal(Msl),
    #[brw(magic = 41u8)]
    CarReset(Crs),
    #[brw(magic = 42u8)]
    ButtonFunction(Bfn),
    #[brw(magic = 43u8)]
    AutoXInfo(Axi),
    #[brw(magic = 44u8)]
    AutoXObject(Axo),
    #[brw(magic = 45u8)]
    Button(Btn),
    #[brw(magic = 46u8)]
    ButtonClick(Btc),
    #[brw(magic = 47u8)]
    ButtonType(Btt),
    #[brw(magic = 48u8)]
    ReplayInformation(Rip),
    #[brw(magic = 49u8)]
    ScreenShot(Ssh),
    #[brw(magic = 50u8)]
    Contact(Con),
    #[brw(magic = 51u8)]
    ObjectHit(Obh),
    #[brw(magic = 52u8)]
    HotLapValidity(Hlv),
    #[brw(magic = 53u8)]
    PlayerAllowedCars(Plc),
    #[brw(magic = 54u8)]
    AutoXMultipleObjects(Axm),
    #[brw(magic = 55u8)]
    AdminCommandReport(Acr),
    #[brw(magic = 56u8)]
    Handicaps(Hcp),
    #[brw(magic = 57u8)]
    Nci(Nci),
    #[brw(magic = 58u8)]
    Jrr(Jrr),
    #[brw(magic = 59u8)]
    UserControlObject(Uco),
    #[brw(magic = 60u8)]
    ObjectControl(Oco),
    #[brw(magic = 61u8)]
    TargetToConnection(Ttc),
    #[brw(magic = 62u8)]
    SelectedVehicle(Slc),
    #[brw(magic = 63u8)]
    VehicleStateChanged(Csc),
    #[brw(magic = 64u8)]
    ConnectionInterfaceMode(Cim),
    #[brw(magic = 65u8)]
    ModsAllowed(Mal),

    #[brw(magic = 250u8)]
    RelayAdminRequest(relay::AdminRequest),
    #[brw(magic = 251u8)]
    RelayAdminResponse(relay::AdminResponse),
    #[brw(magic = 252u8)]
    RelayHostListRequest(relay::HostListRequest),
    #[brw(magic = 253u8)]
    RelayHostList(relay::HostList),
    #[brw(magic = 254u8)]
    RelayHostSelect(relay::HostSelect),
    #[brw(magic = 255u8)]
    RelayError(relay::RelayError),
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
