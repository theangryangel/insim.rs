use bytes::{Buf, BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, Default, PartialEq, Eq, Debug)]
struct TestWrapped(u8, u32);

#[test]
fn test_ser_wrapped_encode() {
    let mut buf = BytesMut::new();

    let i = TestWrapped(1, 2);

    let mut comparison = BytesMut::new();
    comparison.put_u8(i.0);
    comparison.put_u32_le(i.1);

    let res = i.encode(&mut buf);
    assert!(res.is_ok());
    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_ser_wrapped_decode() {
    let i = TestWrapped(2, 3);

    let mut comparison = BytesMut::new();
    comparison.put_u8(i.0);
    comparison.put_u32_le(i.1);

    let decoded = TestWrapped::decode(&mut comparison, None);

    assert!(decoded.is_ok());
    let decoded = decoded.unwrap();

    assert_eq!(decoded, i);
}
