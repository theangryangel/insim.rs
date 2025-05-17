//! Contains [crate::Packet] enum

use std::fmt::Debug;

use insim_core::{Decode, Encode};

use crate::insim::*;
#[cfg(feature = "relay")]
use crate::relay::*;

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

    #[cfg(feature = "relay")]
    /// Instruction - Ask the LFS World relay if we are an admin
    RelayArq(Arq),

    #[cfg(feature = "relay")]
    /// Information - LFS World relay response if we are an admin
    RelayArp(Arp),

    #[cfg(feature = "relay")]
    /// Instruction - Ask the LFS World relay for a list of hosts
    RelayHlr(Hlr),

    #[cfg(feature = "relay")]
    /// Information - LFS World relay response to a HostListRequest
    RelayHos(Hos),

    #[cfg(feature = "relay")]
    /// Instruction - Ask the LFS World relay to select a host and start relaying Insim packets
    RelaySel(Sel),

    #[cfg(feature = "relay")]
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
            #[cfg(feature = "relay")]
            Packet::RelayHos(i) => 4 + (i.hinfo.len() * 40),
            #[cfg(feature = "relay")]
            Packet::RelaySel(_) => 68,
            _ => {
                // a sensible default for everything else
                4
            },
        }
    }
}

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
            #[cfg(feature = "relay")]
            Packet::RelayArq(i) => i.reqi = reqi.into(),
            #[cfg(feature = "relay")]
            Packet::RelayArp(i) => i.reqi = reqi.into(),
            #[cfg(feature = "relay")]
            Packet::RelayHlr(i) => i.reqi = reqi.into(),
            #[cfg(feature = "relay")]
            Packet::RelayHos(i) => i.reqi = reqi.into(),
            #[cfg(feature = "relay")]
            Packet::RelaySel(i) => i.reqi = reqi.into(),
            #[cfg(feature = "relay")]
            Packet::RelayErr(i) => i.reqi = reqi.into(),
        };
        self
    }
}

impl Decode for Packet {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let discrimator = u8::decode(buf)?;
        let packet = match discrimator {
            1 => Self::Isi(Isi::decode(buf)?),
            2 => Self::Ver(Ver::decode(buf)?),
            3 => Self::Tiny(Tiny::decode(buf)?),
            4 => Self::Small(Small::decode(buf)?),
            5 => Self::Sta(Sta::decode(buf)?),
            6 => Self::Sch(Sch::decode(buf)?),
            7 => Self::Sfp(Sfp::decode(buf)?),
            8 => Self::Scc(Scc::decode(buf)?),
            9 => Self::Cpp(Cpp::decode(buf)?),
            10 => Self::Ism(Ism::decode(buf)?),
            11 => Self::Mso(Mso::decode(buf)?),
            12 => Self::Iii(Iii::decode(buf)?),
            13 => Self::Mst(Mst::decode(buf)?),
            14 => Self::Mtc(Mtc::decode(buf)?),
            15 => Self::Mod(Mod::decode(buf)?),
            16 => Self::Vtn(Vtn::decode(buf)?),
            17 => Self::Rst(Rst::decode(buf)?),
            18 => Self::Ncn(Ncn::decode(buf)?),
            19 => Self::Cnl(Cnl::decode(buf)?),
            20 => Self::Cpr(Cpr::decode(buf)?),
            21 => Self::Npl(Npl::decode(buf)?),
            22 => Self::Plp(Plp::decode(buf)?),
            23 => Self::Pll(Pll::decode(buf)?),
            24 => Self::Lap(Lap::decode(buf)?),
            25 => Self::Spx(Spx::decode(buf)?),
            26 => Self::Pit(Pit::decode(buf)?),
            27 => Self::Psf(Psf::decode(buf)?),
            28 => Self::Pla(Pla::decode(buf)?),
            29 => Self::Cch(Cch::decode(buf)?),
            30 => Self::Pen(Pen::decode(buf)?),
            31 => Self::Toc(Toc::decode(buf)?),
            32 => Self::Flg(Flg::decode(buf)?),
            33 => Self::Pfl(Pfl::decode(buf)?),
            34 => Self::Fin(Fin::decode(buf)?),
            35 => Self::Res(Res::decode(buf)?),
            36 => Self::Reo(Reo::decode(buf)?),
            37 => Self::Nlp(Nlp::decode(buf)?),
            38 => Self::Mci(Mci::decode(buf)?),
            39 => Self::Msx(Msx::decode(buf)?),
            40 => Self::Msl(Msl::decode(buf)?),
            41 => Self::Crs(Crs::decode(buf)?),
            42 => Self::Bfn(Bfn::decode(buf)?),
            43 => Self::Axi(Axi::decode(buf)?),
            44 => Self::Axo(Axo::decode(buf)?),
            45 => Self::Btn(Btn::decode(buf)?),
            46 => Self::Btc(Btc::decode(buf)?),
            47 => Self::Btt(Btt::decode(buf)?),
            48 => Self::Rip(Rip::decode(buf)?),
            49 => Self::Ssh(Ssh::decode(buf)?),
            50 => Self::Con(Con::decode(buf)?),
            51 => Self::Obh(Obh::decode(buf)?),
            52 => Self::Hlv(Hlv::decode(buf)?),
            53 => Self::Plc(Plc::decode(buf)?),
            54 => Self::Axm(Axm::decode(buf)?),
            55 => Self::Acr(Acr::decode(buf)?),
            56 => Self::Hcp(Hcp::decode(buf)?),
            57 => Self::Nci(Nci::decode(buf)?),
            58 => Self::Jrr(Jrr::decode(buf)?),
            59 => Self::Uco(Uco::decode(buf)?),
            60 => Self::Oco(Oco::decode(buf)?),
            61 => Self::Ttc(Ttc::decode(buf)?),
            62 => Self::Slc(Slc::decode(buf)?),
            63 => Self::Csc(Csc::decode(buf)?),
            64 => Self::Cim(Cim::decode(buf)?),
            65 => Self::Mal(Mal::decode(buf)?),
            66 => Self::Plh(Plh::decode(buf)?),
            67 => Self::Ipb(Ipb::decode(buf)?),
            68 => Self::Aic(Aic::decode(buf)?),
            69 => Self::Aii(Aii::decode(buf)?),
            #[cfg(feature = "relay")]
            250 => Self::RelayArq(Arq::decode(buf)?),
            #[cfg(feature = "relay")]
            251 => Self::RelayArp(Arp::decode(buf)?),
            #[cfg(feature = "relay")]
            252 => Self::RelayHlr(Hlr::decode(buf)?),
            #[cfg(feature = "relay")]
            253 => Self::RelayHos(Hos::decode(buf)?),
            #[cfg(feature = "relay")]
            254 => Self::RelaySel(Sel::decode(buf)?),
            #[cfg(feature = "relay")]
            255 => Self::RelayErr(Error::decode(buf)?),
            i => return Err(insim_core::DecodeError::NoVariantMatch { found: i.into() }),
        };

