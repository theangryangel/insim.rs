use crate::{Decodable, Encodable};
use std::convert::From;

#[derive(Debug, Default, Clone, Copy)]
pub enum RaceLaps {
    #[default]
    Practice,
    Laps(usize),
    Hours(usize),
}

impl From<u8> for RaceLaps {
    fn from(value: u8) -> Self {
        match value {
            0 => RaceLaps::Practice,
            1..=99 => RaceLaps::Laps(value.into()),
            100..=190 => RaceLaps::Laps(((value - 100) * 10 + 100).into()),
            191..=238 => RaceLaps::Hours((value - 190).into()),
            _ => RaceLaps::Practice,
        }
    }
}

impl From<RaceLaps> for u8 {
    fn from(item: RaceLaps) -> u8 {
        let data = match item {
            RaceLaps::Practice => 0,
            RaceLaps::Laps(data) => {
                match data {
                    1..=99 => data,
                    100..=1000 => ((data - 100) / 10) + 100,
                    _ => 0, // if it's an invalid structure we're going to push it into practice
                }
            }
            RaceLaps::Hours(data) => data + 190,
        };

        data as u8
    }
}

impl Decodable for RaceLaps {
    fn decode(
        buf: &mut bytes::BytesMut,
        _limit: Option<crate::ser::Limit>,
    ) -> Result<Self, crate::DecodableError>
    where
        Self: Sized,
    {
        let data: RaceLaps = u8::decode(buf, None)?.into();
        Ok(data)
    }
}

impl Encodable for RaceLaps {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        _limit: Option<crate::ser::Limit>,
    ) -> Result<(), crate::EncodableError>
    where
        Self: Sized,
    {
        let data = Into::<u8>::into(*self);
        data.encode(buf, None)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_practise() {
        let data = RaceLaps::Practice;
        assert_eq!(Into::<u8>::into(data), 0);
    }

    #[test]
    fn as_simple_laps() {
        assert_eq!(Into::<u8>::into(RaceLaps::Laps(1)), 1);
        assert_eq!(1, Into::<u8>::into(RaceLaps::Laps(1)));

        assert_eq!(Into::<u8>::into(RaceLaps::Laps(99)), 99);
        assert_eq!(99, Into::<u8>::into(RaceLaps::Laps(99)));
    }

    #[test]
    fn as_complex_laps() {
        assert_eq!(Into::<u8>::into(RaceLaps::Laps(199)), 109);
        assert_eq!(109, Into::<u8>::into(RaceLaps::Laps(199)));
    }

    #[test]
    fn as_hours() {
        assert_eq!(Into::<u8>::into(RaceLaps::Hours(1)), 191);
        assert_eq!(191, Into::<u8>::into(RaceLaps::Hours(1)));
    }
}
