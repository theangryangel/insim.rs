use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeString, Encode, EncodeString};

use crate::identifiers::{ClickId, ConnectionId, RequestId};

const BTN_TEXT_MAX_LEN: usize = 240;
const BTN_TEXT_ALIGN: usize = 4;

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

/// Colour
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
pub enum BtnStyleColour {
    /// NotEditable, defaults to light grey
    #[default]
    NotEditable = 0,
    /// Title, defaults to yellow
    Title = 1,
    /// Unselected text, defaults to black
    UnselectedText = 2,
    /// Selected text, defaults to white
    SelectedText = 3,
    /// Ok, defaults to green
    Ok = 4,
    /// Cancel, defaults to red
    Cancel = 5,
    /// Text string, defaults to pale blue
    TextString = 6,
    /// Unavailable, defaults to grey
    Unavailable = 7,
}

bitflags::bitflags! {
    /// Bitwise flags used within the [Btn] packet
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct BtnStyleFlags: u8 {
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

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
/// Button style
pub struct BtnStyle {
    /// Colour
    pub colour: BtnStyleColour,
    /// Behavioural flags
    pub flags: BtnStyleFlags,
}

impl BtnStyle {
    /// Light grey / NotEditable
    pub fn light_grey(mut self) -> Self {
        self.colour = BtnStyleColour::NotEditable;
        self
    }

    /// Yellow/ Title
    pub fn yellow(mut self) -> Self {
        self.colour = BtnStyleColour::Title;
        self
    }

    /// Black / UnselectedText
    pub fn black(mut self) -> Self {
        self.colour = BtnStyleColour::UnselectedText;
        self
    }

    /// White / SelectedText
    pub fn white(mut self) -> Self {
        self.colour = BtnStyleColour::SelectedText;
        self
    }

    /// Green / Ok
    pub fn green(mut self) -> Self {
        self.colour = BtnStyleColour::Ok;
        self
    }

    /// Red / Cancel
    pub fn red(mut self) -> Self {
        self.colour = BtnStyleColour::Cancel;
        self
    }

    /// Pale blue / TextString
    pub fn pale_blue(mut self) -> Self {
        self.colour = BtnStyleColour::TextString;
        self
    }

    /// Grey / Unavailable
    pub fn grey(mut self) -> Self {
        self.colour = BtnStyleColour::Unavailable;
        self
    }

    /// Set button as clickable
    pub fn clickable(mut self) -> Self {
        self.flags.set(BtnStyleFlags::CLICK, true);
        self
    }

    /// Light button
    pub fn light(mut self) -> Self {
        self.flags.set(BtnStyleFlags::LIGHT, true);
        self.flags.set(BtnStyleFlags::DARK, false);
        self
    }

    /// Dark button
    pub fn dark(mut self) -> Self {
        self.flags.set(BtnStyleFlags::DARK, true);
        self.flags.set(BtnStyleFlags::LIGHT, false);
        self
    }

    /// Align text left
    pub fn align_left(mut self) -> Self {
        self.flags.set(BtnStyleFlags::LEFT, true);
        self
    }

    /// Align text right
    pub fn align_right(mut self) -> Self {
        self.flags.set(BtnStyleFlags::RIGHT, true);
        self
    }
}

impl Encode for BtnStyle {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let colour = match self.colour {
            BtnStyleColour::NotEditable => 0,
            BtnStyleColour::Title => 1,
            BtnStyleColour::UnselectedText => 2,
            BtnStyleColour::SelectedText => 3,
            BtnStyleColour::Ok => 4,
            BtnStyleColour::Cancel => 5,
            BtnStyleColour::TextString => 6,
            BtnStyleColour::Unavailable => 7,
        };
        let flags = self.flags.bits();

        (colour | flags).encode(buf)
    }
}

impl Decode for BtnStyle {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let val = u8::decode(buf)?;

        let colour = match val & !248 {
            1 => BtnStyleColour::Title,
            2 => BtnStyleColour::UnselectedText,
            3 => BtnStyleColour::SelectedText,
            4 => BtnStyleColour::Ok,
            5 => BtnStyleColour::Cancel,
            6 => BtnStyleColour::TextString,
            7 => BtnStyleColour::Unavailable,
            _ => BtnStyleColour::NotEditable,
        };
        let flags = BtnStyleFlags::from_bits_truncate(val);

        Ok(Self { colour, flags })
    }
}

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

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
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

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
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

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
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

    /// Button style,
    pub bstyle: BtnStyle,

    /// Max chars permitted for a button with input
    pub typein: Option<u8>,

    /// Position - left (0-200)
    pub l: u8,

    /// Position - top (0-200)
    pub t: u8,

    /// Position - width (0-200)
    pub w: u8,

    /// Position - height (0-200)
    pub h: u8,

    /// Optional caption
    pub caption: Option<String>,

    /// Text
    pub text: String,
}

