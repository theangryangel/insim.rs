use std::{default::Default, net::Ipv4Addr};

use bytes::{Buf, BufMut};
use indexmap::{set::Iter as IndexSetIter, IndexSet};
use insim_core::{Decode, Encode};

use crate::identifiers::RequestId;

const IPB_MAX_BANS: usize = 120;

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// IP ban list (host only).
///
/// - Used to set or retrieve banned IP addresses.
pub struct Ipb {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    banips: IndexSet<Ipv4Addr>,
}

impl Ipb {
    /// Returns `true` if an IP is contained in this list.
    pub fn contains(&self, v: &Ipv4Addr) -> bool {
        self.banips.contains(v)
    }

    /// Add an IP address to the ban list.
    pub fn insert(&mut self, ip: Ipv4Addr) -> bool {
        self.banips.insert(ip)
    }

    /// Remove an IP address from the ban list.
    pub fn remove(&mut self, ip: &Ipv4Addr) -> bool {
        self.banips.shift_remove(ip)
    }

    /// Is the ban list empty?
    pub fn is_empty(&self) -> bool {
        self.banips.is_empty()
    }

    /// Clear the ban list.
    pub fn clear(&mut self) {
        self.banips.clear()
    }

    /// Iterator for all banned IPs.
    pub fn iter(&self) -> IndexSetIter<'_, Ipv4Addr> {
        self.banips.iter()
    }

    /// Returns the number of banned IPs.
    pub fn len(&self) -> usize {
        self.banips.len()
    }
}

impl Decode for Ipb {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf).map_err(|e| e.nested().context("Ipb::reqi"))?;
        let mut numb = u8::decode(buf).map_err(|e| e.nested().context("Ipb::numb"))?;
        buf.advance(4);
        let mut banips = IndexSet::with_capacity(numb as usize);
        while numb > 0 {
            let ip = Ipv4Addr::from(u32::decode(buf).map_err(|e| e.nested().context("Ipb::ip"))?);
            let _ = banips.insert(ip);
            numb -= 1;
        }

        Ok(Self { reqi, banips })
    }
}

impl Encode for Ipb {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi
            .encode(buf)
            .map_err(|e| e.nested().context("Ipb::reqi"))?;
        let numb = self.banips.len();
        if numb > IPB_MAX_BANS {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: IPB_MAX_BANS,
                found: numb,
            }
            .context("Ipb::numb"));
        }
        (numb as u8)
            .encode(buf)
            .map_err(|e| e.nested().context("Ipb::numb"))?;
        buf.put_bytes(0, 4);
        for i in self.banips.iter() {
            u32::from(*i)
                .encode(buf)
                .map_err(|e| e.nested().context("Ipb::ip"))?;
        }

        Ok(())
    }
}

impl_typical_with_request_id!(Ipb);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipb() {
        assert_from_to_bytes!(
            Ipb,
            [
                2, // reqi
                1, // numb
                0, 0, 0, 0, // padding / unused
                1, 0, 0, 127, // mod 1
            ],
            |ibp: Ipb| {
                assert_eq!(ibp.reqi, RequestId(2));
                assert_eq!(ibp.len(), 1);
                assert!(ibp.contains(&Ipv4Addr::new(127, 0, 0, 1)));
            }
        );
    }
}
