use bytes::Buf;
use insim_core::{
    binrw::{self},
    ReadWriteBuf,
};

use crate::identifiers::{PlayerId, RequestId};

const AIC_MAX_INPUTS: usize = 20;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// AI input type
pub enum AiInputType {
    /// Steering
    Msx(u16),

    /// Throttle
    Throttle(u16),

    /// Brake
    Brake(u16),

    /// Gear up
    Chup(u16),

    /// Gear down
    Chdn(u16),

    /// Ignition
    Ignition(u16),

    /// Extra lights
    ExtraLight(u16),

    /// Head lights
    HeadLights(u16),

    /// Siren
    Siren(u16),

    /// Honk
    Horn(u16),

    /// Flash
    Flash(u16),

    /// Clutch
    Clutch(u16),

    /// Handbrake
    Handbrake(u16),

    /// Indicators
    Indicators(u16),

    /// Gear
    Gear(u16),

    /// Look
    Look(u16),

    /// Pitspeed
    Pitspeed(u16),

    /// Disable Traction Control
    TcDisable(u16),

    /// Fogs rear
    FogRear(u16),

    /// Fogs front
    FogFront(u16),

    /// Reset all controlled values
    ResetAll,

    /// Yield control
    StopControl,
}

impl Default for AiInputType {
    fn default() -> Self {
        Self::Msx(32768)
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Input Control, value
pub struct AiInputVal {
    /// Input
    pub input: AiInputType,

    /// Duration
    pub time: u8,
}

impl binrw::BinRead for AiInputVal {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let pos = reader.stream_position()?;

        let input = u8::read_options(reader, endian, ())?;
        let time = u8::read_options(reader, endian, ())?;
        let val = u16::read_options(reader, endian, ())?;

        let input = match input {
            0 => AiInputType::Msx(val),
            1 => AiInputType::Throttle(val),
            2 => AiInputType::Brake(val),
            3 => AiInputType::Chup(val),
            4 => AiInputType::Chdn(val),
            5 => AiInputType::Ignition(val),
            6 => AiInputType::ExtraLight(val),
            7 => AiInputType::HeadLights(val),
            8 => AiInputType::Siren(val),
            9 => AiInputType::Horn(val),
            10 => AiInputType::Flash(val),
            11 => AiInputType::Clutch(val),
            12 => AiInputType::Handbrake(val),
            13 => AiInputType::Indicators(val),
            14 => AiInputType::Gear(val),
            15 => AiInputType::Look(val),
            16 => AiInputType::Pitspeed(val),
            17 => AiInputType::TcDisable(val),
            18 => AiInputType::FogRear(val),
            19 => AiInputType::FogFront(val),
            254 => AiInputType::ResetAll,
            255 => AiInputType::StopControl,
            _ => return Err(binrw::Error::NoVariantMatch { pos }),
        };

        Ok(Self { input, time })
    }
}

impl binrw::BinWrite for AiInputVal {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self.input {
            AiInputType::Msx(val) => {
                0_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Throttle(val) => {
                1_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Brake(val) => {
                2_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Chup(val) => {
                3_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Chdn(val) => {
                4_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Ignition(val) => {
                5_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::ExtraLight(val) => {
                6_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::HeadLights(val) => {
                7_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Siren(val) => {
                8_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Horn(val) => {
                9_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Flash(val) => {
                10_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Clutch(val) => {
                11_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Handbrake(val) => {
                12_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Indicators(val) => {
                13_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Gear(val) => {
                14_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Look(val) => {
                15_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::Pitspeed(val) => {
                16_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::TcDisable(val) => {
                17_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::FogRear(val) => {
                18_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::FogFront(val) => {
                19_u8.write_options(writer, endian, ())?;
                val.write_options(writer, endian, ())?;
            },
            AiInputType::ResetAll => {
                254_u8.write_options(writer, endian, ())?;
                0_u16.write_options(writer, endian, ())?;
            },
            AiInputType::StopControl => {
                255_u8.write_options(writer, endian, ())?;
                0_u16.write_options(writer, endian, ())?;
            },
        };

        self.time.write_options(writer, endian, ())?;

        Ok(())
    }
}

impl ReadWriteBuf for AiInputVal {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let input = u8::read_buf(buf)?;
        let time = u8::read_buf(buf)?;
        let val = u16::read_buf(buf)?;

        let input = match input {
            0 => AiInputType::Msx(val),
            1 => AiInputType::Throttle(val),
            2 => AiInputType::Brake(val),
            3 => AiInputType::Chup(val),
            4 => AiInputType::Chdn(val),
            5 => AiInputType::Ignition(val),
            6 => AiInputType::ExtraLight(val),
            7 => AiInputType::HeadLights(val),
            8 => AiInputType::Siren(val),
            9 => AiInputType::Horn(val),
            10 => AiInputType::Flash(val),
            11 => AiInputType::Clutch(val),
            12 => AiInputType::Handbrake(val),
            13 => AiInputType::Indicators(val),
            14 => AiInputType::Gear(val),
            15 => AiInputType::Look(val),
            16 => AiInputType::Pitspeed(val),
            17 => AiInputType::TcDisable(val),
            18 => AiInputType::FogRear(val),
            19 => AiInputType::FogFront(val),
            254 => AiInputType::ResetAll,
            255 => AiInputType::StopControl,
            found => {
                return Err(insim_core::Error::NoVariantMatch {
                    found: found as u64,
                })
            },
        };

        Ok(Self { input, time })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        match self.input {
            AiInputType::Msx(val) => {
                0_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Throttle(val) => {
                1_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Brake(val) => {
                2_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Chup(val) => {
                3_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Chdn(val) => {
                4_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Ignition(val) => {
                5_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::ExtraLight(val) => {
                6_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::HeadLights(val) => {
                7_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Siren(val) => {
                8_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Horn(val) => {
                9_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Flash(val) => {
                10_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Clutch(val) => {
                11_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Handbrake(val) => {
                12_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Indicators(val) => {
                13_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Gear(val) => {
                14_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Look(val) => {
                15_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::Pitspeed(val) => {
                16_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::TcDisable(val) => {
                17_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::FogRear(val) => {
                18_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::FogFront(val) => {
                19_u8.write_buf(buf)?;
                val.write_buf(buf)?;
            },
            AiInputType::ResetAll => {
                254_u8.write_buf(buf)?;
                0_u16.write_buf(buf)?;
            },
            AiInputType::StopControl => {
                255_u8.write_buf(buf)?;
                0_u16.write_buf(buf)?;
            },
        };

        self.time.write_buf(buf)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Input Control
pub struct Aic {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Set to choose 16-bit
    pub plid: PlayerId,

    /// Inputs to send
    pub inputs: Vec<AiInputVal>,
}

impl_typical_with_request_id!(Aic);

impl binrw::BinWrite for Aic {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        self.reqi.write_options(writer, endian, ())?;
        self.plid.write_options(writer, endian, ())?;
        for i in self.inputs.iter() {
            i.write_options(writer, endian, ())?;
        }
        Ok(())
    }
}

impl binrw::BinRead for Aic {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let reqi = RequestId::read_options(reader, endian, ())?;
        let plid = PlayerId::read_options(reader, endian, ())?;
        let mut inputs = Vec::new();
        loop {
            let pos = reader.stream_position()?;
            let end = reader.seek(std::io::SeekFrom::End(0))?;

            if pos >= end {
                break;
            }
            let _ = reader.seek(std::io::SeekFrom::Start(pos));
            inputs.push(AiInputVal::read_options(reader, endian, ())?);
        }

        Ok(Self { reqi, plid, inputs })
    }
}

impl ReadWriteBuf for Aic {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let plid = PlayerId::read_buf(buf)?;
        let mut inputs = Vec::new();
        while buf.has_remaining() {
            inputs.push(AiInputVal::read_buf(buf)?);
        }

        Ok(Self { reqi, plid, inputs })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        self.plid.write_buf(buf)?;
        if self.inputs.len() > AIC_MAX_INPUTS {
            return Err(insim_core::Error::TooLarge);
        }
        for i in self.inputs.iter() {
            i.write_buf(buf)?;
        }
        Ok(())
    }
}