impl Decode for Btn {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        let ucid = ConnectionId::decode(buf)?;
        let clickid = ClickId::decode(buf)?;
        let inst = BtnInst::decode(buf)?;
        let bstyle = BtnStyle::decode(buf)?;
        let typein = u8::decode(buf)?;
        let typein = if typein > 0 { Some(typein) } else { None };
        let l = u8::decode(buf)?;
        let t = u8::decode(buf)?;
        let w = u8::decode(buf)?;
        let h = u8::decode(buf)?;

        let (caption, text) = if let Some(&0_u8) = buf.first() {
            // text with caption has a leading \0
            buf.advance(1);

            // find the caption ending
            let split = buf.iter().position(|c| c == &0_u8).unwrap();

            let caption = buf.split_to(split);
            let caption = insim_core::string::codepages::to_lossy_string(&caption);

            buf.advance(1);

            let text = insim_core::string::codepages::to_lossy_string(
                insim_core::string::strip_trailing_nul(buf.as_ref()),
            )
            .to_string();

            // ensure that we don't leave anything unconsumed, so that the codec doesnt raise an
            // incomplete parse error
            let remaining = buf.remaining();
            buf.advance(remaining);

            (Some(caption.to_string()), text.to_string())
        } else {
            // just text
            (None, String::decode_codepage(buf, buf.remaining())?)
        };

        Ok(Self {
            reqi,
            ucid,
            clickid,
            inst,
            bstyle,
            typein,
            l,
            t,
            w,
            h,
            caption,
            text,
        })
    }
}

impl Encode for Btn {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        if self.l > 200 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        if self.t > 200 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        if self.w > 200 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        if self.h > 200 {
            return Err(insim_core::EncodeError::TooLarge);
        }

        self.reqi.encode(buf)?;
        self.ucid.encode(buf)?;
        self.clickid.encode(buf)?;
        self.inst.encode(buf)?;
        self.bstyle.encode(buf)?;
        if let Some(typein) = self.typein {
            typein.encode(buf)?;
        } else {
            0_u8.encode(buf)?;
        }
        self.l.encode(buf)?;
        self.t.encode(buf)?;
        self.w.encode(buf)?;
        self.h.encode(buf)?;

        if let Some(caption) = &self.caption {
            let caption = insim_core::string::codepages::to_lossy_bytes(caption);
            let text = insim_core::string::codepages::to_lossy_bytes(&self.text);

            if (caption.len() + text.len()) > (BTN_TEXT_MAX_LEN - 2) {
                return Err(insim_core::EncodeError::TooLarge);
            }

            buf.put_u8(0);
            buf.extend_from_slice(&caption);
            buf.put_u8(0);
            buf.extend_from_slice(&text);

            let written = caption.len() + text.len() + 2;
            if written < BTN_TEXT_MAX_LEN {
                let align_to = BTN_TEXT_ALIGN - 1;
                let round_to = (written + align_to) & !align_to;
                let round_to = round_to.min(BTN_TEXT_MAX_LEN);
                buf.put_bytes(0, round_to - written);
            }
        } else {
            self.text
                .encode_codepage_with_alignment(buf, 240, 4, false)?;
        }

        Ok(())
    }
}

impl_typical_with_request_id!(Btn);

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
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
    #[insim(pad_after = 1)]
    pub cflags: BtnClickFlags,
}

#[derive(Debug, Clone, Default)]
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

    /// From original button specification (IS_BTN)
    pub typein: Option<u8>,

    /// Typed text, zero to TypeIn specified in IS_BTN
    pub text: String,
}

impl Decode for Btt {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        let ucid = ConnectionId::decode(buf)?;
        let clickid = ClickId::decode(buf)?;
        let inst = BtnInst::decode(buf)?;
        let typein = u8::decode(buf)?;
        if typein > 96 {
            return Err(insim_core::DecodeError::TooLarge);
        }
        let typein = if typein > 0 { Some(typein) } else { None };
        buf.advance(1);
        let text = String::decode_codepage(buf, 96)?;
        Ok(Self {
            reqi,
            ucid,
            clickid,
            inst,
            typein,
            text,
        })
    }
}

impl Encode for Btt {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        self.ucid.encode(buf)?;
        self.clickid.encode(buf)?;
        self.inst.encode(buf)?;
        let typein = self.typein.unwrap_or(0_u8);
        if typein > 96 {
            return Err(insim_core::EncodeError::TooLarge);
        }
        typein.encode(buf)?;
        buf.put_bytes(0, 1);
        self.text.encode_codepage(buf, 96, false)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_btnstyle() {
        let mut bstyle = BtnStyle::default();
        bstyle.flags.set(BtnStyleFlags::CLICK, true);
        bstyle.colour = BtnStyleColour::Title;

        let mut buf = BytesMut::new();
        bstyle.encode(&mut buf).unwrap();

        assert_eq!(buf.as_ref(), [9]);
    }

