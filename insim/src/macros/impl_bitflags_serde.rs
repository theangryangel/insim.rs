macro_rules! impl_bitflags_serde {
    ($type:ty) => {
        #[cfg(feature = "serde")]
        impl ::serde::Serialize for $type {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                ::insim_core::bitflags_serde::serialize(self, serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $type {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> Result<Self, D::Error> {
                ::insim_core::bitflags_serde::deserialize(deserializer)
            }
        }
    };
}
