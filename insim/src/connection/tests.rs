use crate::{
    codec::{Codec, Mode},
    connection::Connection,
    packets,
};
use futures::StreamExt;
use insim_core::identifiers::RequestId;
use tokio_test::io::Builder;
use tokio_util::codec::Framed;

#[tokio::test]
/// Ensure that when a TinyType::None is received, the client automatically responds with a
/// TinyType::None
async fn poll_pong_compressed() {
    let mock = Builder::new()
        .read(
            // Packet::Tiny, subtype TinyType::None, compressed
            &[1, 3, 0, 0],
        )
        .write(
            // Packet::Tiny, subtype: TinyType::None, compressed
            &[1, 3, 0, 0],
        )
        .build();

    let framed = Framed::new(mock, Codec::new(Mode::Compressed));

    let mut t = Connection::new(framed);

    let data = t.next().await;
    assert!(matches!(
        data,
        Some(Ok(packets::Packet::Tiny(packets::insim::Tiny {
            subtype: packets::insim::TinyType::None,
            reqi: RequestId(0)
        })))
    ));
}

#[tokio::test]
/// Ensure that when a TinyType::None is received, the client automatically responds with a
/// TinyType::None
async fn poll_pong_uncompressed() {
    let mock = Builder::new()
        .read(
            // Packet::Tiny, subtype TinyType::None, uncompressed
            &[4, 3, 0, 0],
        )
        .write(
            // Packet::Tiny, subtype: TinyType::None, uncompressed
            &[4, 3, 0, 0],
        )
        .build();

    let framed = Framed::new(mock, Codec::new(Mode::Uncompressed));

    let mut t = Connection::new(framed);

    let data = t.next().await;
    assert!(matches!(
        data,
        Some(Ok(packets::Packet::Tiny(packets::insim::Tiny {
            subtype: packets::insim::TinyType::None,
            reqi: RequestId(0)
        })))
    ));
}
