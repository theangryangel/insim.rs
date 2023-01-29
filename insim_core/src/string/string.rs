//! Utility functions for working with LFS' CString-like strings. If you need formatting through
//! Codepages you should be looking at the ICodePageString custom struct.
//!
//! Effectively LFS transmits strings as a "CString", with the exception that a CString must always
//! be terminated by a \0 byte. In LFS's wireformat this is not always the case.
//!
//! The istring module provides simple methods for reading and writing these.

use bytes::BytesMut;

use crate::{ser::Limit, Decodable, DecodableError, Encodable, EncodableError};

use super::strip_trailing_nul;

impl Encodable for String {
    fn encode(&self, buf: &mut BytesMut, limit: Option<Limit>) -> Result<(), EncodableError> {
        self.as_bytes().to_vec().encode(buf, limit)?; // FIXME implement [T].encode
        Ok(())
    }
}

impl Decodable for String {
    fn decode(buf: &mut BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError> {
        let limit = if limit.is_none() {
            tracing::warn!(
                "No limit received, assuming the rest of the buffer: {:?}",
                &buf
            );
            Some(Limit::Bytes(buf.len()))
        } else {
            limit
        };

        let binding = Vec::<u8>::decode(buf, limit)?;
        let data = strip_trailing_nul(&binding);
        Ok(String::from_utf8_lossy(data).to_string())
    }
}
