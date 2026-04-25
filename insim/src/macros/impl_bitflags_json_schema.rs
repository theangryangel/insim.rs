macro_rules! impl_bitflags_json_schema {
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

        #[cfg(feature = "schemars")]
        impl ::schemars::JsonSchema for $type {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                ::std::stringify!($type).into()
            }

            fn json_schema(_gen: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                let names: ::std::vec::Vec<::std::string::String> =
                    <$type as ::bitflags::Flags>::FLAGS
                        .iter()
                        .map(|f| f.name().to_owned())
                        .collect();
                ::schemars::json_schema!({
                    "type": "array",
                    "items": { "type": "string", "enum": names },
                    "uniqueItems": true
                })
            }
        }
    };
}
