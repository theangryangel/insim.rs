use binrw::{BinRead, BinWrite};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Handles parsing a Track name.
#[non_exhaustive]
pub enum Track {
    #[default]
    Bl1,
    Bl1r,
    Bl1x,

    Bl2,
    Bl2r,
    Bl2x,

    Bl3,
    Bl3r,
    Bl3x,
    Bl4,

    So1,
    So1r,
    So1x,

    So2,
    So2r,
    So2x,

    So3,
    So3r,
    So3x,

    So4,
    So4r,
    So4x,

    So5,
    So5r,
    So5x,

    So6,
    So6r,
    So6x,

    Fe1,
    Fe1r,
    Fe1x,

    Fe2,
    Fe2r,
    Fe2x,

    Fe3,
    Fe3r,
    Fe3x,

    Fe4,
    Fe4r,
    Fe4x,

    Fe5,
    Fe5r,
    Fe5x,

    Fe6,
    Fe6r,
    Fe6x,

    Au1,
    Au2,
    Au3,
    Au4,

    Ky1,
    Ky1r,
    Ky1x,

    Ky2,
    Ky2r,
    Ky2x,

    Ky3,
    Ky3r,
    Ky3x,

    We1,
    We1r,
    We1x,

    We2,
    We2r,
    We2x,

    We3,
    We4,
    We4r,
    We4x,

    We5,
    We5r,
    We5x,

    As1,
    As1r,
    As1x,

    As2,
    As2r,
    AS2x,

    As3,
    As3r,
    As3x,

    As4,
    As4r,
    As4x,

    As5,
    As5r,
    As5x,

    As6,
    As6r,
    As6x,

    As7,
    As7r,
    As7x,

    Ro1,
    Ro1x,

    Ro2,
    Ro2x,

    Ro3,
    Ro3x,

    Ro4,
    Ro4x,

    Ro5,
    Ro5x,

    Ro6,
    Ro6x,

    Ro7,
    Ro7x,

    Ro8,
    Ro8x, 

    Ro9,
    Ro9x,

    Ro10,
    Ro10x,

    Ro11,
    Ro11x,

    La1,
    La2,
}

impl BinRead for Track {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let pos = reader.stream_position()?;

