//! Contains [crate::Packet] enum

use std::fmt::Debug;

use insim_core::binrw::{self, binrw};

use crate::{identifiers::RequestId, insim::*, relay::*};

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, from_variants::FromVariants)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
/// Enum representing all possible packets receivable via an Insim connection.
/// Each variant may either be instructional (tell LFS to do something), informational (you are
/// told something about LFS), or both.
pub enum Packet {
    /// Instruction - handshake or init
    #[brw(magic = 1u8)]
    Isi(Isi),

    /// Information - version info
    #[brw(magic = 2u8)]
    Ver(Ver),

    /// Both - multi-purpose
    #[brw(magic = 3u8)]
    Tiny(Tiny),

    /// Both - multi-purpose
    #[brw(magic = 4u8)]
    Small(Small),

    /// Information - State info
    #[brw(magic = 5u8)]
    Sta(Sta),

    /// Instruction - Single character
    #[brw(magic = 6u8)]
    Sch(Sch),

    /// Instruction - State Flags Pack
    #[brw(magic = 7u8)]
    Sfp(Sfp),

    /// Both - Set Car Cam
    #[brw(magic = 8u8)]
    Scc(Scc),

    /// Both - Camera position pack
    #[brw(magic = 9u8)]
    Cpp(Cpp),

    /// Information - Start multiplayer
    #[brw(magic = 10u8)]
    Ism(Ism),

    /// Information - Message out
    #[brw(magic = 11u8)]
    Mso(Mso),

    /// Information - Hidden /i message
    #[brw(magic = 12u8)]
    Iii(Iii),

    /// Instruction - Type a message or /command
    #[brw(magic = 13u8)]
    Mst(Mst),

    /// Instruction - Message to connection
    #[brw(magic = 14u8)]
    Mtc(Mtc),

    /// Instruction - set screen mode
    #[brw(magic = 15u8)]
    Mod(Mod),

    /// Information - Vote notification
    #[brw(magic = 16u8)]
    Vtn(Vtn),

    /// Information - Race start
    #[brw(magic = 17u8)]
    Rst(Rst),

    /// Information - New connection
    #[brw(magic = 18u8)]
    Ncn(Ncn),

    /// Information - Connection left
    #[brw(magic = 19u8)]
    Cnl(Cnl),

    /// Information - Connection renamed
    #[brw(magic = 20u8)]
    Cpr(Cpr),

    /// Information - New player (player joined)
    #[brw(magic = 21u8)]
    Npl(Npl),

    /// Information - Player telepits
    #[brw(magic = 22u8)]
    Plp(Plp),

    /// Information - Player left
    #[brw(magic = 23u8)]
    Pll(Pll),

    /// Information - Lap time
    #[brw(magic = 24u8)]
    Lap(Lap),

    /// Information - Split time
    #[brw(magic = 25u8)]
    Spx(Spx),

    /// Information - Pit stop start
    #[brw(magic = 26u8)]
    Pit(Pit),

    /// Information - Pit stop finish
    #[brw(magic = 27u8)]
    Psf(Psf),

    /// Information - Player entered pit lane
    #[brw(magic = 28u8)]
    Pla(Pla),

    /// Information - Camera changed
    #[brw(magic = 29u8)]
    Cch(Cch),

    /// Information - Penalty
    #[brw(magic = 30u8)]
    Pen(Pen),

    /// Information - Take over
    #[brw(magic = 31u8)]
    Toc(Toc),

    /// Information - Flag
    #[brw(magic = 32u8)]
    Flg(Flg),

    /// Information - Player flags
    #[brw(magic = 33u8)]
    Pfl(Pfl),

    /// Information - Finished race - unverified result
    #[brw(magic = 34u8)]
    Fin(Fin),

    /// Information - Verified finish result
    #[brw(magic = 35u8)]
    Res(Res),

    /// Both - Player reorder
    #[brw(magic = 36u8)]
    Reo(Reo),

