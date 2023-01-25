use bytes::{Buf, BufMut, BytesMut};
use insim_core::{Decodable, Encodable, InsimDecode, InsimEncode};

#[derive(InsimEncode, InsimDecode, Default, PartialEq, Eq, Debug)]
struct TestTheFarmChild {
    pub i: u8,
    pub j: u32,
    pub k: bool,
}

#[derive(InsimEncode, InsimDecode, Default, PartialEq, Eq, Debug)]
struct TestTheFarm {
    pub i: u8,
    #[insim(pad_bytes_before = "2")]
    pub j: u32,
    pub k: TestTheFarmChild,
    #[insim(skip)]
    #[allow(unused)]
    pub l: u8,
    pub m: u8,
}

#[derive(InsimEncode, InsimDecode, PartialEq, Eq, Debug)]
#[repr(u8)]
enum TestTheFarmEnum {
    Moo(TestTheFarm) = 0,
    Oink(u8) = 1,
}

impl Default for TestTheFarmEnum {
    fn default() -> Self {
        TestTheFarmEnum::Oink(100)
    }
}

#[test]
fn test_ser_the_farm_encode() {
    let mut buf = BytesMut::new();

    let i = TestTheFarmEnum::Moo(TestTheFarm {
        i: 1,
        j: 99,
        k: TestTheFarmChild {
            i: 3,
            j: 4,
            k: true,
        },
        l: 88,
        m: 1,
    });

    let mut comparison = BytesMut::new();

    comparison.put_u8(0);
    comparison.put_u8(1);
    comparison.put_bytes(0, 2);
    comparison.put_u32_le(99);

    comparison.put_u8(3);
    comparison.put_u32_le(4);
    comparison.put_u8(true as u8);

    comparison.put_u8(1);

    let res = i.encode(&mut buf, None);
    assert!(res.is_ok());
    assert_eq!(&buf[..], &comparison[..]);
}

#[test]
fn test_ser_the_farm_decode() {
    let i = TestTheFarmEnum::Moo(TestTheFarm {
        i: 1,
        j: 99,
        k: TestTheFarmChild {
            i: 3,
            j: 4,
            k: true,
        },
        l: 0,
        m: 1,
    });

    let mut comparison = BytesMut::new();

    comparison.put_u8(0);
    comparison.put_u8(1);
    comparison.put_bytes(0, 2);
    comparison.put_u32_le(99);

    comparison.put_u8(3);
    comparison.put_u32_le(4);
    comparison.put_u8(true as u8);

    comparison.put_u8(1);

    let decoded = TestTheFarmEnum::decode(&mut comparison, None).unwrap();
    assert_eq!(decoded, i, "Expected decoded struct to match");
}
