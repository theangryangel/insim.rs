use super::InsimString;
use serde::ser::{Serialize, Serializer};

impl Serialize for InsimString {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}
