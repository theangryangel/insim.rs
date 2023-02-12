use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::Serialize;

#[non_exhaustive]
#[derive(PartialEq, PartialOrd, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum License {
    Demo,
    S1,
    S2,
    S3,
}

impl Display for License {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            License::Demo => write!(f, "Demo"),
            License::S1 => write!(f, "S1"),
            License::S2 => write!(f, "S2"),
            License::S3 => write!(f, "S3"),
        }
    }
}
