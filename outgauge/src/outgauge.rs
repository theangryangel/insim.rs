use insim_core::{
    binrw::{self, binrw},
    identifiers::player::PlayerId,
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
    vehicle::Vehicle,
};

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OutgaugePack {
    pub time: u32,

    pub car: Vehicle,

    pub flags: u16,

    pub gear: u8,

    pub plid: PlayerId,

    pub speed: f32,

    pub rpm: f32,

    pub turbo: f32,

    pub engtemp: f32,

    pub fuel: f32,

    pub oilpressure: f32,

    pub dashlights: u32,

    pub showlights: u32,

    pub throttle: f32,

    pub brake: f32,

    pub clutch: f32,

    #[bw(write_with = binrw_write_codepage_string::<16, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub display1: String,

    #[bw(write_with = binrw_write_codepage_string::<16, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub display2: String,

    #[br(try)]
    #[bw(if(id.is_some()))]
    pub id: Option<u32>,
}
