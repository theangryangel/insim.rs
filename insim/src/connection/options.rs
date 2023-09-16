use std::{net::SocketAddr, time::Duration};

use insim_core::identifiers::RequestId;
use tokio::{
    net::{TcpStream, UdpSocket},
    time::timeout,
};

use crate::{
    codec::{Mode, Codec},
    result::Result, network::{Framed, Network},
};

use super::network_options::NetworkOptions;

#[derive(Clone, Default)]
pub struct ConnectionOptions {
    pub network_options: NetworkOptions,
    pub connection_timeout: Option<Duration>,
}

impl ConnectionOptions {
}
