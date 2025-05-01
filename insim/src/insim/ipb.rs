use std::{default::Default, net::Ipv4Addr};

use bytes::{Buf, BufMut};
use indexmap::{set::Iter as IndexSetIter, IndexSet};
use insim_core::{Decode, Encode};

use crate::identifiers::RequestId;

const IPB_MAX_BANS: usize = 120;

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player Bans - Receive or set player bans, by IP address
pub struct Ipb {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    banips: IndexSet<Ipv4Addr>,
}

impl Ipb {
    /// Returns `true` if a Vehicle is contained in this packet
    pub fn contains(&self, v: &Ipv4Addr) -> bool {
        self.banips.contains(v)
    }

    /// Push a compressed form of a mod onto the list of allowed mods
    /// and update the count.
    pub fn insert(&mut self, ip: Ipv4Addr) -> bool {
        self.banips.insert(ip)
    }

    /// Remove a Vehicle from this packet
    pub fn remove(&mut self, ip: &Ipv4Addr) -> bool {
        self.banips.shift_remove(ip)
    }

    /// Does this packet have no vehicles associated?
    pub fn is_empty(&self) -> bool {
        self.banips.is_empty()
    }

    /// Clear any previously allowed mods.
    pub fn clear(&mut self) {
        self.banips.clear()
    }

    /// Iterator for all allowed mods
    pub fn iter(&self) -> IndexSetIter<'_, Ipv4Addr> {
        self.banips.iter()
    }

    /// Returns the number of allowed mods
    pub fn len(&self) -> usize {
        self.banips.len()
    }
}

impl Decode for Ipb {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::decode(buf)?;
        let mut numb = u8::decode(buf)?;
        buf.advance(4);
        let mut banips = IndexSet::with_capacity(numb as usize);
        while numb > 0 {
            let ip = Ipv4Addr::from(u32::decode(buf)?);
            let _ = banips.insert(ip);
            numb -= 1;
        }

        Ok(Self { reqi, banips })
    }
}

impl Encode for Ipb {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.encode(buf)?;
        let numb = self.banips.len();
        if numb > IPB_MAX_BANS {
            return Err(insim_core::Error::TooLarge);
        }
        (numb as u8).encode(buf)?;
        buf.put_bytes(0, 4);
        for i in self.banips.iter() {
            u32::from(*i).encode(buf)?;
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
