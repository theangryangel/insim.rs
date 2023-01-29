use bytes::{BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, PartialEq, Eq, Debug)]
#[repr(u8)]
#[allow(unused)]
enum TestEnumNewType {
    U8(u8) = 1,
    U32(u32) = 2,
    I8(i8) = 4,
}

impl Default for TestEnumNewType {
    fn default() -> Self {
        Self::U8(100)
    }
}

#[test]
fn test_ser_enum_newtype_encode() {
    let mut buf = BytesMut::new();

    let i = TestEnumNewType::I8(-1);
    i.encode(&mut buf, None)
        .expect("Expected encoding to succeed");

    let mut comparison = BytesMut::new();
    comparison.put_u8(4);
    comparison.put_i8(-1);

    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_ser_enum_newtype_decode() {
    let mut comparison = BytesMut::new();
    comparison.put_u8(4);
    comparison.put_i8(-1);

    let i =
        TestEnumNewType::decode(&mut comparison, None).expect("Expected decode of enum to succeed");

    assert_eq!(TestEnumNewType::I8(-1), i);
}

#[derive(InsimEncode, InsimDecode, PartialEq, Eq, Debug)]
#[repr(u8)]
#[allow(unused)]
enum TestEnumUnit {
    Unit = 1,
    NewType(u32) = 2,
    Tuple(u32, u32) = 3,
    Struct { a: u32 } = 4,
}

impl Default for TestEnumUnit {
    fn default() -> Self {
        Self::Unit
    }
}

#[test]
fn test_ser_enum_unit_encode() {
    let mut buf = BytesMut::new();

    let i = TestEnumUnit::Unit;
    i.encode(&mut buf, None)
        .expect("Expected encoding to succeed");

    let mut comparison = BytesMut::new();
    comparison.put_u8(1);

    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_ser_enum_unit_decode() {
    let mut comparison = BytesMut::new();
    comparison.put_u8(1);

    let i =
        TestEnumUnit::decode(&mut comparison, None).expect("Expected decode of enum to succeed");

    assert_eq!(TestEnumUnit::Unit, i);
}