    #[test]
    fn test_bfn() {
        assert_from_to_bytes!(
            Bfn,
            [
                0,   // reqi
                3,   // subt
                4,   // ucid
                45,  // clickid
                48,  // clickmax
                128, // inst
            ],
            |parsed: Bfn| {
                assert_eq!(parsed.ucid, ConnectionId(4));
                assert!(matches!(parsed.subt, BfnType::BtnRequest));
                assert_eq!(parsed.clickid, ClickId(45));
                assert_eq!(parsed.clickmax, 48);
                assert!(parsed.inst.contains(BtnInst::ALWAYSON));
            }
        );
    }

    #[test]
    fn test_btn_without_caption() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            0,   // reqi
            4,   // ucid
            45,  // clickid
            128, // inst
            9,   // bstyle
            3,   // typein
            20,  // l
            30,  // t
            40,  // w
            50,  // h
        ]);
        data.extend_from_slice(b"123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789$");

        assert_from_to_bytes!(Btn, data.as_ref(), |parsed: Btn| {
            assert_eq!(parsed.ucid, ConnectionId(4));
            assert_eq!(parsed.clickid, ClickId(45));
            assert!(matches!(parsed.typein, Some(3)));
            assert_eq!(parsed.l, 20);
            assert_eq!(parsed.t, 30);
            assert_eq!(parsed.w, 40);
            assert_eq!(parsed.h, 50);
        });
    }

    #[test]
    fn test_btn_with_caption() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            0,   // reqi
            4,   // ucid
            45,  // clickid
            128, // inst
            9,   // bstyle
            3,   // typein
            20,  // l
            30,  // t
            40,  // w
            50,  // h
        ]);
        data.put_u8(0);
        data.extend_from_slice(b"1234");
        data.put_u8(0);
        data.extend_from_slice(b"abcdefg");
        data.put_bytes(0, 3);

        assert_from_to_bytes!(Btn, data.as_ref(), |parsed: Btn| {
            assert_eq!(parsed.ucid, ConnectionId(4));
            assert_eq!(parsed.clickid, ClickId(45));
            assert!(matches!(parsed.typein, Some(3)));
            assert_eq!(parsed.l, 20);
            assert_eq!(parsed.t, 30);
            assert_eq!(parsed.w, 40);
            assert_eq!(parsed.h, 50);
            assert_eq!(&parsed.caption.unwrap(), "1234");
            assert_eq!(&parsed.text, "abcdefg");
        });
    }

    #[test]
    fn test_btn_with_zero_typein_is_none() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            0,   // reqi
            4,   // ucid
            45,  // clickid
            128, // inst
            9,   // bstyle
            0,   // typein
            20,  // l
            30,  // t
            40,  // w
            50,  // h
        ]);
        data.put_u8(0);
        data.extend_from_slice(b"1234");
        data.put_u8(0);
        data.extend_from_slice(b"abcdefg");
        data.put_bytes(0, 3);

        assert_from_to_bytes!(Btn, data.as_ref(), |parsed: Btn| {
            assert!(parsed.typein.is_none());
        });
    }

    #[test]
    fn test_btc() {
        assert_from_to_bytes!(
            Btc,
            [
                1,   // reqi
                2,   // ucid
                3,   // clickid
                128, // inst
                2,   // cflags
                0,
            ],
            |parsed: Btc| {
                assert_eq!(parsed.ucid, ConnectionId(2));
                assert_eq!(parsed.clickid, ClickId(3));
                assert!(parsed.inst.contains(BtnInst::ALWAYSON));
                assert!(parsed.cflags.contains(BtnClickFlags::RMB));
            }
        );
    }

    #[test]
    fn test_btt() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            1,   // reqi
            2,   // ucid
            3,   // clickid
            128, // inst
            7,   // typein
            0,   // sp3
        ]);
        data.extend_from_slice(b"123456|^$");
        data.put_bytes(0, 87);

        assert_from_to_bytes!(Btt, data.as_ref(), |parsed: Btt| {
            assert_eq!(parsed.ucid, ConnectionId(2));
            assert_eq!(parsed.clickid, ClickId(3));
            assert!(parsed.inst.contains(BtnInst::ALWAYSON));
            assert!(matches!(parsed.typein, Some(7)));
            assert_eq!(parsed.text, "123456|^$");
        });
    }
}
