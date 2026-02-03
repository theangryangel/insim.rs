extern crate proc_macro;

use quote::quote;

mod enum_variant;
mod struct_field;
use enum_variant::Variant;
use struct_field::Field;

#[derive(Debug, darling::FromDeriveInput)]
#[darling(supports(struct_named, enum_any), forward_attrs(repr))]
pub(super) struct Receiver {
    ident: syn::Ident,
    data: darling::ast::Data<Variant, Field>,
    attrs: Vec<syn::Attribute>,
}

impl Receiver {
    fn repr_type(&self) -> Result<Option<syn::Ident>, darling::Error> {
        let attr = match self.attrs.iter().find(|a| a.path().is_ident("repr")) {
            Some(attr) => attr,
            None => return Ok(None),
        };
        let mut repr_ty = None;
        attr.parse_nested_meta(|m| {
            let ident = m
                .path
                .get_ident()
                .ok_or_else(|| darling::Error::custom("repr must be an ident").with_span(&m.path))?
                .clone();
            repr_ty = Some(ident);
            Ok(())
        })
        .map_err(|e| {
            darling::Error::custom(format!("failed to parse repr: {}", e)).with_span(attr)
        })?;

        match repr_ty.as_ref().map(|repr| repr.to_string()) {
            Some(ref name) if matches!(name.as_str(), "u8" | "u16" | "u32" | "u64") => Ok(repr_ty),
            Some(_) => Err(darling::Error::custom("repr must be u8/u16/u32/u64").with_span(attr)),
            None => Ok(None),
        }
    }

    fn encode_enum(
        &self,
        variants: &[Variant],
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let name = &self.ident;
        let repr_ty = self
            .repr_type()?
            .ok_or_else(|| darling::Error::custom("repr must be u8/u16/u32/u64").with_span(name))?;

        let mut to_variants = Vec::new();
        for f in variants.iter() {
            if f.skip() {
                continue;
            }

            let variant_name = &f.ident;
            let discrim = f.discriminant.as_ref().ok_or_else(|| {
                darling::Error::custom("enum variants must have explicit discriminants")
                    .with_span(&f.ident)
            })?;

            to_variants.push(quote! {
                Self::#variant_name => #discrim,
            });
        }

        Ok(quote! {
            impl ::insim_core::Encode for #name {
                /// Write
                fn encode(&self, buf: &mut ::bytes::BytesMut) -> Result<(), ::insim_core::EncodeError> {
                    let val: #repr_ty = match self {
                        #(#to_variants)*
                    };
                    val.encode(buf)?;
                    Ok(())
                }
            }
        })
    }

    fn encode_struct(
        &self,
        fields: &darling::ast::Fields<Field>,
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let name = &self.ident;

        let mut to_bytes_fields = Vec::new();
        for f in fields.iter() {
            if f.skip() {
                continue;
            }
            to_bytes_fields.push(f.encode(name)?);
        }

        Ok(quote! {
            impl ::insim_core::Encode for #name {
                /// Write
                fn encode(&self, buf: &mut ::bytes::BytesMut) -> Result<(), ::insim_core::EncodeError> {
                    #(#to_bytes_fields)*
                    Ok(())
                }
            }
        })
    }

    pub fn encode(&self) -> Result<proc_macro2::TokenStream, darling::Error> {
        match &self.data {
            darling::ast::Data::Enum(items) => self.encode_enum(items),
            darling::ast::Data::Struct(fields) => self.encode_struct(fields),
        }
    }

    fn decode_enum(
        &self,
        variants: &[Variant],
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let name = &self.ident;
        let repr_ty = self
            .repr_type()?
            .ok_or_else(|| darling::Error::custom("repr must be u8/u16/u32/u64").with_span(name))?;
        let mut from_variants = Vec::new();
        for f in variants.iter() {
            if f.skip() {
                continue;
            }
            let variant_name = f.ident.clone();
            let discrim = f.discriminant.clone().ok_or_else(|| {
                darling::Error::custom("enum variants must have explicit discriminants")
                    .with_span(&f.ident)
            })?;

            from_variants.push(quote! {
                #discrim => Self::#variant_name,
            });
        }

        Ok(quote! {
            impl ::insim_core::Decode for #name {
                /// Read
                fn decode(buf: &mut ::bytes::Bytes) -> Result<Self, ::insim_core::DecodeError> {
                    let val: Self = match #repr_ty::decode(buf)? {
                        #(#from_variants)*
                        found => return Err(::insim_core::DecodeErrorKind::NoVariantMatch { found: found as u64 }.into())
                    };
                    Ok(val)
                }
            }
        })
    }

    fn decode_struct(
        &self,
        fields: &darling::ast::Fields<Field>,
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let name = &self.ident;

        let mut from_bytes_fields = Vec::new();
        for f in fields.iter() {
            if f.skip() {
                continue;
            }
            from_bytes_fields.push(f.decode(name)?);
        }

        let mut from_bytes_fields_init = Vec::new();
        for f in fields.iter() {
            if f.skip() {
                continue;
            }
            let field_name = f
                .ident
                .as_ref()
                .ok_or_else(|| darling::Error::custom("missing field name").with_span(name))?;

            from_bytes_fields_init.push(quote! {
                #field_name
            });
        }

        Ok(quote! {
            impl ::insim_core::Decode for #name {
                /// Read
                fn decode(buf: &mut ::bytes::Bytes) -> Result<Self, ::insim_core::DecodeError> {
                    #(#from_bytes_fields)*
                    Ok(Self {
                        #(#from_bytes_fields_init),*
                    })
                }
            }
        })
    }

    pub fn decode(&self) -> Result<proc_macro2::TokenStream, darling::Error> {
        match &self.data {
            darling::ast::Data::Enum(items) => self.decode_enum(items),
            darling::ast::Data::Struct(fields) => self.decode_struct(fields),
        }
    }
}
