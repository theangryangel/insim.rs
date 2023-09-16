use tokio::net::{TcpStream, UdpSocket};

use crate::{
    result::Result,
    codec::Codec, network::{Framed, websocket::TungsteniteWebSocket},
};

// The "Inner" connection for Connection, so that we can avoid Box'ing
// Since the ConnectionOptions is all very hard coded, for "high level" API usage,
// I think this fine.
// i.e. if we add a Websocket option down the line, then ConnectionOptions needs to understand it
// therefore we cannot just box stuff magically anyway.
pub(crate) enum ConnectionInner<C: Codec> {
    Tcp(Framed<C, TcpStream>),
    Udp(Framed<C, UdpSocket>),
    WebSocket(Framed<C, TungsteniteWebSocket>),
}

impl<C: Codec> ConnectionInner<C> {
    pub async fn read(&mut self) -> Result<C::Item> {
        match self {
            Self::Tcp(i) => i.read().await,
            Self::Udp(i) => i.read().await,
            Self::WebSocket(i) => i.read().await,
        }
    }

    pub async fn write<P: Into<C::Item>>(&mut self, packet: P) -> Result<()> {
        match self {
            Self::Tcp(i) => i.write(packet).await,
            Self::Udp(i) => i.write(packet).await,
            Self::WebSocket(i) => i.write(packet).await,
        }
    }
}
