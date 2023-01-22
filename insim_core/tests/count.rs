use bytes::{Buf, BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, Default, PartialEq, Eq, Debug)]
struct TestCount {
    pub j: u32,

    #[insim(count = "j")]
    pub i: Vec<u8>,
}

#[test]
fn test_count_encode() {
    let mut buf = BytesMut::new();

    let i = TestCount {
        j: 0,
        i: vec![1, 2, 3],
    };

    let mut comparison = BytesMut::new();
    comparison.put_u32_le(3 as u32);
    for i in i.i.iter() {
        comparison.put_u8(*i);
    }

    let res = i.encode(&mut buf);
    assert!(res.is_ok());
    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_count_decode() {
    let i = TestCount {
        j: 3,
        i: vec![1, 2, 3],
    };

    let mut comparison = BytesMut::new();
    comparison.put_u32_le(3 as u32);
    for i in i.i.iter() {
        comparison.put_u8(*i);
    }

    let decoded = TestCount::decode(&mut comparison, None);
    assert!(decoded.is_ok(), "Expected decode to succeed");
    assert_eq!(decoded.unwrap(), i, "Expected decoded struct to match");
}
