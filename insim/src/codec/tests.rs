use crate::{
    codec::{Codec, Mode},
    insim::{Tiny, TinyType},
    packet::Packet,
};
use bytes::{Buf, BytesMut};
use insim_core::{identifiers::RequestId, InsimDecode, InsimEncode};
use tokio_test::assert_ok;

#[tokio::test]
/// Ensure that Codec can decode a basic small packet
async fn read_tiny_ping() {
    let mut mock = BytesMut::new();
    mock.extend_from_slice(
        // Packet::Tiny, subtype TinyType::Ping, compressed, reqi=2
        &[1, 3, 2, 3],
    );

    let codec = Codec::new(Mode::Compressed);
    let data = codec.decode(&mut mock);
    assert_ok!(&data);
    let data = data.unwrap();

    assert!(matches!(
        data,
        Some(Packet::Tiny(Tiny {
            reqi: RequestId(2),
            subt: TinyType::Ping,
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

    let codec = Codec::new(Mode::Compressed);
    let res = codec.encode(
        &Packet::Tiny(Tiny {
            subt: TinyType::Ping,
            reqi: RequestId(2),
        }),
        &mut buf,
    );
    assert_ok!(res);

    assert_eq!(&mock[..], &buf[..])
}
