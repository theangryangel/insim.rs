use insim_core::{
    identifiers::{ClickId, ConnectionId, RequestId},
    prelude::*,
    ser::Limit,
};

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags::bitflags! {
    /// Bitwise flags used within the [Sta] packet
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct BtnStyleFlags: u8 {
        const C1 = (1 << 0);

        const C2 = (1 << 1);

        const C4 = (1 << 2);

        const CLICK = (1 << 3);

        const LIGHT = (1 << 4);

        const DARK = (1 << 5);

        const LEFT = (1 << 6);

        const RIGHT = (1 << 7);
    }
}

impl Decodable for BtnStyleFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        Ok(Self::from_bits_truncate(u8::decode(buf, limit)?))
    }
}

impl Encodable for BtnStyleFlags {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.bits().encode(buf, limit)?;
        Ok(())
    }
}

bitflags::bitflags! {
    /// Bitwise flags used within the [Sta] packet
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct BtnClickFlags: u8 {
        const LMB = (1 << 0);

        const RMB = (1 << 1);

        const CTRL = (1 << 2);

        const SHIFT = (1 << 3);
    }
}

impl Decodable for BtnClickFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        Ok(Self::from_bits_truncate(u8::decode(buf, limit)?))
    }
}

impl Encodable for BtnClickFlags {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.bits().encode(buf, limit)?;
        Ok(())
    }
}

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within [Bfn] to specify the action to take.
pub enum BfnType {
    #[default]
    DeleteButton = 0,

    Clear = 1,

    UserClear = 2,

    ButtonsRequested = 3,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button Function
pub struct Bfn {
    pub reqi: RequestId,
    pub subt: BfnType,

    pub ucid: ConnectionId,
    pub clickid: ClickId,
    pub clickmax: u8,
    pub inst: u8,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button
pub struct Btn {
    pub reqi: RequestId,
    pub ucid: ConnectionId,

    pub clickid: ClickId,
    pub inst: u8,
    pub bstyle: BtnStyleFlags,
    pub typein: u8,

    pub l: u8,
    pub t: u8,
    pub w: u8,
    pub h: u8,

    #[insim(bytes = "240")]
    pub text: String, // FIXME: this should be upto 240 characters and always a multiple of 4
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button Click - Sent back when a user clicks a button
pub struct Btc {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: ClickId,

    pub inst: u8,
    #[insim(pad_bytes_after = "1")]
    pub cflags: BtnClickFlags,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button Type - Sent back when a user types into a text entry "button"
pub struct Btt {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: ClickId,
    pub inst: u8,

    #[insim(pad_bytes_after = "1")]
    pub typein: u8,

    #[insim(bytes = "96")]
    pub text: String,
}