    /// Information - Node and lap
    #[brw(magic = 37u8)]
    Nlp(Nlp),

    /// Information - Multi-car info
    #[brw(magic = 38u8)]
    Mci(Mci),

    /// Instruction - Type a message
    #[brw(magic = 39u8)]
    Msx(Msx),

    /// Instruction - Message to local computer
    #[brw(magic = 40u8)]
    Msl(Msl),

    /// Information - Car reset
    #[brw(magic = 41u8)]
    Crs(Crs),

    /// Both - Delete or receive buttons
    #[brw(magic = 42u8)]
    Bfn(Bfn),

    /// Information - AutoX layout info
    #[brw(magic = 43u8)]
    Axi(Axi),

    /// Information - Player hit an AutoX object
    #[brw(magic = 44u8)]
    Axo(Axo),

    /// Instruction - Show a button
    #[brw(magic = 45u8)]
    Btn(Btn),

    /// Information - Button clicked
    #[brw(magic = 46u8)]
    Btc(Btc),

    /// Information - Button was typed into
    #[brw(magic = 47u8)]
    Btt(Btt),

    /// Both - Replay information
    #[brw(magic = 48u8)]
    Rip(Rip),

    /// Both - screenshot
    #[brw(magic = 49u8)]
    Ssh(Ssh),

    /// Information - contact between vehicles
    #[brw(magic = 50u8)]
    Con(Con),

    /// Information - Object hit
    #[brw(magic = 51u8)]
    Obh(Obh),

    /// Information - Hot lap validity violation
    #[brw(magic = 52u8)]
    Hlv(Hlv),

    /// Instruction - Restrict player vehicles
    #[brw(magic = 53u8)]
    Plc(Plc),

    /// Both - AutoX - multiple object
    #[brw(magic = 54u8)]
    Axm(Axm),

    /// Information - Admin command report
    #[brw(magic = 55u8)]
    Acr(Acr),

    /// Instruction - Handicap
    #[brw(magic = 56u8)]
    Hcp(Hcp),

    /// Information - New connection information
    #[brw(magic = 57u8)]
    Nci(Nci),

    /// Instruction - Join reply response
    #[brw(magic = 58u8)]
    Jrr(Jrr),

    /// Information - report insim checkpoint/circle
    #[brw(magic = 59u8)]
    Uco(Uco),

    /// Instruction - Object control
    #[brw(magic = 60u8)]
    Oco(Oco),

    /// Instruction - Multi-purpose, target to connection
    #[brw(magic = 61u8)]
    Ttc(Ttc),

    /// Information - Player selected vehicle
    #[brw(magic = 62u8)]
    Slc(Slc),

    /// Information - Vehicle changed state
    #[brw(magic = 63u8)]
    Csc(Csc),

    /// Information - Connection interface mode
    #[brw(magic = 64u8)]
    Cim(Cim),

    /// Both - Set mods a player is allowed
    #[brw(magic = 65u8)]
    Mal(Mal),

    /// Both - Set/receive player handicap
    #[brw(magic = 66u8)]
    Plh(Plh),

    /// Both - Set/receive player bans
    #[brw(magic = 67u8)]
    Ipb(Ipb),

    /// Instruction - Set AI control value
    #[brw(magic = 68u8)]
    Aic(Aic),

    /// Information - AI information
    #[brw(magic = 69u8)]
    Aii(Aii),

    /// Instruction - Ask the LFS World relay if we are an admin
    #[brw(magic = 250u8)]
    RelayArq(Arq),

    /// Information - LFS World relay response if we are an admin
    #[brw(magic = 251u8)]
    RelayArp(Arp),

    /// Instruction - Ask the LFS World relay for a list of hosts
    #[brw(magic = 252u8)]
    RelayHlr(Hlr),

    /// Information - LFS World relay response to a HostListRequest
    #[brw(magic = 253u8)]
    RelayHos(Hos),

    /// Instruction - Ask the LFS World relay to select a host and start relaying Insim packets
    #[brw(magic = 254u8)]
    RelaySel(Sel),

