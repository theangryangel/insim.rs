use std::time::Duration;

use insim_core::{
    binrw::{self, binrw, BinRead, BinWrite},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use crate::identifiers::{PlayerId, RequestId};

use super::{obh::binrw_parse_spclose_strip_reserved_bits, CompCarInfo};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Used within [Con] packet to give a break down of information about the Contact between the two
/// players.
pub struct ConInfo {
    /// Unique player id
    pub plid: PlayerId,

    /// Additional information
    pub info: CompCarInfo,

    /// Front wheel steer in degrees (right positive)
    pub steer: u8,

    /// Throttle - Insim defines this as a u4, insim.rs will silently truncate this u8.
    pub thr: u8,

    /// Brake - Insim defines this as a u4, insim.rs will validate this on encoding.
    pub brk: u8,

    /// Clutch (0-15) - Insim defines this as a u4, insim.rs will validate this on encoding.
    pub clu: u8,

    /// Handbrake - Insim defines this as a u4, insim.rs will validate this on encoding.
    pub han: u8,

    /// Gear (15=R) - Insim defines this as a u4, insim.rs will validate this on encoding.
    pub gearsp: u8,

    /// Speed in m/s
    pub speed: u8,

    /// Car's motion if Speed > 0: 0 = world y direction, 128 = 180 deg
    pub direction: u8,

    /// direction of forward axis: 0 = world y direction, 128 = 180 deg
    pub heading: u8,

    /// m/s^2 longitudinal acceleration (forward positive)
    pub accelf: u8,

    /// m/s^2 lateral acceleration (right positive)
    pub accelr: u8,

    /// position (1 metre = 16)
    pub x: i16,

    /// position (1 metre = 16)
    pub y: i16,
}

// Manual implementation of BinRead, so that we can accomodate thrbrk, cluhan, etc.
impl BinRead for ConInfo {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let plid = PlayerId::read_options(reader, endian, ())?;
        let info = CompCarInfo::read_options(reader, endian, ())?;
        // pad 1 bytes
        reader.seek(std::io::SeekFrom::Current(1))?;
        let steer = u8::read_options(reader, endian, ())?;

        let thrbrk = u8::read_options(reader, endian, ())?;
        let thr = thrbrk >> 4; // top 4 bits are thr
        let brk = thrbrk & !0b11110000; // last 4 bits are brk

        let cluhan = u8::read_options(reader, endian, ())?;
        let clu = cluhan >> 4; // top 4 bits are clu
        let han = cluhan & !0b11110000; // last 4 bits are han

        let gearsp = u8::read_options(reader, endian, ())?;
        let gearsp = gearsp >> 4; // gearsp is only first 4 bits

        let speed = u8::read_options(reader, endian, ())?;
        let direction = u8::read_options(reader, endian, ())?;
        let heading = u8::read_options(reader, endian, ())?;
        let accelf = u8::read_options(reader, endian, ())?;
        let accelr = u8::read_options(reader, endian, ())?;

        let x = i16::read_options(reader, endian, ())?;
        let y = i16::read_options(reader, endian, ())?;

        Ok(Self {
            plid,
            info,
            steer,
            thr,
            brk,
            clu,
            han,
            gearsp,
            speed,
            direction,
            heading,
            accelf,
            accelr,
            x,
            y,
        })
    }
}

// Manual implementation of BinWrite, so that we can accomodate thrbrk, cluhan, etc.
impl BinWrite for ConInfo {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        self.plid.write_options(writer, endian, ())?;
        self.info.write_options(writer, endian, ())?;
        0_u8.write_options(writer, endian, ())?; // pad 1 bytes
        self.steer.write_options(writer, endian, ())?;

        if self.thr > 15 {
            let pos = writer.stream_position()?;
            return Err(binrw::Error::AssertFail {
                pos,
                message: "thr must be <= 15".into(),
            });
        }

        if self.brk > 15 {
            let pos = writer.stream_position()?;
            return Err(binrw::Error::AssertFail {
                pos,
                message: "brk must be <= 15".into(),
            });
        }

        let thrbrk: u8 = (self.thr << 4) | (self.brk & !0b11110000);
        thrbrk.write_options(writer, endian, ())?;

        if self.clu > 15 {
            let pos = writer.stream_position()?;
            return Err(binrw::Error::AssertFail {
                pos,
                message: "clu must be <= 15".into(),
            });
        }

        if self.han > 15 {
            let pos = writer.stream_position()?;
            return Err(binrw::Error::AssertFail {
                pos,
                message: "han must be <= 15".into(),
            });
        }

        let cluhan: u8 = (self.clu << 4) | (self.han & !0b11110000);
        cluhan.write_options(writer, endian, ())?;

        if self.gearsp > 15 {
            let pos = writer.stream_position()?;
            return Err(binrw::Error::AssertFail {
                pos,
                message: "gearsp must be <= 15".into(),
            });
        }

        let gearsp = self.gearsp & !0b11110000;
        gearsp.write_options(writer, endian, ())?;

        self.speed.write_options(writer, endian, ())?;
        self.direction.write_options(writer, endian, ())?;
        self.heading.write_options(writer, endian, ())?;
        self.accelf.write_options(writer, endian, ())?;
        self.accelr.write_options(writer, endian, ())?;
        self.x.write_options(writer, endian, ())?;
        self.y.write_options(writer, endian, ())?;

        Ok(())
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Contact between 2 vehicles
pub struct Con {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Low 12 bits: closing speed (10 = 1 m/s)
    /// The high 4 bits are automatically stripped.
    #[br(parse_with = binrw_parse_spclose_strip_reserved_bits)]
    pub spclose: u16,

    #[br(parse_with = binrw_parse_duration::<u16, 10, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 10, _>)]
    /// Time since last reset. Warning this is looping.
    pub time: Duration,

    /// Contact information for vehicle A
    pub a: ConInfo,

    /// Contact information for vehicle B
    pub b: ConInfo,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn ensure_coninfo_decodes() {
        // ConInfo has some fields which are effectively u4.
        // We need to ensure that we carefully decode them.
        let coninfo = ConInfo::read_le(&mut Cursor::new(vec![
            1, 0,          // CompCarInfoinfo
            0,          // padding
            12,         // steering
            247,        // thrbrk
            193,        // cluhan
            0b11110000, // gearsp
            0,          //speed
            0,          // direction
            1,          // heading
            2,          // accelf
            3,          // accelr
            0, 0, // X
            0, 0, // Y
        ]))
        .unwrap();

        assert_eq!(coninfo.thr, 15);
        assert_eq!(coninfo.brk, 7);

        assert_eq!(coninfo.clu, 12);
        assert_eq!(coninfo.han, 1);

        assert_eq!(coninfo.gearsp, 15);
    }
}
