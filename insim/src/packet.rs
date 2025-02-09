//! Contains [crate::Packet] enum

use std::fmt::Debug;

use insim_core::FromToBytes;

use crate::{identifiers::RequestId, insim::*, relay::*};

#[derive(Debug, Clone, from_variants::FromVariants)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[non_exhaustive]
/// Enum representing all possible packets receivable via an Insim connection.
/// Each variant may either be instructional (tell LFS to do something), informational (you are
/// told something about LFS), or both.
pub enum Packet {
    /// Instruction - handshake or init
    Isi(Isi),

    /// Information - version info
    Ver(Ver),

    /// Both - multi-purpose
    Tiny(Tiny),

    /// Both - multi-purpose
    Small(Small),

    /// Information - State info
    Sta(Sta),

    /// Instruction - Single character
    Sch(Sch),

    /// Instruction - State Flags Pack
    Sfp(Sfp),

    /// Both - Set Car Cam
    Scc(Scc),

    /// Both - Camera position pack
    Cpp(Cpp),

    /// Information - Start multiplayer
    Ism(Ism),

    /// Information - Message out
    Mso(Mso),

    /// Information - Hidden /i message
    Iii(Iii),

    /// Instruction - Type a message or /command
    Mst(Mst),

    /// Instruction - Message to connection
    Mtc(Mtc),

    /// Instruction - set screen mode
    Mod(Mod),

    /// Information - Vote notification
    Vtn(Vtn),

    /// Information - Race start
    Rst(Rst),

    /// Information - New connection
    Ncn(Ncn),

    /// Information - Connection left
    Cnl(Cnl),

    /// Information - Connection renamed
    Cpr(Cpr),

    /// Information - New player (player joined)
    Npl(Npl),

    /// Information - Player telepits
    Plp(Plp),

    /// Information - Player left
    Pll(Pll),

    /// Information - Lap time
    Lap(Lap),

    /// Information - Split time
    Spx(Spx),

    /// Information - Pit stop start
    Pit(Pit),

    /// Information - Pit stop finish
    Psf(Psf),

    /// Information - Player entered pit lane
    Pla(Pla),

    /// Information - Camera changed
    Cch(Cch),

    /// Information - Penalty
    Pen(Pen),

    /// Information - Take over
    Toc(Toc),

    /// Information - Flag
    Flg(Flg),

    /// Information - Player flags
    Pfl(Pfl),

    /// Information - Finished race - unverified result
    Fin(Fin),

    /// Information - Verified finish result
    Res(Res),

    /// Both - Player reorder
    Reo(Reo),

    /// Information - Node and lap
    Nlp(Nlp),

    /// Information - Multi-car info
    Mci(Mci),

    /// Instruction - Type a message
    Msx(Msx),

    /// Instruction - Message to local computer
    Msl(Msl),

    /// Information - Car reset
    Crs(Crs),

    /// Both - Delete or receive buttons
    Bfn(Bfn),

    /// Information - AutoX layout info
    Axi(Axi),

    /// Information - Player hit an AutoX object
    Axo(Axo),

    /// Instruction - Show a button
    Btn(Btn),

    /// Information - Button clicked
    Btc(Btc),

    /// Information - Button was typed into
    Btt(Btt),

    /// Both - Replay information
    Rip(Rip),

    /// Both - screenshot
    Ssh(Ssh),

    /// Information - contact between vehicles
    Con(Con),

    /// Information - Object hit
    Obh(Obh),

    /// Information - Hot lap validity violation
    Hlv(Hlv),

    /// Instruction - Restrict player vehicles
    Plc(Plc),

    /// Both - AutoX - multiple object
    Axm(Axm),

    /// Information - Admin command report
    Acr(Acr),

    /// Instruction - Handicap
    Hcp(Hcp),

    /// Information - New connection information
    Nci(Nci),

    /// Instruction - Join reply response
    Jrr(Jrr),

    /// Information - report insim checkpoint/circle
    Uco(Uco),

    /// Instruction - Object control
    Oco(Oco),

    /// Instruction - Multi-purpose, target to connection
    Ttc(Ttc),

    /// Information - Player selected vehicle
    Slc(Slc),

    /// Information - Vehicle changed state
    Csc(Csc),

    /// Information - Connection interface mode
    Cim(Cim),

    /// Both - Set mods a player is allowed
    Mal(Mal),

    /// Both - Set/receive player handicap
    Plh(Plh),

    /// Both - Set/receive player bans
    Ipb(Ipb),

    /// Instruction - Set AI control value
    Aic(Aic),

    /// Information - AI information
    Aii(Aii),

    /// Instruction - Ask the LFS World relay if we are an admin
    RelayArq(Arq),

    /// Information - LFS World relay response if we are an admin
    RelayArp(Arp),

    /// Instruction - Ask the LFS World relay for a list of hosts
    RelayHlr(Hlr),

    /// Information - LFS World relay response to a HostListRequest
    RelayHos(Hos),

    /// Instruction - Ask the LFS World relay to select a host and start relaying Insim packets
    RelaySel(Sel),

