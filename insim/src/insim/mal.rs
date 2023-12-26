use crate::error::Error;
use indexmap::{set::Iter as IndexSetIter, IndexSet};
use insim_core::{
    binrw::{self, binrw, BinRead, BinResult, BinWrite},
    identifiers::{ConnectionId, RequestId},
    vehicle::Vehicle,
};
use std::default::Default;

#[cfg(feature = "serde")]
use serde::Serialize;

const MAX_MAL_SIZE: usize = 120;

#[binrw::parser(reader, endian)]
pub fn binrw_parse_mal_allowed_mods(count: u8) -> BinResult<IndexSet<Vehicle>> {
    let mut data = IndexSet::new();
    for _i in 0..count {
        data.insert(Vehicle::Mod(u32::read_options(reader, endian, ())?));
    }
    Ok(data)
}

#[binrw::writer(writer, endian)]
pub fn binrw_write_mal_allowed_mods(input: &IndexSet<Vehicle>) -> BinResult<()> {
    for i in input.iter() {
        match i {
            Vehicle::Mod(val) => val.write_options(writer, endian, ())?,
            _ => {
                panic!("Non-Mod vehicle managed to get into the HashSet. Should not be possible.")
            }
        }
    }

    Ok(())
}

#[binrw]
#[bw(assert(allowed_mods.len() <= MAX_MAL_SIZE))]
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Mods Allowed - restrict the mods that can be used
pub struct Mal {
    pub reqi: RequestId,

    /// Number of mods in this packet
    #[bw(calc = allowed_mods.len() as u8)]
    numm: u8,

    #[brw(pad_after = 3)]
    pub ucid: ConnectionId,

    #[br(parse_with = binrw_parse_mal_allowed_mods, args(numm))]
    #[bw(write_with = binrw_write_mal_allowed_mods)]
    allowed_mods: IndexSet<Vehicle>,
}

impl Mal {
    pub fn contains(&self, v: &Vehicle) -> bool {
        self.allowed_mods.contains(v)
    }

    /// Push a compressed form of a mod onto the list of allowed mods
    /// and update the count.
    pub fn insert(&mut self, vehicle: Vehicle) -> Result<bool, Error> {
        match vehicle {
            Vehicle::Mod(_) => Ok(self.allowed_mods.insert(vehicle)),
            _ => Err(Error::VehicleNotAMod),
        }
    }

    pub fn remove(&mut self, vehicle: &Vehicle) -> bool {
        self.allowed_mods.remove(vehicle)
    }

    pub fn is_empty(&self) -> bool {
        self.allowed_mods.is_empty()
    }

    /// Clear any previously allowed mods.
    pub fn clear(&mut self) {
        self.allowed_mods.clear()
    }

    pub fn iter(&self) -> IndexSetIter<'_, Vehicle> {
        self.allowed_mods.iter()
    }

    pub fn len(&self) -> usize {
        self.allowed_mods.len()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek};

    use super::*;

    #[test]
    fn test_standard_vehicle_rejected() {
        let mut data = Mal::default();

        assert!(data.insert(Vehicle::Xfg).is_err());
        assert!(data.insert(Vehicle::Mod(1)).is_ok());
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn test_encoding() {
        let mut data = Mal::default();
        assert!(data.insert(Vehicle::Mod(1)).is_ok());
        assert!(data.insert(Vehicle::Mod(2)).is_ok());
        data.reqi = RequestId(2);
        data.ucid = ConnectionId(3);

        let mut buf = Cursor::new(Vec::new());
        data.write_le(&mut buf).unwrap();
        buf.rewind().unwrap();

        let buf2 = buf.clone().into_inner();
        assert_eq!(
            buf2,
            [
                2, // reqi
                2, // numm
                3, // connection id
                0, 0, 0, // padding / unused
                1, 0, 0, 0, // mod 1
                2, 0, 0, 0, // mod 2
            ]
        );

        let data2 = Mal::read_le(&mut buf).unwrap();
        assert_eq!(data, data2);
        assert_eq!(data2.len(), 2);
    }
}
