//! A lower level API to working with Insim.
//!
//! # Example
//! ```rust
//! // Connect to the Insim Relay
//! let tcp: TcpStream = TcpStream::connect("isrelay.lfs.net:47474").await.unwrap();
//!
//! // Create a Transport, using the uncompressed packet length mode (insim protocol <= 8 uses this,
//! // insim >= 9 supports both compressed and uncompressed modes).
//! let mut t = insim::protocol::transport::Transport::new(
//!     tcp,
//!     insim::protocol::codec::Mode::Uncompressed
//! );
//!
//! // Send a Init packet to handshake with the server.
//! let isi = insim::protocol::insim::Init {
//!     name: "insim.rs".into(),
//!     password: "".into(),
//!     prefix: b'!',
//!     version: insim::protocol::VERSION,
//!     interval: 1000,
//!     flags: insim::protocol::insim::InitFlags::MCI,
//!     reqi: 1,
//! };
//!
//! t.send(isi).await;
//!
//! // Select a host from the relay to receive data from.
//! t.send(insim::protocol::relay::HostSelect {
//!     hname: "Nubbins AU Demo".into(),
//!     ..Default::default()
//! }).await;
//!
//! // Print the data from the relay.
//! while let Some(m) = t.next().await {
//!     tracing::debug!("{:?}", m);
//! }
//! ````

use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

pub mod codec;
pub mod insim;
mod macros;
pub mod position;
pub mod relay;
pub mod transport;

pub const VERSION: u8 = 9;

use crate::packet;

packet!(
    Packet,
    "1" => Init(insim::Init),
    "2" => Version(insim::Version),
    "3" => Tiny(insim::Tiny),
    "4" => Small(insim::Small),
    "5" => State(insim::Sta),
    "6" => SingleCharacter(insim::Sch),
    "7" => StateFlagsPack(insim::Sfp),
    "8" => SetCarCam(insim::Scc),
    "9" => CamPosPack(insim::Cpp),
    "10" => MultiPlayerNotification(insim::Ism),
    "11" => MessageOut(insim::Mso),
    "12" => InsimInfo(insim::Iii),
    "13" => MessageType(insim::Mst),
    "14" => MessageToConnection(insim::Mtc),
    "15" => ScreenMode(insim::Mode),
    "16" => VoteNotification(insim::Vtn),
    "17" => RaceStart(insim::Rst),
    "18" => NewConnection(insim::Ncn),
    "19" => ConnectionLeave(insim::Cnl),
    "20" => ConnectionPlayerRenamed(insim::Cpr),
    "21" => NewPlayer(insim::Npl),
    "22" => PlayerPits(insim::Plp),
    "23" => PlayerLeave(insim::Pll),
    "24" => Lap(insim::Lap),
    "25" => SplitX(insim::Spx),
    "26" => PitStopStart(insim::Pit),
    "27" => PitStopFinish(insim::Psf),
    "28" => PitLane(insim::Pla),
    "29" => CameraChange(insim::Cch),
    "30" => Penalty(insim::Pen),
    "31" => TakeOverCar(insim::Toc),
    "32" => Flag(insim::Flg),
    "33" => PlayerFlags(insim::Pfl),
    "34" => Finished(insim::Fin),
    "35" => Result(insim::Res),
    "36" => Reorder(insim::Reo),
    "37" => NodeLap(insim::Nlp),
    "38" => MultiCarInfo(insim::Mci),
    "39" => MesssageExtended(insim::Msx),
    "40" => MessageLocal(insim::Msl),
    "41" => CarReset(insim::Crs),
    "42" => ButtonFunction(insim::Bfn),
    "43" => AutoXInfo(insim::Axi),
    "44" => AutoXObject(insim::Axo),
    "45" => Button(insim::Btn),
    "46" => ButtonClick(insim::Btc),
    "47" => ButtonType(insim::Btt),
    "48" => ReplayInformation(insim::Rip),
    "49" => ScreenShot(insim::Ssh),
    "50" => Contact(insim::Con),
    "51" => ObjectHit(insim::Obh),
    "52" => HotLapValidity(insim::Hlv),
    "53" => PlayerAllowedCars(insim::Plc),
    "54" => AutoXMultipleObjects(insim::Axm),
    "55" => AdminCommandReport(insim::Acr),
    "56" => Handicaps(insim::Hcp),
    "57" => Nci(insim::Nci),
    "58" => Jrr(insim::Jrr),
    "59" => UserControlObject(insim::Uco),
    "60" => ObjectControl(insim::Oco),
    "61" => TargetToConnection(insim::Ttc),
    "62" => SelectedVehicle(insim::Slc),
    "63" => VehicleStateChanged(insim::Csc),
    "64" => ConnectionInterfaceMode(insim::Cim),
    "65" => ModsAllowed(insim::Mal),

    "250" => RelayAdminRequest(relay::AdminRequest),
    "251" => RelayAdminResponse(relay::AdminResponse),
    "252" => RelayHostListRequest(relay::HostListRequest),
    "253" => RelayHostList(relay::HostList),
    "254" => RelayHostSelect(relay::HostSelect),
    "255" => RelayError(relay::Error),
);