    /// Information - LFS World relay error (recoverable)
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

impl FromToBytes for Packet {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let discrimator = u8::from_bytes(buf)?;
        let packet = match discrimator {
            1 => Self::Isi(Isi::from_bytes(buf)?),
            2 => Self::Ver(Ver::from_bytes(buf)?),
            3 => Self::Tiny(Tiny::from_bytes(buf)?),
            4 => Self::Small(Small::from_bytes(buf)?),
            5 => Self::Sta(Sta::from_bytes(buf)?),
            6 => Self::Sch(Sch::from_bytes(buf)?),
            7 => Self::Sfp(Sfp::from_bytes(buf)?),
            8 => Self::Scc(Scc::from_bytes(buf)?),
            9 => Self::Cpp(Cpp::from_bytes(buf)?),
            10 => Self::Ism(Ism::from_bytes(buf)?),
            11 => Self::Mso(Mso::from_bytes(buf)?),
            12 => Self::Iii(Iii::from_bytes(buf)?),
            13 => Self::Mst(Mst::from_bytes(buf)?),
            14 => Self::Mtc(Mtc::from_bytes(buf)?),
            15 => Self::Mod(Mod::from_bytes(buf)?),
            16 => Self::Vtn(Vtn::from_bytes(buf)?),
            17 => Self::Rst(Rst::from_bytes(buf)?),
            18 => Self::Ncn(Ncn::from_bytes(buf)?),
            19 => Self::Cnl(Cnl::from_bytes(buf)?),
            20 => Self::Cpr(Cpr::from_bytes(buf)?),
            21 => Self::Npl(Npl::from_bytes(buf)?),
            22 => Self::Plp(Plp::from_bytes(buf)?),
            23 => Self::Pll(Pll::from_bytes(buf)?),
            24 => Self::Lap(Lap::from_bytes(buf)?),
            25 => Self::Spx(Spx::from_bytes(buf)?),
            26 => Self::Pit(Pit::from_bytes(buf)?),
            27 => Self::Psf(Psf::from_bytes(buf)?),
            28 => Self::Pla(Pla::from_bytes(buf)?),
            29 => Self::Cch(Cch::from_bytes(buf)?),
            30 => Self::Pen(Pen::from_bytes(buf)?),
            31 => Self::Toc(Toc::from_bytes(buf)?),
            32 => Self::Flg(Flg::from_bytes(buf)?),
            33 => Self::Pfl(Pfl::from_bytes(buf)?),
            34 => Self::Fin(Fin::from_bytes(buf)?),
            35 => Self::Res(Res::from_bytes(buf)?),
            36 => Self::Reo(Reo::from_bytes(buf)?),
            37 => Self::Nlp(Nlp::from_bytes(buf)?),
            38 => Self::Mci(Mci::from_bytes(buf)?),
            39 => Self::Msx(Msx::from_bytes(buf)?),
            40 => Self::Msl(Msl::from_bytes(buf)?),
            41 => Self::Crs(Crs::from_bytes(buf)?),
            42 => Self::Bfn(Bfn::from_bytes(buf)?),
            43 => Self::Axi(Axi::from_bytes(buf)?),
            44 => Self::Axo(Axo::from_bytes(buf)?),
            45 => Self::Btn(Btn::from_bytes(buf)?),
            46 => Self::Btc(Btc::from_bytes(buf)?),
            47 => Self::Btt(Btt::from_bytes(buf)?),
            48 => Self::Rip(Rip::from_bytes(buf)?),
            49 => Self::Ssh(Ssh::from_bytes(buf)?),
            50 => Self::Con(Con::from_bytes(buf)?),
            51 => Self::Obh(Obh::from_bytes(buf)?),
            52 => Self::Hlv(Hlv::from_bytes(buf)?),
            53 => Self::Plc(Plc::from_bytes(buf)?),
            54 => Self::Axm(Axm::from_bytes(buf)?),
            55 => Self::Acr(Acr::from_bytes(buf)?),
            56 => Self::Hcp(Hcp::from_bytes(buf)?),
            57 => Self::Nci(Nci::from_bytes(buf)?),
            58 => Self::Jrr(Jrr::from_bytes(buf)?),
            59 => Self::Uco(Uco::from_bytes(buf)?),
            60 => Self::Oco(Oco::from_bytes(buf)?),
            61 => Self::Ttc(Ttc::from_bytes(buf)?),
            62 => Self::Slc(Slc::from_bytes(buf)?),
            63 => Self::Csc(Csc::from_bytes(buf)?),
            64 => Self::Cim(Cim::from_bytes(buf)?),
            65 => Self::Mal(Mal::from_bytes(buf)?),
            66 => Self::Plh(Plh::from_bytes(buf)?),
            67 => Self::Ipb(Ipb::from_bytes(buf)?),
            68 => Self::Aic(Aic::from_bytes(buf)?),
            69 => Self::Aii(Aii::from_bytes(buf)?),

            250 => Self::RelayArq(Arq::from_bytes(buf)?),
            251 => Self::RelayArp(Arp::from_bytes(buf)?),
            252 => Self::RelayHlr(Hlr::from_bytes(buf)?),
            253 => Self::RelayHos(Hos::from_bytes(buf)?),
            254 => Self::RelaySel(Sel::from_bytes(buf)?),
            255 => Self::RelayErr(Error::from_bytes(buf)?),

            i => return Err(insim_core::Error::NoVariantMatch { found: i })
        };

        Ok(packet)
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        todo!()
    }
}
