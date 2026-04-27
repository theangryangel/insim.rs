macro_rules! impl_bitflags_json_schema {
    ($type:ty, $item_name:literal) => {
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

            fn json_schema(generator: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                let names: ::std::vec::Vec<::serde_json::Value> =
                    <$type as ::bitflags::Flags>::FLAGS
                        .iter()
                        .map(|f| ::serde_json::Value::String(f.name().to_owned()))
                        .collect();

                // Register the individual flag names as a named enum definition so
                // that code-generators (e.g. datamodel-codegen) produce a typed enum
                // class rather than a bare list of strings.
                let _ = generator.definitions_mut().insert(
                    $item_name.to_owned(),
                    ::serde_json::json!({
                        "type": "string",
                        "enum": names
                    }),
                );

                let defs_path = generator
                    .settings()
                    .definitions_path
                    .as_ref()
                    .trim_end_matches('/');
                let ref_path = ::std::format!("#{}/{}", defs_path, $item_name);

                ::schemars::json_schema!({
                    "type": "array",
                    "items": { "$ref": ref_path },
                    "uniqueItems": true
                })
            }
        }
    };
}
