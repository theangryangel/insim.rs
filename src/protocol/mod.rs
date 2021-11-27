use deku::prelude::*;
use serde::Serialize; // TODO make serde support an optional feature

pub mod codec;
pub mod insim;
mod macros;
pub mod relay;
pub mod transport;

use crate::generate_insim_packet;

generate_insim_packet!(
    Packet,
    Init => insim::Init, "1",
    Version => insim::Version, "2",
    Tiny => insim::Tiny, "3",
    Small => insim::Small, "4",
    State => insim::Sta, "5",
    Sch => insim::Sch, "6",
    MessageOut => insim::MessageOut, "11",
    Ncn => insim::Ncn, "18",
    Cnl => insim::Cnl, "19",
    Npl => insim::Npl, "21",
    Plp => insim::Plp, "22",
    Pll => insim::Pll, "23",
    Lap => insim::Lap, "24",
    SplitX => insim::SplitX, "25",
    Flg => insim::Flg, "32",
    MultiCarInfo => insim::MultiCarInfo, "38",

    RelayAdminRequest => relay::AdminRequest, "250",
    RelayAdminResponse => relay::AdminResponse, "251",
    RelayHostListRequest => relay::HostListRequest, "252",
    RelayHostList => relay::HostList, "253",
    RelayHostSelect => relay::HostSelect, "254",
    RelayError => relay::Error, "255",
);
