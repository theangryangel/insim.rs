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
    fn repr_type(&self) -> Option<syn::Ident> {
        let attr = self.attrs.iter().find(|a| a.path().is_ident("repr"))?;
        let mut repr_ty = None;
        attr.parse_nested_meta(|m| {
            repr_ty = Some(m.path.get_ident().expect("Missing ident").clone());
            Ok(())
        })
        .expect("Expected to parse nested meta");
        match repr_ty.as_ref()?.to_string().as_str() {
            "u8" | "u16" | "u32" | "u64" => Some(repr_ty?),
            _ => None,
        }
    }

    fn encode_enum(
        &self,
        variants: &[Variant],
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let name = &self.ident;
        let repr_ty = &self
            .repr_type()
            .expect("ReadWriteBuf requires a repr type of u8..u64");

        let to_variants = variants.iter().filter_map(|f| {
            if f.skip() {
                return None;
            }

            let variant_name = &f.ident;
            let discrim = f
                .discriminant
                .as_ref()
                .expect("ReadWriteBuf only works with discriminants");

            Some(quote! {
                Self::#variant_name => #discrim,
            })
        });

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

        let to_bytes_fields = fields
            .iter()
            .filter_map(|f| if f.skip() { None } else { Some(f.encode()) });

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
        let repr_ty = &self
            .repr_type()
            .expect("ReadWriteBuf requires a repr type of u8..u64");
        let from_variants = variants.iter().filter_map(|f| {
            if f.skip() {
                return None;
            }
            let variant_name = f.ident.clone();
            let discrim = f
                .discriminant
                .clone()
                .expect("ReadWriteBuf only works with discriminants");

            Some(quote! {
                #discrim => Self::#variant_name,
            })
        });

        Ok(quote! {
            impl ::insim_core::Decode for #name {
                /// Read
                fn decode(buf: &mut ::bytes::Bytes) -> Result<Self, ::insim_core::DecodeError> {
                    let val: Self = match #repr_ty::decode(buf)? {
                        #(#from_variants)*
                        found => return Err(::insim_core::DecodeError::NoVariantMatch { found: found as u64 })
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

        let from_bytes_fields = fields
            .iter()
            .filter_map(|f| if f.skip() { None } else { Some(f.decode()) });

        let from_bytes_fields_init = fields.iter().filter_map(|f| {
            if f.skip() {
                return None;
            }
            let field_name = f.ident.as_ref().expect("Missing field name");

            Some(quote! {
                #field_name
            })
        });

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
