use deku::prelude::*;

pub(crate) mod codec;
pub mod insim;
mod macros;
pub mod relay;
pub(crate) mod stream;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(endian = "little", type = "u8")]
pub enum Packet {
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