        Ok(packet)
    }
}

impl Encode for Packet {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        match self {
            Self::Isi(i) => {
                1_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Ver(i) => {
                2_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Tiny(i) => {
                3_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Small(i) => {
                4_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Sta(i) => {
                5_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Sch(i) => {
                6_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Sfp(i) => {
                7_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Scc(i) => {
                8_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Cpp(i) => {
                9_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Ism(i) => {
                10_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Mso(i) => {
                11_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Iii(i) => {
                12_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Mst(i) => {
                13_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Mtc(i) => {
                14_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Mod(i) => {
                15_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Vtn(i) => {
                16_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Rst(i) => {
                17_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Ncn(i) => {
                18_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Cnl(i) => {
                19_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Cpr(i) => {
                20_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Npl(i) => {
                21_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Plp(i) => {
                22_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Pll(i) => {
                23_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Lap(i) => {
                24_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Spx(i) => {
                25_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Pit(i) => {
                26_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Psf(i) => {
                27_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Pla(i) => {
                28_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Cch(i) => {
                29_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Pen(i) => {
                30_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Toc(i) => {
                31_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Flg(i) => {
                32_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Pfl(i) => {
                33_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Fin(i) => {
                34_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Res(i) => {
                35_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Reo(i) => {
                36_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Nlp(i) => {
                37_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Mci(i) => {
                38_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Msx(i) => {
                39_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Msl(i) => {
                40_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Crs(i) => {
                41_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Bfn(i) => {
                42_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Axi(i) => {
                43_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Axo(i) => {
                44_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Btn(i) => {
                45_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Btc(i) => {
                46_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Btt(i) => {
                47_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Rip(i) => {
                48_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Ssh(i) => {
                49_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Con(i) => {
                50_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Obh(i) => {
                51_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Hlv(i) => {
                52_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Plc(i) => {
                53_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Axm(i) => {
                54_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Acr(i) => {
                55_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Hcp(i) => {
                56_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Nci(i) => {
                57_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Jrr(i) => {
                58_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Uco(i) => {
                59_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Oco(i) => {
                60_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Ttc(i) => {
                61_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Slc(i) => {
                62_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Csc(i) => {
                63_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Cim(i) => {
                64_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Mal(i) => {
                65_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Plh(i) => {
                66_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Ipb(i) => {
                67_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Aic(i) => {
                68_u8.encode(buf)?;
                i.encode(buf)?;
            },
            Self::Aii(i) => {
                69_u8.encode(buf)?;
                i.encode(buf)?;
            },
            #[cfg(feature = "relay")]
            Self::RelayArq(i) => {
                250_u8.encode(buf)?;
                i.encode(buf)?;
            },
            #[cfg(feature = "relay")]
            Self::RelayArp(i) => {
                251_u8.encode(buf)?;
                i.encode(buf)?;
            },
            #[cfg(feature = "relay")]
            Self::RelayHlr(i) => {
                252_u8.encode(buf)?;
                i.encode(buf)?;
            },
            #[cfg(feature = "relay")]
            Self::RelayHos(i) => {
                253_u8.encode(buf)?;
                i.encode(buf)?;
            },
            #[cfg(feature = "relay")]
            Self::RelaySel(i) => {
                254_u8.encode(buf)?;
                i.encode(buf)?;
            },
            #[cfg(feature = "relay")]
            Self::RelayErr(i) => {
                255_u8.encode(buf)?;
                i.encode(buf)?;
            },
        };

        Ok(())
    }
}
