use std::time::Duration;

use bytes::{Buf, BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, Debug, Default, Eq, PartialEq)]
struct Testu8 {
    pub i: Duration,
}

#[test]
fn test_ser_duration_encode() {
    let mut buf = BytesMut::new();

    let i = Testu8 {
        i: Duration::from_secs(1),
    };

    let mut comparison = BytesMut::new();
    comparison.put_u32_le(1000);

    let res = i.encode(&mut buf, None);
    assert!(res.is_ok());
    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_ser_duration_decode() {
    let i = Testu8 {
        i: Duration::from_secs(1),
    };

    let mut comparison = BytesMut::new();
    comparison.put_u32_le(1000);

    let decoded = Testu8::decode(&mut comparison, None);

    assert!(decoded.is_ok());
    let decoded = decoded.unwrap();

    assert_eq!(decoded, i);
}
