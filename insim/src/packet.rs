//! Contains [crate::Packet] enum

use std::fmt::Debug;

use insim_core::{Decode, Encode};

use crate::insim::*;

/// Helper method to assist in converting the inner part of a [Packet] variant into [Packet] with a
/// request identifier set. Mostly useful for things like [Packet::Tiny].
pub trait WithRequestId
where
    Self: std::fmt::Debug,
{
    /// Convert something into a Packet with a request identifier
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug;
}

macro_rules! define_packet {
    (
        $(
            $(#[$variant_attr:meta])*
            $variant:ident = $disc:expr
        ),* $(,)?
    ) => {
        /// Enum representing all possible packets receivable via an Insim connection.
        /// Each variant may either be instructional (tell LFS to do something), informational (you are
        /// told something about LFS), or both.
        #[derive(Debug, Clone, from_variants::FromVariants)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(tag = "type"))]
        #[non_exhaustive]
        pub enum Packet {
            $(
                $(#[$variant_attr])*
                $variant($variant),
            )*
        }

        impl WithRequestId for Packet {
            fn with_request_id<R: Into<crate::identifiers::RequestId>>(
                mut self,
                reqi: R,
            ) -> impl Into<crate::Packet> + std::fmt::Debug {
                let reqi = reqi.into();
                match &mut self {
                    $(
                        Self::$variant(inner) => inner.reqi = reqi,
                    )*
                }
                self
            }
        }

        impl Decode for Packet {
            fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
                let discriminator = u8::decode(buf)?;
                match discriminator {
                    $(
                        $disc => Ok(Self::$variant(<$variant>::decode(buf)?)),
                    )*
                    i => {
                        return Err(
                            insim_core::DecodeErrorKind::NoVariantMatch { found: i.into() }
                                .context("Unknown packet identifier"),
                        );
                    },
                }
            }
        }

        impl Encode for Packet {
            fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
                match self {
                    $(
                        Self::$variant(inner) => {
                            ($disc as u8).encode(buf)?;
                            inner.encode(buf)?;
                        },
                    )*
                }
                Ok(())
            }
        }
    };
}

define_packet! (
    /// Instruction - handshake or init
    Isi = 1,
    /// Information - version info
    Ver = 2,
    /// Both - multi-purpose
    Tiny = 3,
    /// Both - multi-purpose
    Small = 4,
    /// Information - State info
    Sta = 5,
    /// Instruction - Single character
    Sch = 6,
    /// Instruction - State Flags Pack
    Sfp = 7,
    /// Both - Set Car Cam
    Scc = 8,
    /// Both - Camera position pack
    Cpp = 9,
    /// Information - Start multiplayer
    Ism = 10,
    /// Information - Message out
    Mso = 11,
    /// Information - Hidden /i message
    Iii = 12,
    /// Instruction - Type a message or /command
    Mst = 13,
    /// Instruction - Message to connection
    Mtc = 14,
    /// Instruction - set screen mode
    Mod = 15,
    /// Information - Vote notification
    Vtn = 16,
    /// Information - Race start
    Rst = 17,
    /// Information - New connection
    Ncn = 18,
    /// Information - Connection left
    Cnl = 19,
    /// Information - Connection renamed
    Cpr = 20,
    /// Information - New player (player joined)
    Npl = 21,
    /// Information - Player telepits
    Plp = 22,
    /// Information - Player left
    Pll = 23,
    /// Information - Lap time
    Lap = 24,
    /// Information - Split time
    Spx = 25,
    /// Information - Pit stop start
    Pit = 26,
    /// Information - Pit stop finish
    Psf = 27,
    /// Information - Player entered pit lane
    Pla = 28,
    /// Information - Camera changed
    Cch = 29,
    /// Information - Penalty
    Pen = 30,
    /// Information - Take over
    Toc = 31,
    /// Information - Flag
    Flg = 32,
    /// Information - Player flags
    Pfl = 33,
    /// Information - Finished race - unverified result
    Fin = 34,
    /// Information - Verified finish result
    Res = 35,
    /// Both - Player reorder
    Reo = 36,
    /// Information - Node and lap
    Nlp = 37,
    /// Information - Multi-car info
    Mci = 38,
    /// Instruction - Type a message
    Msx = 39,
    /// Instruction - Message to local computer
    Msl = 40,
    /// Information - Car reset
    Crs = 41,
    /// Both - Delete or receive buttons
    Bfn = 42,
    /// Information - AutoX layout info
    Axi = 43,
    /// Information - Player hit an AutoX object
    Axo = 44,
    /// Instruction - Show a button
    Btn = 45,
    /// Information - Button clicked
    Btc = 46,
    /// Information - Button was typed into
    Btt = 47,
    /// Both - Replay information
    Rip = 48,
    /// Both - screenshot
    Ssh = 49,
    /// Information - contact between vehicles
    Con = 50,
    /// Information - Object hit
    Obh = 51,
    /// Information - Hot lap validity violation
    Hlv = 52,
    /// Instruction - Restrict player vehicles
    Plc = 53,
    /// Both - AutoX - multiple object
    Axm = 54,
    /// Information - Admin command report
    Acr = 55,
    /// Instruction - Handicap
    Hcp = 56,
    /// Information - New connection information
    Nci = 57,
    /// Instruction - Join reply response
    Jrr = 58,
    /// Information - report insim checkpoint/circle
    Uco = 59,
    /// Instruction - Object control
    Oco = 60,
    /// Instruction - Multi-purpose, target to connection
    Ttc = 61,
    /// Information - Player selected vehicle
    Slc = 62,
    /// Information - Vehicle changed state
    Csc = 63,
    /// Information - Connection interface mode
    Cim = 64,
    /// Both - Set mods a player is allowed
    Mal = 65,
    /// Both - Set/receive player handicap
    Plh = 66,
    /// Both - Set/receive player bans
    Ipb = 67,
    /// Instruction - Set AI control value
    Aic = 68,
    /// Information - AI information
    Aii = 69
);

