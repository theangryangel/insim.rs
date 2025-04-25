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
            repr_ty = Some(m.path.get_ident().unwrap().clone());
            Ok(())
        })
        .unwrap();
        match repr_ty.as_ref()?.to_string().as_str() {
            "u8" | "u16" | "u32" | "u64" => Some(repr_ty?),
            _ => None,
        }
    }

    fn parse_enum(&self, variants: &[Variant]) -> Result<proc_macro2::TokenStream, darling::Error> {
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
            impl ::insim_core::ReadWriteBuf for #name {
                /// Read
                fn read_buf(buf: &mut ::bytes::Bytes) -> Result<Self, ::insim_core::Error> {
                    let val: Self = match #repr_ty::read_buf(buf)? {
                        #(#from_variants)*
                        found => return Err(::insim_core::Error::NoVariantMatch { found: found as u64 })
                    };
                    Ok(val)
                }

                /// Write
                fn write_buf(&self, buf: &mut ::bytes::BytesMut) -> Result<(), ::insim_core::Error> {
                    let val: #repr_ty = match self {
                        #(#to_variants)*
                    };
                    val.write_buf(buf)?;
                    Ok(())
                }
            }
        })
    }

    fn parse_struct(
        &self,
        fields: &darling::ast::Fields<Field>,
    ) -> Result<proc_macro2::TokenStream, darling::Error> {
        let name = &self.ident;

        let to_bytes_fields = fields.iter().filter_map(|f| {
            if f.skip() {
                None
            } else {
                Some(f.impl_write_buf())
            }
        });

        let from_bytes_fields = fields.iter().filter_map(|f| {
            if f.skip() {
                None
            } else {
                Some(f.impl_read_buf())
            }
        });

        let from_bytes_fields_init = fields.iter().filter_map(|f| {
            if f.skip() {
                return None;
            }
            let field_name = f.ident.as_ref().unwrap();

            Some(quote! {
                #field_name
            })
        });

        Ok(quote! {
            impl ::insim_core::ReadWriteBuf for #name {
                /// Read
                fn read_buf(buf: &mut ::bytes::Bytes) -> Result<Self, ::insim_core::Error> {
                    #(#from_bytes_fields)*
                    Ok(Self {
                        #(#from_bytes_fields_init),*
                    })
                }

                /// Write
                fn write_buf(&self, buf: &mut ::bytes::BytesMut) -> Result<(), ::insim_core::Error> {
                    #(#to_bytes_fields)*
                    Ok(())
                }
            }
        })
    }

    pub fn parse(&self) -> Result<proc_macro2::TokenStream, darling::Error> {
        match &self.data {
            darling::ast::Data::Enum(items) => self.parse_enum(items),
            darling::ast::Data::Struct(fields) => self.parse_struct(fields),
        }
    }
}
