use crate::{
    codec::{Codec, Mode},
    packets,
};
use bytes::BytesMut;
use insim_core::identifiers::RequestId;
use tokio_test::assert_ok;

#[tokio::test]
/// Ensure that Codec can decode a basic small packet
async fn read_tiny_ping() {
    let mut mock = BytesMut::new();
    mock.extend_from_slice(
        // Packet::Tiny, subtype TinyType::Ping, compressed, reqi=2
        &[1, 3, 2, 3],
    );

    let mut codec = Codec {
        mode: Mode::Compressed,
    };
    let data = codec.decode(&mut mock);
    assert_ok!(&data);
    let data = data.unwrap();

    assert!(matches!(
        data,
        Some(packets::Packet::Tiny(packets::insim::Tiny {
            subt: packets::insim::TinyType::Ping,
            reqi: RequestId(2)
        }))
    ));
}

#[tokio::test]
/// Ensure that Codec can write a basic small packet
async fn write_tiny_ping() {
    let mut mock = BytesMut::new();
    mock.extend_from_slice(
        // Packet::Tiny, subtype TinyType::Ping, compressed, reqi=2
        &[1, 3, 2, 3],
    );

    let mut buf = BytesMut::new();

    let mut codec = Codec {
        mode: Mode::Compressed,
    };
    let res = codec.encode(
        packets::insim::Tiny {
            subt: packets::insim::TinyType::Ping,
            reqi: RequestId(2),
        }
        .into(),
        &mut buf,
    );
    assert_ok!(res);

    assert_eq!(&mock[..], &buf[..])
}
