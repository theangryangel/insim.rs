use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

use crate::codec::Mode;
use crate::packets::Packet;
use crate::connection::builder::ConnectionBuilder as Config;
use crate::packets::insim::{IsiFlags, Isi};

use super::traits::{ReadWritePacket, ReadPacket, WritePacket};

#[derive(Clone)]
pub enum Transport {
    Tcp { 
        remote: SocketAddr,
        codec_mode: Mode,
        verify_version: bool,
        wait_for_initial_pong: bool,
    },
    Udp { 
        local: Option<SocketAddr>, 
        remote: SocketAddr,
        codec_mode: Mode,
        verify_version: bool,
        wait_for_initial_pong: bool,
    },
    Relay {
        select_host: Option<String>,
        connect_timeout: Duration,
    }
}

impl Transport {
    pub async fn connect(&self, isi: Isi) -> crate::result::Result<Box<dyn ReadWritePacket>> {
        match self {
            Transport::Tcp { remote, codec_mode, verify_version, wait_for_initial_pong } => {
                let stream = timeout(Duration::from_secs(super::TIMEOUT_SECS), TcpStream::connect(remote)).await??;

                let mut stream = super::tcp::Tcp::new(stream, *codec_mode);

                super::handshake(
                    &mut stream,
                    Duration::from_secs(super::TIMEOUT_SECS),
                    isi,
                    *wait_for_initial_pong,
                    *verify_version,
                )
                .await?;

                Ok(stream.boxed())
            },
            Transport::Udp { local, remote, codec_mode, verify_version, wait_for_initial_pong } => {
                let local = local.unwrap_or("0.0.0.0:0".parse()?);

                let stream = UdpSocket::bind(local).await?;
                stream.connect(remote).await.unwrap();
                let mut isi = isi.clone();
                isi.udpport = stream.local_addr().unwrap().port().into();
                let mut stream = crate::connection::udp::Udp::new(stream, *codec_mode);

                super::handshake(
                    &mut stream,
                    Duration::from_secs(super::TIMEOUT_SECS),
                    isi,
                    *wait_for_initial_pong,
                    *verify_version,
                )
                .await?;

                Ok(stream.boxed())
            },
            Transport::Relay { select_host, connect_timeout } => {
                use crate::packets::relay::HostSelect;

                // Why not call connect_tcp? purely so we won't call request_game_state multiple times
                // until we select a host there is no game state to request
                let stream = timeout(
                    *connect_timeout,
                    TcpStream::connect("isrelay.lfs.net:47474"),
                )
                .await??;

                let mut stream = super::tcp::Tcp::new(
                    stream, 
                    // TODO: Talk to LFS devs, find out if/when relay gets compressed support?
                    Mode::Uncompressed
                );
                
                super::handshake(
                    &mut stream,
                    Duration::from_secs(super::TIMEOUT_SECS),
                    isi,
                    false,  // Relay does not respond to ping requests
                    false, // Relay does not respond to version requests until after the host is selected
                )
                .await?;

                if let Some(hostname) = select_host {
                    stream
                        .write(
                            HostSelect {
                                hname: hostname.to_string(),
                                ..Default::default()
                            }
                            .into(),
                        )
                        .await?;
                }

                Ok(stream.boxed())
            }
        }

    }
    
}


#[derive(Clone)]
pub struct ConnectionOptions {
    pub name: String,
    pub password: String,
    pub flags: IsiFlags,
    pub prefix: Option<char>,
    pub interval: Duration,

    pub transport: Transport,
}

impl ConnectionOptions{

    pub fn as_isi(&self) -> Isi {

        Isi::default()
        
    }
    
}


// Connection represents the public facing API
pub struct Connection {}

impl Connection {

    pub fn spawn(config: ConnectionOptions) -> Self {
        let mut actor = Actor {
            config: config.clone(),
        };

        let handle = tokio::spawn(async move { run_actor(actor).await });

        Self {}
    }

    pub async fn shutdown(&self) {
        unimplemented!()
    }

    pub async fn send<T: Into<Packet>>(&self, packet: T) {
        unimplemented!()
    }
}


struct Actor {
    config: ConnectionOptions,
}

async fn run_actor(mut actor: Actor) {

    loop {

        // connect
        let isi = actor.config.as_isi();
        let mut stream: Box<dyn ReadWritePacket> = actor.config.transport.connect(isi).await.unwrap();

        loop {
            let res = timeout(
                Duration::from_secs(super::TIMEOUT_SECS), stream.read()
            ).await;

            if res.is_err() {
                // Timeout
                break;
            }

            match res.unwrap() {
                Ok(Some(packet)) => {
                    super::maybe_keepalive(&mut stream, &packet).await;

                },
                Ok(None) => {
                    break;  
                },
                Err(e) => {
                    break;
                }

            }

        }
    }
    
}
