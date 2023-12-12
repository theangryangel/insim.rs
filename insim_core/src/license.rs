#[non_exhaustive]
#[derive(PartialEq, PartialOrd, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum License {
    Demo,
    S1,
    S2,
    S3,
}

impl std::fmt::Display for License {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            License::Demo => write!(f, "Demo"),
            License::S1 => write!(f, "S1"),
            License::S2 => write!(f, "S2"),
            License::S3 => write!(f, "S3"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_order() {
        assert!(License::S3 > License::S2);
        assert!(License::S3 > License::S1);
        assert!(License::S3 > License::Demo);

        assert!(License::S2 > License::S1);
        assert!(License::S2 > License::Demo);

        assert!(License::S1 > License::Demo);
    }
}
