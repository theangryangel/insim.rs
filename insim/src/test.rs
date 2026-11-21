//! Test helpers: a [`Session`] connected to a scripted LFS.
//!
//! The two ends talk over loopback TCP, so tests exercise the real socket path.
//! The helpers panic on failure, which is what a test wants.
//!
//! ```rust,no_run
//! # async fn example() {
//! use insim::insim::{Ism, TinyType};
//!
//! let (mut session, mut lfs) = insim::test::pair().await;
//! let mut ism = session.request::<Ism>(TinyType::Ism).unwrap();
//! session.flush().await.unwrap();
//! let request = lfs.recv().await.unwrap();
//! lfs.send(Ism { reqi: request.request_id().unwrap(), ..Default::default() }).await;
//! # }
//! ```

use bytes::BytesMut;
use insim_core::net::{Decoder, Encoder};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{
    Packet,
    insim::Isi,
    net::{InsimCodec, Response, Session, Tokio, Unstarted},
};

/// A started session and the fake LFS it is connected to.
///
/// The session is resumed, not handshaken; use [`pair_unstarted`] to test the
/// handshake.
pub async fn pair() -> (Session<Tokio>, Lfs) {
    let (unstarted, lfs) = pair_unstarted().await;
    (unstarted.resume(), lfs)
}

/// A session that has not started InSim yet, and the fake LFS it is connected to.
pub async fn pair_unstarted() -> (Unstarted<Tokio>, Lfs) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind loopback listener");
    let address = listener.local_addr().expect("listener address");
    let (stream, accepted) = tokio::join!(TcpStream::connect(address), listener.accept());
    let session = Session::tcp(stream.expect("connect to fake LFS")).expect("session");
    let (stream, _) = accepted.expect("accept session");
    (
        session,
        Lfs {
            stream,
            input: BytesMut::new(),
        },
    )
}

/// The LFS end of a [`pair`]. It sends and receives packets exactly as given and
/// does nothing automatically, including keepalives.
#[derive(Debug)]
pub struct Lfs {
    stream: TcpStream,
    input: BytesMut,
}
impl Lfs {
    /// Send a packet to the session.
    pub async fn send(&mut self, packet: impl Into<Packet>) {
        let mut bytes = BytesMut::new();
        InsimCodec
            .encode(&packet.into(), &mut bytes)
            .expect("encode packet");
        self.stream
            .write_all(&bytes)
            .await
            .expect("write to session");
    }
    /// The next packet from the session, or `None` once it has closed the connection.
    pub async fn recv(&mut self) -> Option<Packet> {
        loop {
            if let Some(packet) = InsimCodec.decode(&mut self.input).expect("decode packet") {
                return Some(packet);
            }
            let mut buf = [0; 1024];
            match self.stream.read(&mut buf).await {
                Ok(0) => return None,
                Ok(n) => self.input.extend_from_slice(&buf[..n]),
                Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => return None,
                Err(error) => panic!("read from session: {error}"),
            }
        }
    }
    /// The next packet, which must be a `T`.
    ///
    /// # Panics
    /// If the session closes or sends anything else first.
    pub async fn expect<T: Response + std::fmt::Debug>(&mut self) -> T {
        let packet = self.recv().await.expect("session closed");
        let description = format!("{packet:?}");
        T::from_packet(packet)
            .unwrap_or_else(|| panic!("expected {}, got {description}", std::any::type_name::<T>()))
    }
    /// Answer an [`Isi`] with a matching [`Ver`](crate::insim::Ver), completing a
    /// handshake started on a [`pair_unstarted`] session. Returns the Isi.
    pub async fn accept_handshake(&mut self) -> Isi {
        let isi: Isi = self.expect().await;
        self.send(crate::insim::Ver {
            reqi: isi.reqi,
            insimver: crate::VERSION,
            ..Default::default()
        })
        .await;
        isi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        identifiers::RequestId,
        insim::{Tiny, TinyType, Ver},
    };

    #[tokio::test]
    async fn pair_carries_packets_both_ways_and_reports_close() {
        let (mut session, mut lfs) = pair().await;
        session.write(TinyType::Ping).await.unwrap();
        let ping: Tiny = lfs.expect().await;
        assert_eq!(ping.subt, TinyType::Ping);
        lfs.send(Tiny {
            reqi: RequestId(3),
            subt: TinyType::Reply,
        })
        .await;
        assert!(matches!(session.read().await.unwrap(), Packet::Tiny(_)));
        drop(session);
        assert!(lfs.recv().await.is_none());
    }

    #[tokio::test]
    async fn accept_handshake_completes_an_unstarted_session() {
        let (unstarted, mut lfs) = pair_unstarted().await;
        let isi = Isi {
            reqi: RequestId(9),
            ..Default::default()
        };
        let (session, isi) = tokio::join!(unstarted.handshake(isi), lfs.accept_handshake());
        assert_eq!(isi.reqi, RequestId(9));
        let mut session = session.unwrap();
        assert!(matches!(
            session.read().await.unwrap(),
            Packet::Ver(Ver {
                reqi: RequestId(9),
                ..
            })
        ));
    }
}
