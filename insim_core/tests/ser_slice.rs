use bytes::{Buf, BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, Debug, Default, Eq, PartialEq)]
struct Testu8 {
    pub i: [u8; 2],
}

#[test]
fn test_ser_slice_encode() {
    let mut buf = BytesMut::new();

    let i = Testu8 { i: [1, 2] };

    let mut comparison = BytesMut::new();
    comparison.put_u8(i.i[0]);
    comparison.put_u8(i.i[1]);

    let res = i.encode(&mut buf, None);
    assert!(res.is_ok());
    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_ser_slice_decode() {
    let i = Testu8 { i: [1, 2] };

    let mut comparison = BytesMut::new();
    comparison.put_u8(i.i[0]);
    comparison.put_u8(i.i[1]);

    let decoded = Testu8::decode(&mut comparison, None);

    assert!(decoded.is_ok());
    let decoded = decoded.unwrap();

    assert_eq!(decoded, i);
}
