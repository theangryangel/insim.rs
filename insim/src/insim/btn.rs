use insim_core::{
    binrw::{self, binrw},
    identifiers::{ClickId, ConnectionId, RequestId},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags::bitflags! {
    /// Bitwise flags used within the [Sta] packet
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
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

bitflags::bitflags! {
    /// Bitwise flags used within the [Sta] packet
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct BtnClickFlags: u8 {
        const LMB = (1 << 0);

        const RMB = (1 << 1);

        const CTRL = (1 << 2);

        const SHIFT = (1 << 3);
    }
}

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within [Bfn] to specify the action to take.
pub enum BfnType {
    #[default]
    DeleteButton = 0,

    Clear = 1,

    UserClear = 2,

    ButtonsRequested = 3,
}

#[binrw]
#[derive(Debug, Clone, Default)]
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

#[binrw]
#[derive(Debug, Clone, Default)]
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

    #[br(parse_with = binrw_parse_codepage_string::<240, _>)]
    #[bw(write_with = binrw_write_codepage_string::<240, _>, args(false, 4))]
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::Btn;
    use insim_core::binrw::BinWrite;
    use std::io::Cursor;

    #[test]
    fn test_btn() {
        let data = Btn {
            text: "aaaaa".into(),
            ..Default::default()
        };

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());
        let buf = buf.into_inner();

        // we need to add the size and type to the buf len
        assert_eq!(buf.len() + 2, 20);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button Click - Sent back when a user clicks a button
pub struct Btc {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: ClickId,

    pub inst: u8,

    #[brw(pad_after = 1)]
    pub cflags: BtnClickFlags,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Button Type - Sent back when a user types into a text entry "button"
pub struct Btt {
    pub reqi: RequestId,
    pub ucid: ConnectionId,
    pub clickid: ClickId,
    pub inst: u8,

    #[brw(pad_after = 1)]
    pub typein: u8,

    #[br(parse_with = binrw_parse_codepage_string::<96, _>)]
    #[bw(write_with = binrw_write_codepage_string::<96, _>)]
    pub text: String,
}