        <[u8; 6]>::read_options(reader, endian, args).map(|bytes| match bytes {
            [b'B', b'L', b'1', 0, 0, 0] => Ok(Self::Bl1),
            [b'B', b'L', b'1', b'R', 0, 0] => Ok(Self::Bl1r),
            [b'B', b'L', b'2', 0, 0, 0] => Ok(Self::Bl2),
            [b'B', b'L', b'2', b'R', 0, 0] => Ok(Self::Bl2r),
            [b'B', b'L', b'3', 0, 0, 0] => Ok(Self::Bl3),
            [b'B', b'L', b'3', b'R', 0, 0] => Ok(Self::Bl3r),
            [b'B', b'L', b'4', 0, 0, 0] => Ok(Self::Bl4),

            [b'B', b'L', b'X', 0, 0, 0] => Ok(Self::Blx),

            [b'S', b'O', b'1', 0, 0, 0] => Ok(Self::So1),
            [b'S', b'O', b'1', b'R', 0, 0] => Ok(Self::So1r),
            [b'S', b'O', b'2', 0, 0, 0] => Ok(Self::So2),
            [b'S', b'O', b'2', b'R', 0, 0] => Ok(Self::So2r),
            [b'S', b'O', b'3', 0, 0, 0] => Ok(Self::So3),
            [b'S', b'O', b'3', b'R', 0, 0] => Ok(Self::So3r),
            [b'S', b'O', b'4', 0, 0, 0] => Ok(Self::So4),
            [b'S', b'O', b'4', b'R', 0, 0] => Ok(Self::So4r),
            [b'S', b'O', b'5', 0, 0, 0] => Ok(Self::So5),
            [b'S', b'O', b'5', b'R', 0, 0] => Ok(Self::So5r),
            [b'S', b'O', b'6', 0, 0, 0] => Ok(Self::So6),
            [b'S', b'O', b'6', b'R', 0, 0] => Ok(Self::So6r),

            [b'S', b'O', b'X', 0, 0, 0] => Ok(Self::Sox),

            [b'F', b'E', b'1', 0, 0, 0] => Ok(Self::Fe1),
            [b'F', b'E', b'1', b'R', 0, 0] => Ok(Self::Fe1r),
            [b'F', b'E', b'2', 0, 0, 0] => Ok(Self::Fe2),
            [b'F', b'E', b'2', b'R', 0, 0] => Ok(Self::Fe2r),
            [b'F', b'E', b'3', 0, 0, 0] => Ok(Self::Fe3),
            [b'F', b'E', b'3', b'R', 0, 0] => Ok(Self::Fe3r),
            [b'F', b'E', b'4', 0, 0, 0] => Ok(Self::Fe4),
            [b'F', b'E', b'4', b'R', 0, 0] => Ok(Self::Fe4r),
            [b'F', b'E', b'5', 0, 0, 0] => Ok(Self::Fe5),
            [b'F', b'E', b'5', b'R', 0, 0] => Ok(Self::Fe5r),
            [b'F', b'E', b'6', 0, 0, 0] => Ok(Self::Fe6),
            [b'F', b'E', b'6', b'R', 0, 0] => Ok(Self::Fe6r),

            [b'F', b'E', b'X', 0, 0, 0] => Ok(Self::Fex),

            [b'A', b'U', b'1', 0, 0, 0] => Ok(Self::Au1),
            [b'A', b'U', b'2', 0, 0, 0] => Ok(Self::Au2),
            [b'A', b'U', b'3', 0, 0, 0] => Ok(Self::Au3),
            [b'A', b'U', b'4', 0, 0, 0] => Ok(Self::Au4),

            [b'K', b'Y', b'1', 0, 0, 0] => Ok(Self::Ky1),
            [b'K', b'Y', b'1', b'R', 0, 0] => Ok(Self::Ky1r),
            [b'K', b'Y', b'2', 0, 0, 0] => Ok(Self::Ky2),
            [b'K', b'Y', b'2', b'R', 0, 0] => Ok(Self::Ky2r),
            [b'K', b'Y', b'3', 0, 0, 0] => Ok(Self::Ky3),
            [b'K', b'Y', b'3', b'R', 0, 0] => Ok(Self::Ky3r),

            [b'K', b'Y', b'X', 0, 0, 0] => Ok(Self::Kyx),

            [b'W', b'E', b'1', 0, 0, 0] => Ok(Self::We1),
            [b'W', b'E', b'1', b'R', 0, 0] => Ok(Self::We1r),
            [b'W', b'E', b'2', 0, 0, 0] => Ok(Self::We2),
            [b'W', b'E', b'2', b'R', 0, 0] => Ok(Self::We2r),
            [b'W', b'E', b'3', 0, 0, 0] => Ok(Self::We3),
            [b'W', b'E', b'4', 0, 0, 0] => Ok(Self::We4),
            [b'W', b'E', b'4', b'R', 0, 0] => Ok(Self::We4r),
            [b'W', b'E', b'5', 0, 0, 0] => Ok(Self::We5),
            [b'W', b'E', b'5', b'R', 0, 0] => Ok(Self::We5r),

            [b'W', b'E', b'X', 0, 0, 0] => Ok(Self::Wex),

            [b'A', b'S', b'1', 0, 0, 0] => Ok(Self::As1),
            [b'A', b'S', b'1', b'R', 0, 0] => Ok(Self::As1r),
            [b'A', b'S', b'2', 0, 0, 0] => Ok(Self::As2),
            [b'A', b'S', b'2', b'R', 0, 0] => Ok(Self::As2r),
            [b'A', b'S', b'3', 0, 0, 0] => Ok(Self::As3),
            [b'A', b'S', b'3', b'R', 0, 0] => Ok(Self::As3r),
            [b'A', b'S', b'4', 0, 0, 0] => Ok(Self::As4),
            [b'A', b'S', b'4', b'R', 0, 0] => Ok(Self::As4r),
            [b'A', b'S', b'5', 0, 0, 0] => Ok(Self::As5),
            [b'A', b'S', b'5', b'R', 0, 0] => Ok(Self::As5r),
            [b'A', b'S', b'6', 0, 0, 0] => Ok(Self::As6),
            [b'A', b'S', b'6', b'R', 0, 0] => Ok(Self::As6r),
            [b'A', b'S', b'7', 0, 0, 0] => Ok(Self::As7),
            [b'A', b'S', b'7', b'R', 0, 0] => Ok(Self::As7r),

            [b'A', b'S', b'X', 0, 0, 0] => Ok(Self::Asx),

            [b'R', b'O', b'1', 0, 0, 0] => Ok(Self::Ro1),
            [b'R', b'O', b'2', 0, 0, 0] => Ok(Self::Ro2),
            [b'R', b'O', b'3', 0, 0, 0] => Ok(Self::Ro3),
            [b'R', b'O', b'4', 0, 0, 0] => Ok(Self::Ro4),
            [b'R', b'O', b'5', 0, 0, 0] => Ok(Self::Ro5),
            [b'R', b'O', b'6', 0, 0, 0] => Ok(Self::Ro6),
            [b'R', b'O', b'7', 0, 0, 0] => Ok(Self::Ro7),
            [b'R', b'O', b'8', 0, 0, 0] => Ok(Self::Ro8),
            [b'R', b'O', b'9', 0, 0, 0] => Ok(Self::Ro9),
            [b'R', b'O', b'1', b'0', 0, 0] => Ok(Self::Ro10),
            [b'R', b'O', b'1', b'1', 0, 0] => Ok(Self::Ro11),

            [b'R', b'O', b'X', 0, 0, 0] => Ok(Self::Rox),

            [b'L', b'A', b'1', 0, 0, 0] => Ok(Self::La1),
            [b'L', b'A', b'2', 0, 0, 0] => Ok(Self::La2),

            _ => {
                panic!("{:?}", String::from_utf8_lossy(&bytes.clone()));
                Err(binrw::Error::NoVariantMatch { pos })
            },
        })?
    }
}

