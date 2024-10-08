use std::{default::Default, net::Ipv4Addr};

use indexmap::{set::Iter as IndexSetIter, IndexSet};
use insim_core::binrw::{self, binrw, BinRead, BinResult, BinWrite};

use crate::identifiers::RequestId;

const IPB_MAX_BANS: usize = 120;

#[binrw::parser(reader, endian)]
fn binrw_parse_ipb_bans(count: u8) -> BinResult<IndexSet<Ipv4Addr>> {
    let mut data = IndexSet::new();
    for _i in 0..count {
        let ip = Ipv4Addr::from(u32::read_options(reader, endian, ())?);
        let _ = data.insert(ip);
    }
    Ok(data)
}

#[binrw::writer(writer, endian)]
fn binrw_write_ipb_bans(input: &IndexSet<Ipv4Addr>) -> BinResult<()> {
    for i in input.iter() {
        u32::from(*i).write_options(writer, endian, ())?;
    }

    Ok(())
}

#[binrw]
#[bw(assert(banips.len() <= IPB_MAX_BANS))]
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player Bans - Receive or set player bans, by IP address
pub struct Ipb {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Number of bans in this packet, from the wire
    /// This value is not to be trusted as we use an IndexSet to record the bans internally. It is technically
    /// possible that LFS could return duplicate entries, but we have no way of verifying that.
    #[bw(calc = banips.len() as u8)]
    #[brw(pad_after = 4)]
    numb: u8,

    #[br(parse_with = binrw_parse_ipb_bans, args(numb))]
    #[bw(write_with = binrw_write_ipb_bans)]
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

impl_typical_with_request_id!(Ipb);

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek};

    use super::*;

    #[test]
    fn test_encoding() {
        let mut bans = Ipb::default();
        bans.reqi = RequestId(2);
        let _ = bans.insert(Ipv4Addr::new(127, 0, 0, 1));

        let mut buf = Cursor::new(Vec::new());
        bans.write_le(&mut buf).unwrap();
        buf.rewind().unwrap();

        let buf2 = buf.clone().into_inner();
        assert_eq!(
            buf2,
            [
                2, // reqi
                1, // numb
                0, 0, 0, 0, // padding / unused
                1, 0, 0, 127, // mod 1
            ]
        );

        let data2 = Ipb::read_le(&mut buf).unwrap();
        assert_eq!(bans, data2);
        assert_eq!(data2.len(), 1);
        assert!(data2.contains(&Ipv4Addr::new(127, 0, 0, 1)));
    }
}
