use crate::string::{InsimStringReader, InsimStringWriter};
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::ctx::{Limit, Size};
use deku::prelude::*;

// TODO make these a custom type
pub fn lfs_string_read(
    rest: &BitSlice<Msb0, u8>,
    byte_size: usize,
) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
    // TODO tidy up error handling
    let (rest, value) = Vec::read(rest, Limit::new_size(Size::Bytes(byte_size)))?;

    Ok((
        rest,
        String::from_lfs(value).map_err(|e| DekuError::Parse(e.to_string()))?,
    ))
}

/// Parse from String to u8 and write
pub fn lfs_string_write(
    output: &mut BitVec<Msb0, u8>,
    field: &str,
    byte_size: usize,
) -> Result<(), DekuError> {
    let value = field
        .to_string()
        .to_lfs(byte_size)
        .map_err(|e| DekuError::Parse(e.to_string()))?;
    value.write(output, ())
}

pub mod insim;
pub mod relay;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "little", type = "u8")]
pub enum Insim {
    // TODO The rest of the packets
    //
    // TODO I hate the way we have to split the structs out in order to have sane Impl's.
    // (See https://github.com/rust-lang/rfcs/pull/2593).
    // TODO Can we mask enum somehow in the encoder/decoder so it's more transparent to the user?
    #[deku(id = "1")]
    Init(insim::Init),

    #[deku(id = "2")]
    Version(insim::Version),

    #[deku(id = "3")]
    Tiny(insim::Tiny),

    #[deku(id = "4")]
    Small(insim::Small),

    #[deku(id = "11")]
    MessageOut(insim::MessageOut),

    #[deku(id = "24")]
    Lap(insim::Lap),

    #[deku(id = "25")]
    SplitX(insim::SplitX),

    #[deku(id = "38")]
    MultiCarInfo(insim::MultiCarInfo),

    #[deku(id = "250")]
    RelayAdminRequest(relay::AdminRequest),

    #[deku(id = "251")]
    RelayAdminResponse(relay::AdminResponse),

    #[deku(id = "252")]
    RelayHostListRequest(relay::HostListRequest),

    #[deku(id = "253")]
    RelayHostList(relay::HostList),

    #[deku(id = "254")]
    RelayHostSelect(relay::HostSelect),

    #[deku(id = "255")]
    RelayError(relay::Error),
}