impl BinWrite for Track {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            Self::Bl1 => [b'B', b'L', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::Bl1r => [b'B', b'L', b'1', b'R', 0, 0].write_options(writer, endian, args),
            Self::Bl2 => [b'B', b'L', b'2', 0, 0, 0].write_options(writer, endian, args),
            Self::Bl2r => [b'B', b'L', b'2', b'R', 0, 0].write_options(writer, endian, args),
            Self::Bl3 => [b'B', b'L', b'3', 0, 0, 0].write_options(writer, endian, args),
            Self::Bl3r => [b'B', b'L', b'3', b'R', 0, 0].write_options(writer, endian, args),
            Self::Bl4 => [b'B', b'L', b'4', 0, 0, 0].write_options(writer, endian, args),
            Self::Blx => [b'B', b'L', b'X', 0, 0, 0].write_options(writer, endian, args),

            Self::So1 => [b'S', b'O', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::So1r => [b'S', b'O', b'1', b'R', 0, 0].write_options(writer, endian, args),
            Self::So2 => [b'S', b'O', b'2', 0, 0, 0].write_options(writer, endian, args),
            Self::So2r => [b'S', b'O', b'2', b'R', 0, 0].write_options(writer, endian, args),
            Self::So3 => [b'S', b'O', b'3', 0, 0, 0].write_options(writer, endian, args),
            Self::So3r => [b'S', b'O', b'3', b'R', 0, 0].write_options(writer, endian, args),
            Self::So4 => [b'S', b'O', b'4', 0, 0, 0].write_options(writer, endian, args),
            Self::So4r => [b'S', b'O', b'4', b'R', 0, 0].write_options(writer, endian, args),
            Self::So5 => [b'S', b'O', b'5', 0, 0, 0].write_options(writer, endian, args),
            Self::So5r => [b'S', b'O', b'5', b'R', 0, 0].write_options(writer, endian, args),
            Self::So6 => [b'S', b'O', b'6', 0, 0, 0].write_options(writer, endian, args),
            Self::So6r => [b'S', b'O', b'6', b'R', 0, 0].write_options(writer, endian, args),

            Self::Fe1 => [b'F', b'E', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::Fe1r => [b'F', b'E', b'1', b'R', 0, 0].write_options(writer, endian, args),
            Self::Fe2 => [b'F', b'E', b'2', 0, 0, 0].write_options(writer, endian, args),
            Self::Fe2r => [b'F', b'E', b'2', b'R', 0, 0].write_options(writer, endian, args),
            Self::Fe3 => [b'F', b'E', b'3', 0, 0, 0].write_options(writer, endian, args),
            Self::Fe3r => [b'F', b'E', b'3', b'R', 0, 0].write_options(writer, endian, args),
            Self::Fe4 => [b'F', b'E', b'4', 0, 0, 0].write_options(writer, endian, args),
            Self::Fe4r => [b'F', b'E', b'4', b'R', 0, 0].write_options(writer, endian, args),
            Self::Fe5 => [b'F', b'E', b'5', 0, 0, 0].write_options(writer, endian, args),
            Self::Fe5r => [b'F', b'E', b'5', b'R', 0, 0].write_options(writer, endian, args),
            Self::Fe6 => [b'F', b'E', b'6', 0, 0, 0].write_options(writer, endian, args),
            Self::Fe6r => [b'F', b'E', b'6', b'R', 0, 0].write_options(writer, endian, args),

            Self::Au1 => [b'A', b'U', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::Au2 => [b'A', b'U', b'2', 0, 0, 0].write_options(writer, endian, args),
            Self::Au3 => [b'A', b'U', b'3', 0, 0, 0].write_options(writer, endian, args),
            Self::Au4 => [b'A', b'U', b'4', 0, 0, 0].write_options(writer, endian, args),

            Self::Ky1 => [b'K', b'Y', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::Ky1r => [b'K', b'Y', b'1', b'R', 0, 0].write_options(writer, endian, args),
            Self::Ky2 => [b'K', b'Y', b'2', 0, 0, 0].write_options(writer, endian, args),
            Self::Ky2r => [b'K', b'Y', b'2', b'R', 0, 0].write_options(writer, endian, args),
            Self::Ky3 => [b'K', b'Y', b'3', 0, 0, 0].write_options(writer, endian, args),
            Self::Ky3r => [b'K', b'Y', b'3', b'R', 0, 0].write_options(writer, endian, args),
            Self::Kyx => [b'K', b'Y', b'X', 0, 0, 0].write_options(writer, endian, args),

            Self::We1 => [b'W', b'E', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::We1r => [b'W', b'E', b'1', b'R', 0, 0].write_options(writer, endian, args),
            Self::We2 => [b'W', b'E', b'2', 0, 0, 0].write_options(writer, endian, args),
            Self::We2r => [b'W', b'E', b'2', b'R', 0, 0].write_options(writer, endian, args),
            Self::We3 => [b'W', b'E', b'3', 0, 0, 0].write_options(writer, endian, args),
            Self::We4 => [b'W', b'E', b'4', 0, 0, 0].write_options(writer, endian, args),
            Self::We4r => [b'W', b'E', b'4', b'R', 0, 0].write_options(writer, endian, args),
            Self::We5 => [b'W', b'E', b'5', 0, 0, 0].write_options(writer, endian, args),
            Self::We5r => [b'W', b'E', b'5', b'R', 0, 0].write_options(writer, endian, args),

            Self::As1 => [b'A', b'S', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::As1r => [b'A', b'S', b'1', b'R', 0, 0].write_options(writer, endian, args),
            Self::As2 => [b'A', b'S', b'2', 0, 0, 0].write_options(writer, endian, args),
            Self::As2r => [b'A', b'S', b'2', b'R', 0, 0].write_options(writer, endian, args),
            Self::As3 => [b'A', b'S', b'3', 0, 0, 0].write_options(writer, endian, args),
            Self::As3r => [b'A', b'S', b'3', b'R', 0, 0].write_options(writer, endian, args),
            Self::As4 => [b'A', b'S', b'4', 0, 0, 0].write_options(writer, endian, args),
            Self::As4r => [b'A', b'S', b'4', b'R', 0, 0].write_options(writer, endian, args),
            Self::As5 => [b'A', b'S', b'5', 0, 0, 0].write_options(writer, endian, args),
            Self::As5r => [b'A', b'S', b'5', b'R', 0, 0].write_options(writer, endian, args),
            Self::As6 => [b'A', b'S', b'6', 0, 0, 0].write_options(writer, endian, args),
            Self::As6r => [b'A', b'S', b'6', b'R', 0, 0].write_options(writer, endian, args),
            Self::As7 => [b'A', b'S', b'7', 0, 0, 0].write_options(writer, endian, args),
            Self::As7r => [b'A', b'S', b'7', b'R', 0, 0].write_options(writer, endian, args),

            Self::Ro1 => [b'R', b'O', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro2 => [b'R', b'O', b'2', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro3 => [b'R', b'O', b'3', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro4 => [b'R', b'O', b'4', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro5 => [b'R', b'O', b'5', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro6 => [b'R', b'O', b'6', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro7 => [b'R', b'O', b'7', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro8 => [b'R', b'O', b'8', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro9 => [b'R', b'O', b'9', 0, 0, 0].write_options(writer, endian, args),
            Self::Ro10 => [b'R', b'O', b'1', b'0', 0, 0].write_options(writer, endian, args),
            Self::Ro11 => [b'R', b'O', b'1', b'1', 0, 0].write_options(writer, endian, args),

            Self::La1 => [b'L', b'A', b'1', 0, 0, 0].write_options(writer, endian, args),
            Self::La2 => [b'L', b'A', b'2', 0, 0, 0].write_options(writer, endian, args),
        }
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Bl1 => write!(f, "Bl1"),
            Self::Bl1r => write!(f, "Bl1r"),
            Self::Bl2 => write!(f, "Bl2"),
            Self::Bl2r => write!(f, "Bl2r"),
            Self::Bl3 => write!(f, "Bl3"),
            Self::Bl3r => write!(f, "Bl3r"),
            Self::Bl4 => write!(f, "Bl4"),

            Self::So1 => write!(f, "So1"),
            Self::So1r => write!(f, "So1r"),
            Self::So2 => write!(f, "So2"),
            Self::So2r => write!(f, "So2r"),
            Self::So3 => write!(f, "So3"),
            Self::So3r => write!(f, "So3r"),
            Self::So4 => write!(f, "So4"),
            Self::So4r => write!(f, "So4r"),
            Self::So5 => write!(f, "So5"),
            Self::So5r => write!(f, "So5r"),
            Self::So6 => write!(f, "So6"),
            Self::So6r => write!(f, "So6r"),

            Self::Fe1 => write!(f, "Fe1"),
            Self::Fe1r => write!(f, "Fe1r"),
            Self::Fe2 => write!(f, "Fe2"),
            Self::Fe2r => write!(f, "Fe2r"),
            Self::Fe3 => write!(f, "Fe3"),
            Self::Fe3r => write!(f, "Fe3r"),
            Self::Fe4 => write!(f, "Fe4"),
            Self::Fe4r => write!(f, "Fe4r"),
            Self::Fe5 => write!(f, "Fe5"),
            Self::Fe5r => write!(f, "Fe5r"),
            Self::Fe6 => write!(f, "Fe6"),
            Self::Fe6r => write!(f, "Fe6r"),

            Self::Au1 => write!(f, "Au1"),
            Self::Au2 => write!(f, "Au2"),
            Self::Au3 => write!(f, "Au3"),
            Self::Au4 => write!(f, "Au4"),

            Self::Ky1 => write!(f, "Ky1"),
            Self::Ky1r => write!(f, "Ky1r"),
            Self::Ky2 => write!(f, "Ky2"),
            Self::Ky2r => write!(f, "Ky2r"),
            Self::Ky3 => write!(f, "Ky3"),
            Self::Ky3r => write!(f, "Ky3r"),

            Self::We1 => write!(f, "We1"),
            Self::We1r => write!(f, "We1r"),
            Self::We2 => write!(f, "We2"),
            Self::We2r => write!(f, "We2r"),
            Self::We3 => write!(f, "We3"),
            Self::We4 => write!(f, "We4"),
            Self::We4r => write!(f, "We4r"),
            Self::We5 => write!(f, "We5"),
            Self::We5r => write!(f, "We5r"),

            Self::As1 => write!(f, "As1"),
            Self::As1r => write!(f, "As1r"),
            Self::As2 => write!(f, "As2"),
            Self::As2r => write!(f, "As2r"),
            Self::As3 => write!(f, "As3"),
            Self::As3r => write!(f, "As3r"),
            Self::As4 => write!(f, "As4"),
            Self::As4r => write!(f, "As4r"),
            Self::As5 => write!(f, "As5"),
            Self::As5r => write!(f, "As5r"),
            Self::As6 => write!(f, "As6"),
            Self::As6r => write!(f, "As6r"),
            Self::As7 => write!(f, "As7"),
            Self::As7r => write!(f, "As7r"),

            Self::Ro1 => write!(f, "Ro1"),
            Self::Ro2 => write!(f, "Ro2"),
            Self::Ro3 => write!(f, "Ro3"),
            Self::Ro4 => write!(f, "Ro4"),
            Self::Ro5 => write!(f, "Ro5"),
            Self::Ro6 => write!(f, "Ro6"),
            Self::Ro7 => write!(f, "Ro7"),
            Self::Ro8 => write!(f, "Ro8"),
            Self::Ro9 => write!(f, "Ro9"),
            Self::Ro10 => write!(f, "Ro10"),
            Self::Ro11 => write!(f, "Ro11"),

            Self::La1 => write!(f, "La1"),
            Self::La2 => write!(f, "La2"),
        }
    }
}

impl Track {
    /// Is this a reversed track?
    pub fn is_reverse(&self) -> bool {
        matches!(
            self,
            Self::Bl1r
                | Self::Bl2r
                | Self::Bl3r
                | Self::So1r
                | Self::So2r
                | Self::So3r
                | Self::So4r
                | Self::So5r
                | Self::So6r
                | Self::Fe1r
                | Self::Fe2r
                | Self::Fe3r
                | Self::Fe4r
                | Self::Fe5r
                | Self::Fe6r
                | Self::Ky1r
                | Self::Ky2r
                | Self::Ky3r
                | Self::We1r
                | Self::We2r
                | Self::We4r
                | Self::We5r
                | Self::As1r
                | Self::As2r
                | Self::As3r
                | Self::As4r
                | Self::As5r
                | Self::As6r
                | Self::As7r
        )
    }

    /// Are we in open world mode?
    pub fn is_open_world(&self) -> bool {
        matches!(
            self,
            Self::Blx | Self::Sox | Self::Asx | Self::Fex | Self::Kyx | Self::Wex | Self::Rox
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinRead;
    use std::io::Cursor;

    #[test]
    fn test_open_world() {
        let mut data = Cursor::new(vec![b'B', b'L', b'1', 0, 0, 0]);
        let track = Track::read_le(&mut data).unwrap();

        assert!(!track.is_open_world());
        assert!(!track.is_reverse());
        assert_eq!(track.to_string(), "Bl1");
    }

    #[test]
    fn test_reverse() {
        let mut data = Cursor::new(vec![b'B', b'L', b'1', b'R', 0, 0]);
        let track = Track::read_le(&mut data).unwrap();

        assert!(!track.is_open_world());
        assert!(track.is_reverse());
        assert_eq!(track.to_string(), "Bl1r");
    }
}
