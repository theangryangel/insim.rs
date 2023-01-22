use bytes::{BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, PartialEq, Eq, Debug)]
#[repr(u8)]
#[allow(unused)]
enum TestEnum {
    U8(u8) = 1,
    U32(u32) = 2,
    I8(i8) = 4,
}

impl Default for TestEnum {
    fn default() -> Self {
        Self::U8(100)
    }
}

#[test]
fn test_enum_encode() {
    let mut buf = BytesMut::new();

    let i = TestEnum::I8(-1);
    i.encode(&mut buf).expect("Expected encoding to succeed");

    let mut comparison = BytesMut::new();
    comparison.put_u8(4);
    comparison.put_i8(-1);

    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_enum_decode() {
    let mut comparison = BytesMut::new();
    comparison.put_u8(4);
    comparison.put_i8(-1);

    let i = TestEnum::decode(&mut comparison, None).expect("Expected decode of enum to succeed");

    assert_eq!(TestEnum::I8(-1), i);
}
