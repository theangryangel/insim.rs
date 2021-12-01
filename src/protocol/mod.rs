use deku::prelude::*;
use serde::Serialize; // TODO make serde support an optional feature

pub mod codec;
pub mod insim;
mod macros;
pub mod position;
pub mod relay;
pub mod transport;

use crate::packet;

packet!(
    Packet,
    "1" => Init(insim::Init),
    "2" => Version(insim::Version),
    "3" => Tiny(insim::Tiny),
    "4" => Small(insim::Small),
    "5" => State(insim::Sta),
    "6" => Sch(insim::Sch),
    "7" => Sfp(insim::Sfp),
    "8" => Scc(insim::Scc),
    "9" => Cpp(insim::Cpp),
    "10" => Ism(insim::Ism),
    "11" => MessageOut(insim::MessageOut),
    "12" => Iii(insim::Iii),
    "13" => Mst(insim::Mst),
    "14" => Mtc(insim::Mtc),
    "15" => ScreenMode(insim::ScreenMode),
    "16" => VoteNotification(insim::VoteNotification),
    "17" => RaceStart(insim::RaceStart),
    "18" => Ncn(insim::Ncn),
    "19" => Cnl(insim::Cnl),
    "20" => PlayerRenamed(insim::PlayerRenamed),
    "21" => Npl(insim::Npl),
    "22" => Plp(insim::Plp),
    "23" => Pll(insim::Pll),
    "24" => Lap(insim::Lap),
    "25" => SplitX(insim::SplitX),
    "26" => PitStopStart(insim::PitStopStart),
    "27" => PitStopFinish(insim::PitStopFinish),
    "28" => PitLane(insim::PitLane),
    "29" => CameraChange(insim::CameraChange),
    "30" => Penalty(insim::Penalty),
    "31" => TakeOverCar(insim::TakeOverCar),
    "32" => Flg(insim::Flg),
    "33" => PlayerFlags(insim::PlayerFlags),
    "34" => Fin(insim::Fin),
    "35" => Res(insim::Res),
    "36" => Reorder(insim::Reorder),
    "37" => NodeLap(insim::NodeLap),
    "38" => MultiCarInfo(insim::MultiCarInfo),
    "39" => Msx(insim::Msx),
    "40" => Msl(insim::Msl),
    "50" => Con(insim::Con),

    "250" => RelayAdminRequest(relay::AdminRequest),
    "251" => RelayAdminResponse(relay::AdminResponse),
    "252" => RelayHostListRequest(relay::HostListRequest),
    "253" => RelayHostList(relay::HostList),
    "254" => RelayHostSelect(relay::HostSelect),
    "255" => RelayError(relay::Error),
);
