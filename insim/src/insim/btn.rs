use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeErrorKind, DecodeString, Encode, EncodeString};

use crate::identifiers::{ClickId, ConnectionId, RequestId};

const BTN_TEXT_MAX_LEN: usize = 240;
const BTN_TEXT_ALIGN: usize = 4;

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Bitwise flags used within [Btn], [Btc], and [Btt].
    ///
    /// - Mostly internal, but includes user-visible behavior.
    pub struct BtnInst: u8 {
        /// If this bit is set the button is visible in all screens
        const ALWAYSON = (1 << 7);
    }
}

impl_bitflags_from_to_bytes!(BtnInst, u8);

/// Colour
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Button style flags for [Btn].
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct BtnStyleFlags: u8 {
        /// Clickable button (sends [Btc]).
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Button style configuration (colour + flags).
pub struct BtnStyle {
    /// Colour selection.
    pub colour: BtnStyleColour,
    /// Behavioral flags (alignment, clickability, light/dark).
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

        (colour | flags)
            .encode(buf)
            .map_err(|e| e.nested().context("BtnStyle::value"))
    }
}

impl Decode for BtnStyle {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let val = u8::decode(buf).map_err(|e| e.nested().context("BtnStyle::value"))?;

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
    /// Bitwise flags reported for a button click.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Debug, Default, Clone, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Action type for [Bfn].
pub enum BfnType {
    #[default]
    /// Delete one button or a range of buttons.
    DelBtn = 0,

    /// Clear all buttons.
    Clear = 1,

    /// User cleared all buttons.
    UserClear = 2,

    /// User requested buttons.
    BtnRequest = 3,
}

#[derive(Debug, Clone, Default, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Button management command or notification.
///
/// - Deletes or clears buttons, or reports user button actions.
pub struct Bfn {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Action to perform or report.
    pub subt: BfnType,

    /// Target connection (0 = local, 255 = all).
    pub ucid: ConnectionId,

    /// Button id or start of range (for delete actions).
    pub clickid: ClickId,

    /// End of range for delete actions.
    pub clickmax: u8,

    /// Internal button instance flags.
    pub inst: BtnInst,
}

impl_typical_with_request_id!(Bfn);

#[derive(Debug, Clone, PartialEq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Create or update a button.
///
/// - Buttons can include optional captions and input fields.
/// - Position and size are specified in screen-relative units.
pub struct Btn {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection to display the button (0 = local, 255 = all).
    pub ucid: ConnectionId,

    /// Button id (0 to 239).
    pub clickid: ClickId,

    /// Internal button instance flags.
    pub inst: BtnInst,

    /// Button style.
    pub bstyle: BtnStyle,

    /// Max chars permitted for a button with input.
    pub typein: Option<u8>,

    /// Position: left (0-200).
    pub l: u8,

    /// Position: top (0-200).
    pub t: u8,

    /// Position: width (1-200).
    pub w: u8,

    /// Position: height (1-200).
    pub h: u8,

    /// Optional caption displayed before the main text.
    pub caption: Option<String>,

    /// Button text.
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
            let split = if let Some(split) = buf.iter().position(|c| c == &0_u8) {
                split
            } else {
                return Err(DecodeErrorKind::ExpectedNull
                    .context("Btn: Expected caption but found no \0 in text field"));
            };

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
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 200,
                found: self.l as usize,
            }
            .context("Btn::l"));
        }

        if self.t > 200 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 200,
                found: self.t as usize,
            }
            .context("Btn::t"));
        }

        if self.w < 1 || self.w > 200 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 1,
                max: 200,
                found: self.w as usize,
            }
            .context("Btn::w"));
        }

        if self.h < 1 || self.h > 200 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 1,
                max: 200,
                found: self.t as usize,
            }
            .context("Btn::h"));
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
                return Err(insim_core::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max: BTN_TEXT_MAX_LEN - 2,
                    found: caption.len() + text.len(),
                }
                .context("Btn: Caption + text too large"));
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

#[derive(Debug, Clone, Default, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Sent when a user clicks a button.
///
/// - Reports the button id and click modifiers.
pub struct Btc {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,
    /// Connection that clicked the button (0 = local).
    pub ucid: ConnectionId,
    /// Button id that was clicked.
    pub clickid: ClickId,

    /// Internal button instance flags.
    pub inst: BtnInst,

    /// Click modifiers (left/right/ctrl/shift).
    #[insim(pad_after = 1)]
    pub cflags: BtnClickFlags,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Sent when a user types into a text entry button.
///
/// - Includes the input text and the original `typein` limit.
pub struct Btt {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,
    /// Connection that typed into the button (0 = local).
    pub ucid: ConnectionId,

    /// Button id that received input.
    pub clickid: ClickId,

    /// Internal button instance flags.
    pub inst: BtnInst,

    /// Original input limit from [Btn].
    pub typein: Option<u8>,

    /// Entered text.
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
            return Err(insim_core::DecodeErrorKind::OutOfRange {
                min: 0,
                max: 96,
                found: typein as usize,
            }
            .context("Btn::typein"));
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
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 96,
                found: typein as usize,
            }
            .context("Btn::typein"));
        }
        typein.encode(buf)?;
        buf.put_bytes(0, 1);
        self.text.encode_codepage(buf, 96, false)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

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

    #[test]
    fn test_contextual_error() {
        let btn = Btn {
            ..Default::default()
        };

        let mut buf = BytesMut::new();
        let res = btn.encode(&mut buf);

        assert!(res.is_err());
        assert!(matches!(
            res,
            Err(insim_core::encode::EncodeError {
                context: Some(Cow::Borrowed("Btn::w")),
                kind: insim_core::encode::EncodeErrorKind::OutOfRange {
                    min: 1,
                    max: 200,
                    found: 0
                }
            })
        ));
    }
}
