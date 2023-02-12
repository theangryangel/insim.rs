use bytes::{BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, PartialEq, Eq, Debug)]
#[allow(unused)]
#[insim(magic = b"TEST")]
#[repr(u8)]
enum TestEnumMagic {
    U8(u8) = 1,
    U32(u32) = 2,
    I8(i8) = 4,
}

impl Default for TestEnumMagic {
    fn default() -> Self {
        Self::U8(100)
    }
}

#[test]
fn test_ser_enum_magic_encode() {
    let mut buf = BytesMut::new();

    let i = TestEnumMagic::I8(-1);
    i.encode(&mut buf, None)
        .expect("Expected encoding to succeed");

    let mut comparison = BytesMut::new();
    comparison.put_slice("TEST".as_bytes());
    comparison.put_u8(4);
    comparison.put_i8(-1);

    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_ser_enum_magic_decode() {
    let mut comparison = BytesMut::new();
    comparison.put_slice("TEST".as_bytes());
    comparison.put_u8(4);
    comparison.put_i8(-1);

    let i =
        TestEnumMagic::decode(&mut comparison, None).expect("Expected decode of enum to succeed");

    assert_eq!(TestEnumMagic::I8(-1), i);
}