    /// Information - LFS World relay error (recoverable)
    #[brw(magic = 255u8)]
    RelayErr(Error),
}

impl Default for Packet {
    fn default() -> Self {
        Self::Tiny(Tiny::default())
    }
}

impl Packet {
    /// Hint at the possible *minimum* size of a packet, so that when we encode it, it can pre-allocate a
    /// ballpark buffer.
    /// It must not be trusted. An incorrect implementation of size_hint() should not lead to memory safety violations.
    pub fn size_hint(&self) -> usize {
        // TODO: For some of these packets we can be more intelligent.
        // i.e. see RelayHostList
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
            Packet::Res(_) => 86,
            Packet::Reo(_) => 44,
            Packet::Nlp(_) => 10,
            Packet::Mci(_) => 32,
            Packet::Msx(_) => 100,
            Packet::Msl(_) => 132,
            Packet::Crs(_) => 4,
            Packet::Bfn(_) => 8,
            Packet::Axi(_) => 40,
            Packet::Axo(_) => 4,
            Packet::Btn(_) => 16,
            Packet::Btc(_) => 8,
            Packet::Btt(_) => 104,
            Packet::Rip(_) => 80,
            Packet::Ssh(_) => 40,
            Packet::Con(_) => 40,
            Packet::Obh(_) => 24,
            Packet::Hlv(_) => 16,
            Packet::Plc(_) => 8,
            Packet::Axm(_) => 16,
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
            Packet::Mal(_) => 12,
            Packet::Aic(i) => 4 + (i.inputs.len() * 4),
            Packet::Aii(_) => 96,
            Packet::RelayHos(i) => 4 + (i.hinfo.len() * 40),
            Packet::RelaySel(_) => 68,
            _ => {
                // a sensible default for everything else
                4
            },
        }
    }

    /// Does this packet indicate that we should send a ping reply back?
    #[tracing::instrument]
    pub fn maybe_pong(&self) -> Option<Self> {
        match self {
            Packet::Tiny(Tiny {
                subt: TinyType::None,
                reqi: RequestId(0),
            }) => Some(Self::Tiny(Tiny {
                reqi: RequestId(0),
                subt: TinyType::None,
            })),
            _ => None,
        }
    }

    /// Does this packet contain the version of the Insim server, and can we verify it?
    #[tracing::instrument]
    pub fn maybe_verify_version(&self) -> crate::result::Result<bool> {
        match self {
            Packet::Ver(Ver { insimver, .. }) => {
                if *insimver != crate::VERSION {
                    return Err(crate::error::Error::IncompatibleVersion(*insimver));
                }

                Ok(true)
            },
            _ => Ok(false),
        }
    }
}

pub trait WithRequestId
where
    Self: std::fmt::Debug,
{
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug;
}

