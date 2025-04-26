use crate::identifiers::{ClickId, ConnectionId, RequestId};

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Bitwise flags used within the [Btn] packet
    /// Mainly used internally by InSim but also provides some extra user flags
    pub struct BtnInst: u8 {
        /// If this bit is set the button is visible in all screens
        const ALWAYSON = (1 << 7);
    }
}

impl_bitflags_from_to_bytes!(BtnInst, u8);

bitflags::bitflags! {
    /// Bitwise flags used within the [Btn] packet
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct BtnStyleFlags: u8 {

        // TODO: We need to abstract these lowest 3 bits with helpers to set the colours
        // colour 0: light grey			(not user editable)
        // colour 1: title colour		(default:yellow)
        // colour 2: unselected text	(default:black)
        // colour 3: selected text		(default:white)
        // colour 4: ok					(default:green)
        // colour 5: cancel				(default:red)
        // colour 6: text string		(default:pale blue)
        // colour 7: unavailable		(default:grey)

        /// TODO
        const C1 = (1 << 0);
        /// TODO
        const C2 = (1 << 1);
        /// TODO
        const C4 = (1 << 2);

        /// Click this button to send IS_BTC
        const CLICK = (1 << 3);

        /// Light button
        const LIGHT = (1 << 4);

        /// Dark button
        const DARK = (1 << 5);

        /// Align text left
        const LEFT = (1 << 6);

        /// Align text right
        const RIGHT = (1 << 7);
    }
}

impl_bitflags_from_to_bytes!(BtnStyleFlags, u8);

bitflags::bitflags! {
    /// Bitwise flags used within the [Sta] packet
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct BtnClickFlags: u8 {
        /// Left click
        const LMB = (1 << 0);

        /// Right click
        const RMB = (1 << 1);

        /// Ctrl+click
        const CTRL = (1 << 2);

        /// Shift+click
        const SHIFT = (1 << 3);
    }
}

impl_bitflags_from_to_bytes!(BtnClickFlags, u8);

#[derive(Debug, Default, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
/// Used within [Bfn] to specify the action to take.
pub enum BfnType {
    #[default]
    /// Instruction - delete one button or range of buttons (must set ClickID)
    DelBtn = 0,

    /// Instruction - clear all buttons
    Clear = 1,

    /// Report - user cleared all buttons
    UserClear = 2,

    /// Report - user requested buttons
    BtnRequest = 3,
}

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Button Function
pub struct Bfn {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Button function type
    pub subt: BfnType,

    /// Unique connection ID to send to or received from (0 = local / 255 = all)
    pub ucid: ConnectionId,

    /// If subt is BFN_DEL_BTN: ID of single button to delete or first button in range
    pub clickid: ClickId,

    /// If subt is BFN_DEL_BTN: ID of last button in range (if greater than ClickID)
    pub clickmax: u8,

    /// Priarmily used internally by LFS
    pub inst: BtnInst,
}

impl_typical_with_request_id!(Bfn);

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Button - Instructional to create a button
pub struct Btn {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Connection to display the button (0 = local / 255 = all)
    pub ucid: ConnectionId,

    /// Button ID (0 to 239)
    pub clickid: ClickId,

    /// Primarily used internally by LFS
    pub inst: BtnInst,

    /// Button style flags
    pub bstyle: BtnStyleFlags,

    /// Max chars permitted for a buttonw within input
    pub typein: u8,

    /// Position - left (0-200)
    pub l: u8,
    /// Position - top (0-200)
    pub t: u8,
    /// Position - width (0-200)
    pub w: u8,
    /// Position - height (0-200)
    pub h: u8,

    /// Text
    #[read_write_buf(codepage(length = 240, align_to = 4))]
    pub text: String,
}

impl_typical_with_request_id!(Btn);

#[cfg(test)]
mod tests {
    #[test]
    fn test_btn() {
        // FIXME
    }
}

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Button Click - Sent back when a user clicks a button
pub struct Btc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Connection that clicked the button (zero if local)
    pub ucid: ConnectionId,
    /// Button identifier originally sent in IS_BTN
    pub clickid: ClickId,

    /// Primarily used internally by LFS
    pub inst: BtnInst,

    /// Button click flags
    #[read_write_buf(pad_after = 1)]
    pub cflags: BtnClickFlags,
}

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Button Type - Sent back when a user types into a text entry "button"
pub struct Btt {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Connection that typed into the button (zero if local)
    pub ucid: ConnectionId,

    /// Button identifier originally sent in IS_BTN
    pub clickid: ClickId,

    /// Primarily used internally by LFS
    pub inst: BtnInst,

    #[read_write_buf(pad_after = 1)]
    /// From original button specification (IS_BTN)
    pub typein: u8,

    #[read_write_buf(codepage(length = 96))]
    /// Typed text, zero to TypeIn specified in IS_BTN
    pub text: String,
}
