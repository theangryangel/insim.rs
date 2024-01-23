//! Definitions for the Insim Relay packets.
//! The InSim Relay is a service that can connect to your LFS host via Insim and relay the InSim
//! information sent by your host, to anyone who connects to the Insim Relay.
//!
//! This relayed data can be used by programmers for various things, such as the LFS Remote
//! (remote viewing / adminning of a race) and race-tracking to store race information and
//! statistics.
//!
//! See [https://en.lfsmanual.net/wiki/InSim_Relay](https://en.lfsmanual.net/wiki/InSim_Relay) for more information.

mod admin_request;
mod admin_response;
mod error;
mod host_list;
mod host_list_request;
mod host_select;

pub use admin_request::Arq;
pub use admin_response::Arp;
pub use error::{Error, RelayErrorKind};
pub use host_list::{Hos, HostInfo, HostInfoFlags};
pub use host_list_request::Hlr;
pub use host_select::Sel;
