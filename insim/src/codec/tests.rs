use crate::codec::{Codec, Mode};
use bytes::{Buf, BytesMut};
use insim_core::{identifiers::RequestId, InsimDecode, InsimEncode};
use tokio_test::assert_ok;

use super::VersionedFrame;

#[derive(Debug, Default, Clone, InsimEncode, InsimDecode)]
#[repr(u8)]
enum TestTinyType {
    #[default]
    None = 0,
    One = 1,
    Two = 2,
    Ping = 3,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
/// General purpose Tiny packet
pub struct TestTiny {
    reqi: RequestId,
    subt: TestTinyType,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[repr(u8)]
enum TestPacket {
    Tiny(TestTiny) = 3,
}

impl From<()> for TestPacket {
    fn from(_value: ()) -> Self {
        TestPacket::Tiny(TestTiny::default())
    }
}

impl VersionedFrame for TestPacket {
    type Init = ();

    fn is_ping(&self) -> bool {
        false
    }

    fn pong(_reqi: Option<RequestId>) -> Self {
        Self::Tiny(TestTiny::default())
    }

    fn maybe_verify_version(&self) -> crate::result::Result<bool> {
        Ok(true)
    }
}

#[tokio::test]
/// Ensure that Codec can decode a basic small packet
async fn read_tiny_ping() {
    let mut mock = BytesMut::new();
    mock.extend_from_slice(
        // Packet::Tiny, subtype TinyType::Ping, compressed, reqi=2
        &[1, 3, 2, 3],
    );

    let codec: Codec<TestPacket> = Codec::new(Mode::Compressed);
    let data = codec.decode(&mut mock);
    assert_ok!(&data);
    let data = data.unwrap();

    assert!(matches!(
        data,
        Some(TestPacket::Tiny(TestTiny {
            reqi: RequestId(2),
            subt: TestTinyType::Ping,
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
        &TestPacket::Tiny(TestTiny {
            subt: TestTinyType::Ping,
            reqi: RequestId(2),
        }),
        &mut buf,
    );
    assert_ok!(res);

    assert_eq!(&mock[..], &buf[..])
}