impl Default for Packet {
    /// A Tiny with Type of None was selected as the default as it's a non-damaging packet
    /// (keepalive)
    fn default() -> Self {
        TinyType::None.into()
    }
}

impl Packet {
    /// Hint at the possible *minimum* size of a packet, so that when we encode it, it can pre-allocate a
    /// ballpark buffer.
    /// It must not be trusted. An incorrect implementation of size_hint() should not lead to memory safety violations.
    pub fn size_hint(&self) -> usize {
        // TODO: For some of these packets we can be more intelligent.
        match self {
            Packet::Isi(_) => 44,
            Packet::Ver(_) => 20,
            Packet::Small(_) => 8,
            Packet::Sta(_) => 28,
            Packet::Sch(_) => 8,
            Packet::Sfp(_) => 8,
            Packet::Scc(_) => 8,
            Packet::Cpp(_) => 32,
            Packet::Ism(_) => 40,
            Packet::Mso(_) => 12,
            Packet::Iii(_) => 12,
            Packet::Mst(_) => 68,
            Packet::Mtc(_) => 12,
            Packet::Mod(_) => 20,
            Packet::Vtn(_) => 8,
            Packet::Rst(_) => 28,
            Packet::Ncn(_) => 56,
            Packet::Cnl(_) => 8,
            Packet::Cpr(_) => 36,
            Packet::Npl(_) => 76,
            Packet::Lap(_) => 20,
            Packet::Spx(_) => 16,
            Packet::Pit(_) => 24,
            Packet::Psf(_) => 12,
            Packet::Pla(_) => 8,
            Packet::Cch(_) => 8,
            Packet::Pen(_) => 8,
            Packet::Toc(_) => 8,
            Packet::Flg(_) => 8,
            Packet::Pfl(_) => 8,
            Packet::Fin(_) => 20,
            Packet::Res(_) => 84,
            Packet::Reo(_) => 52,
            Packet::Nlp(n) => {
                let count = n.info.len();
                4 + (count * 6)
            },
            Packet::Mci(m) => 4 + (m.info.len() * 28),
            Packet::Msx(_) => 100,
            Packet::Msl(_) => 132,
            Packet::Bfn(_) => 8,
            Packet::Axi(_) => 40,
            Packet::Btn(_) => 16,
            Packet::Btc(_) => 8,
            Packet::Btt(_) => 104,
            Packet::Rip(_) => 80,
            Packet::Ssh(_) => 40,
            Packet::Con(_) => 44,
            Packet::Obh(_) => 28,
            Packet::Hlv(_) => 20,
            Packet::Plc(_) => 8,
            Packet::Axm(a) => 8 + (a.info.len() * 8),
            Packet::Acr(_) => 12,
            Packet::Hcp(_) => 68,
            Packet::Nci(_) => 16,
            Packet::Jrr(_) => 16,
            Packet::Uco(_) => 28,
            Packet::Oco(_) => 8,
            Packet::Ttc(_) => 8,
            Packet::Slc(_) => 8,
            Packet::Csc(_) => 20,
            Packet::Cim(_) => 8,
            Packet::Mal(m) => 8 + (m.len() * 4),
            Packet::Plh(p) => 4 + (p.hcaps.len() * 4),
            Packet::Ipb(i) => 8 + (i.len() * 4),
            Packet::Aic(i) => 4 + (i.inputs.len() * 4),
            Packet::Aii(_) => 96,
            _ => 4, // a sensible default for everything else
        }
    }
}

#[cfg(test)]
mod test {
    use super::Packet;
    fn assert_send<T: Send>() {}

    #[test]
    fn ensure_packet_is_send() {
        assert_send::<Packet>();
    }
}
