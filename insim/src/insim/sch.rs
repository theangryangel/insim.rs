use bitflags::bitflags;
use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    ReadWriteBuf,
};

use crate::identifiers::RequestId;

bitflags! {
    /// Bitwise flags used within the [Sch] packet
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct SchFlags: u8 {
        /// Shift
        const SHIFT = (1 << 0);

        /// Ctrl
        const CTRL = (1 << 1);
    }
}

generate_bitflag_helpers! {
    SchFlags,
    pub shift => SHIFT,
    pub ctrl => CTRL
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send Single Character
pub struct Sch {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Character
    #[bw(map = |&x| x as u8)]
    #[br(map = |x: u8| x as char)]
    pub charb: char,

    /// Character modifiers/flags
    #[brw(pad_after = 2)]
    pub flags: SchFlags,
}

impl ReadWriteBuf for Sch {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        buf.advance(1);
        let charb = char::read_buf(buf)?;
        let flags = u8::read_buf(buf)?;
        let flags = SchFlags::from_bits_truncate(flags);
        buf.advance(2);

        Ok(Self { reqi, charb, flags })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        buf.put_bytes(0, 1);
        self.charb.write_buf(buf)?;
        let flags = self.flags.bits();
        flags.write_buf(buf)?;
        buf.put_bytes(0, 2);
        Ok(())
    }
}

impl_typical_with_request_id!(Sch);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sch() {
        assert_from_to_bytes!(
            Sch,
            [
                1,  // ReqI
                0,  // Zero
                97, // CharB
                1,  // Flags
                0,  // Spare2
                0,  // Spare3
            ],
            |parsed: Sch| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.charb, 'a');
            }
        );
    }
}
