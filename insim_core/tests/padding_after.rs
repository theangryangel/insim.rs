use bytes::{Buf, BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, Default, PartialEq, Eq, Debug)]
struct TestPaddingAfter {
    pub i: u8,
    #[insim(pad_bytes_after = "2")]
    pub j: u32,
}

#[test]
fn test_padding_after_encode() {
    let mut buf = BytesMut::new();

    let i = TestPaddingAfter { i: 1, j: 99 };

    let mut comparison = BytesMut::new();
    comparison.put_u8(i.i);
    comparison.put_u32_le(i.j);
    comparison.put_bytes(0, 2);

    let res = i.encode(&mut buf);
    assert!(res.is_ok());
    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_padding_after_decode() {
    let i = TestPaddingAfter { i: 1, j: 99 };

    let mut comparison = BytesMut::new();
    comparison.put_u8(i.i);
    comparison.put_u32_le(i.j);
    comparison.put_bytes(0, 2);

    let decoded = TestPaddingAfter::decode(&mut comparison, None);
    assert!(decoded.is_ok(), "Expected decode to succeed");
    assert_eq!(decoded.unwrap(), i, "Expected decoded struct to match");
}
