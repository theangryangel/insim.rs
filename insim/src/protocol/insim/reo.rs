use bytes::BytesMut;
use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*, DecodableError, EncodableError, ser::Limit
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Default)]
// we need to wrap [PlayerId; 32] in a new type because arrays are always considered "foreign", and the trait Decodable isn't defined within this crate.
// FIXME: add some extra methods for convenience
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ReoPlayerList([PlayerId; 32]);

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Reorder
pub struct Reo {
    pub reqi: RequestId,

    pub nump: u8,

    pub plid: ReoPlayerList,
}

impl Decodable for ReoPlayerList {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        let mut data: ReoPlayerList = Default::default();
        for i in 0..32 {
            data.0[i] = PlayerId::decode(buf, None)?;
        }

        Ok(data)
    }
}

impl Encodable for ReoPlayerList {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized {

        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!("ReoPlayerList does not support a limit: {:?}", limit)));
        }

        for i in self.0.iter() {
            i.encode(buf, None)?;
        }

        Ok(())
    }
}