impl WithRequestId for Packet {
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        mut self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug {
        match &mut self {
            Packet::Isi(i) => i.reqi = reqi.into(),
            Packet::Ver(i) => i.reqi = reqi.into(),
            Packet::Tiny(i) => i.reqi = reqi.into(),
            Packet::Small(i) => i.reqi = reqi.into(),
            Packet::Sta(i) => i.reqi = reqi.into(),
            Packet::Sch(i) => i.reqi = reqi.into(),
            Packet::Sfp(i) => i.reqi = reqi.into(),
            Packet::Scc(i) => i.reqi = reqi.into(),
            Packet::Cpp(i) => i.reqi = reqi.into(),
            Packet::Ism(i) => i.reqi = reqi.into(),
            Packet::Mso(i) => i.reqi = reqi.into(),
            Packet::Iii(i) => i.reqi = reqi.into(),
            Packet::Mst(i) => i.reqi = reqi.into(),
            Packet::Mtc(i) => i.reqi = reqi.into(),
            Packet::Mod(i) => i.reqi = reqi.into(),
            Packet::Vtn(i) => i.reqi = reqi.into(),
            Packet::Rst(i) => i.reqi = reqi.into(),
            Packet::Ncn(i) => i.reqi = reqi.into(),
            Packet::Cnl(i) => i.reqi = reqi.into(),
            Packet::Cpr(i) => i.reqi = reqi.into(),
            Packet::Npl(i) => i.reqi = reqi.into(),
            Packet::Plp(i) => i.reqi = reqi.into(),
            Packet::Pll(i) => i.reqi = reqi.into(),
            Packet::Lap(i) => i.reqi = reqi.into(),
            Packet::Spx(i) => i.reqi = reqi.into(),
            Packet::Pit(i) => i.reqi = reqi.into(),
            Packet::Psf(i) => i.reqi = reqi.into(),
            Packet::Pla(i) => i.reqi = reqi.into(),
            Packet::Cch(i) => i.reqi = reqi.into(),
            Packet::Pen(i) => i.reqi = reqi.into(),
            Packet::Toc(i) => i.reqi = reqi.into(),
            Packet::Flg(i) => i.reqi = reqi.into(),
            Packet::Pfl(i) => i.reqi = reqi.into(),
            Packet::Fin(i) => i.reqi = reqi.into(),
            Packet::Res(i) => i.reqi = reqi.into(),
            Packet::Reo(i) => i.reqi = reqi.into(),
            Packet::Nlp(i) => i.reqi = reqi.into(),
            Packet::Mci(i) => i.reqi = reqi.into(),
            Packet::Msx(i) => i.reqi = reqi.into(),
            Packet::Msl(i) => i.reqi = reqi.into(),
            Packet::Crs(i) => i.reqi = reqi.into(),
            Packet::Bfn(i) => i.reqi = reqi.into(),
            Packet::Axi(i) => i.reqi = reqi.into(),
            Packet::Axo(i) => i.reqi = reqi.into(),
            Packet::Btn(i) => i.reqi = reqi.into(),
            Packet::Btc(i) => i.reqi = reqi.into(),
            Packet::Btt(i) => i.reqi = reqi.into(),
            Packet::Rip(i) => i.reqi = reqi.into(),
            Packet::Ssh(i) => i.reqi = reqi.into(),
            Packet::Con(i) => i.reqi = reqi.into(),
            Packet::Obh(i) => i.reqi = reqi.into(),
            Packet::Hlv(i) => i.reqi = reqi.into(),
            Packet::Plc(i) => i.reqi = reqi.into(),
            Packet::Axm(i) => i.reqi = reqi.into(),
            Packet::Acr(i) => i.reqi = reqi.into(),
            Packet::Hcp(i) => i.reqi = reqi.into(),
            Packet::Nci(i) => i.reqi = reqi.into(),
            Packet::Jrr(i) => i.reqi = reqi.into(),
            Packet::Uco(i) => i.reqi = reqi.into(),
            Packet::Oco(i) => i.reqi = reqi.into(),
            Packet::Ttc(i) => i.reqi = reqi.into(),
            Packet::Slc(i) => i.reqi = reqi.into(),
            Packet::Csc(i) => i.reqi = reqi.into(),
            Packet::Cim(i) => i.reqi = reqi.into(),
            Packet::Mal(i) => i.reqi = reqi.into(),
            Packet::Plh(i) => i.reqi = reqi.into(),
            Packet::Ipb(i) => i.reqi = reqi.into(),
            Packet::Aic(i) => i.reqi = reqi.into(),
            Packet::Aii(i) => i.reqi = reqi.into(),
            Packet::RelayArq(i) => i.reqi = reqi.into(),
            Packet::RelayArp(i) => i.reqi = reqi.into(),
            Packet::RelayHlr(i) => i.reqi = reqi.into(),
            Packet::RelayHos(i) => i.reqi = reqi.into(),
            Packet::RelaySel(i) => i.reqi = reqi.into(),
            Packet::RelayErr(i) => i.reqi = reqi.into(),
        };
        self
    }
}
