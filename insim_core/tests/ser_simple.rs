use bytes::{Buf, BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, Debug, Default, Eq, PartialEq)]
struct Testu8 {
    pub i: u8,
}

#[test]
fn test_ser_u8_encode() {
    let mut buf = BytesMut::new();

    let i = Testu8 { i: 1 };

    let mut comparison = BytesMut::new();
    comparison.put_u8(i.i);

    let res = i.encode(&mut buf, None);
    assert!(res.is_ok());
    assert_eq!(&buf[..], b"\x01");
}

#[test]
fn test_ser_u8_decode() {
    let i = Testu8 { i: 1 };

    let mut comparison = BytesMut::new();
    comparison.put_u8(i.i);

    let decoded = Testu8::decode(&mut comparison, None);

    assert!(decoded.is_ok());
    let decoded = decoded.unwrap();

    assert_eq!(decoded, i);
}
