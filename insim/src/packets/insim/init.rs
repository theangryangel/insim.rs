use std::time::Duration;

use insim_core::{identifiers::RequestId, prelude::*, ser::Limit, DecodableError, EncodableError};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    /// Flags for the [Init] packet flags field.
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct InitFlags: u16 {
        //RES0 => (1 << 0),	// bit  0: spare
        //RES_1 => (1 << 1),	// bit  1: spare
         const LOCAL = (1 << 2);	// bit  2: guest or single player
         const MSO_COLS = (1 << 3);	// bit  3: keep colours in MSO text
         const NLP = (1 << 4);	// bit  4: receive NLP packets
         const MCI = (1 << 5);	// bit  5: receive MCI packets
         const CON = (1 << 6);	// bit  6: receive CON packets
         const OBH = (1 << 7);	// bit  7: receive OBH packets
         const HLV = (1 << 8);	// bit  8: receive HLV packets
         const AXM_LOAD = (1 << 9);	// bit  9: receive AXM when loading a layout
         const AXM_EDIT = (1 << 10);	// bit 10: receive AXM when changing objects
         const REQ_JOIN = (1 << 11);	// bit 11: process join requests
    }
}

impl InitFlags {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

impl Encodable for InitFlags {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), insim_core::EncodableError> {
        self.bits().encode(buf, limit)?;
        Ok(())
    }
}

impl Decodable for InitFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError> {
        Ok(Self::from_bits_truncate(u16::decode(buf, limit)?))
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Insim Init, or handshake packet.
/// Required to be sent to the server before any other packets.
pub struct Init {
    /// When set to a non-zero value the server will send a [Version](super::Version) packet in response.
    ///packet in response.
    pub reqi: RequestId,

    // we do not support this feature, using pad_bytes_before
    // on flags to mask it.
    //pub udpport: u16,
    /// Options for the Insim Connection. See [InitFlags] for more information.
    pub flags: InitFlags,

    /// Protocol version of Insim you wish to use.
    pub version: u8,

    /// Messages typed with this prefix will be sent to your InSim program
    /// on the host (in IS_MSO) and not displayed on anyone's screen.
    /// This should be a single ascii character. i.e. '!'.
    pub prefix: char,

    /// Time in between each [Nlp](super::Nlp) or [Mci](super::Mci) packet when set to a non-zero value and
    /// the relevant flags are set.
    pub interval: Duration,

    /// Administrative password.
    pub password: String,

    /// Name of the program.
    pub name: String,
}

impl Encodable for Init {
    fn encode(&self, buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        // impl Encodable by hand because the interval on ISI is u16 rather than u32

        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "ISI does not support a limit: {limit:?}",
            )));
        }

        self.reqi.encode(buf, None)?;

        // pad_after reqi
        buf.put_bytes(0, 1);

        // mask out udpport
        buf.put_bytes(0, 2);

        self.version.encode(buf, None)?;

        (self.prefix as u8).encode(buf, None)?;

        (self.interval.as_millis() as u16).encode(buf, None)?;

        self.password.encode(buf, Some(Limit::Bytes(16)))?;

        self.name.encode(buf, Some(Limit::Bytes(16)))?;

        Ok(())
    }
}

impl Decodable for Init {
    fn decode(buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError>
    where
        Self: Default,
    {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!(
                "ISI does not support a limit: {:?}",
                limit
            )));
        }

        let mut data = Self {
            reqi: RequestId::decode(buf, None)?,
            ..Default::default()
        };

        // pad bytes_after reqi
        buf.advance(1);

        // skip over udpport
        buf.advance(2);

        data.flags = InitFlags::decode(buf, None)?;
        data.version = u8::decode(buf, None)?;
        data.prefix = u8::decode(buf, None)? as char;
        data.interval = Duration::from_millis(u16::decode(buf, None)?.into());
        data.password = String::decode(buf, Some(Limit::Bytes(16)))?;
        data.name = String::decode(buf, Some(Limit::Bytes(16)))?;

        Ok(data)
    }
}