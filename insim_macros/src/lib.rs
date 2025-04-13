//! Macros for insim
extern crate proc_macro;

use darling::{
    ast::{self, Fields},
    FromDeriveInput, FromField, FromVariant,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named, enum_any), forward_attrs(repr))]
struct Receiver {
    pub ident: Ident,
    data: ast::Data<Variant, Field>,
    attrs: Vec<syn::Attribute>,
}

impl Receiver {
    fn repr_type(&self) -> Option<Ident> {
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

    fn parse_enum(&self, variants: &[Variant]) -> TokenStream {
        let name = &self.ident;
        let repr_ty = &self
            .repr_type()
            .expect("ReadWriteBuf requires a repr type of u8..u64");
        let from_variants = variants.iter().filter_map(|f| {
            let variant_name = f.ident.clone();
            let discrim = f
                .discriminant
                .clone()
                .expect("ReadWriteBuf only works with discriminants");
            let skip = f.skip.unwrap_or(false);
            if skip {
                return None;
            }

            Some(quote! {
                #discrim => Self::#variant_name,
            })
        });

        let to_variants = variants.iter().filter_map(|f| {
            let variant_name = &f.ident;
            let discrim = f
                .discriminant
                .as_ref()
                .expect("ReadWriteBuf only works with discriminants");
            let skip = f.skip.unwrap_or(false);
            if skip {
                return None;
            }

            Some(quote! {
                Self::#variant_name => #discrim,
            })
        });

        TokenStream::from(quote! {
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

    fn parse_struct(&self, fields: &Fields<Field>) -> TokenStream {
        let name = &self.ident;

        let to_bytes_fields = fields.iter().filter_map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let pad_after = f.pad_after.unwrap_or(0);
            let pad_before = f.pad_before.unwrap_or(0);
            let skip = f.skip.unwrap_or(false);
            if skip {
                return None;
            }
            let mut tokens = quote! {};

            if pad_before > 0 {
                tokens = quote! {
                    #tokens
                    <::bytes::BytesMut as ::bytes::buf::BufMut>::put_bytes(buf, 0, #pad_before);
                }
            }

            tokens = quote! {
                #tokens
                self.#field_name.write_buf(buf)?;
            };

            if pad_after > 0 {
                tokens = quote! {
                    #tokens
                    <::bytes::BytesMut as ::bytes::buf::BufMut>::put_bytes(buf, 0, #pad_after);
                }
            }

            Some(tokens)
        });

        let from_bytes_fields = fields.iter().filter_map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let pad_after = f.pad_after.unwrap_or(0);
            let pad_before = f.pad_before.unwrap_or(0);
            let field_type = f.ty.clone();
            let skip = f.skip.unwrap_or(false);
            if skip {
                return None;
            }
            let mut tokens = quote! {};

            if pad_before > 0 {
                tokens = quote! {
                    #tokens
                    <::bytes::Bytes as ::bytes::buf::Buf>::advance(buf, #pad_before);
                }
            }

            tokens = quote! {
                #tokens
                let #field_name = #field_type::read_buf(buf)?;
            };

            if pad_after > 0 {
                tokens = quote! {
                    #tokens
                    <::bytes::Bytes as ::bytes::buf::Buf>::advance(buf, #pad_after);
                }
            }

            Some(tokens)
        });

        let from_bytes_fields_init = fields.iter().filter_map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let skip = f.skip.unwrap_or(false);
            if skip {
                return None;
            }

            Some(quote! {
                #field_name
            })
        });

        let expanded = quote! {
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
        };

        TokenStream::from(expanded)
    }

    pub fn parse(&self) -> TokenStream {
        match &self.data {
            ast::Data::Enum(items) => self.parse_enum(items),
            ast::Data::Struct(fields) => self.parse_struct(fields),
        }
    }
}

#[derive(Debug, FromVariant)]
#[darling(attributes(read_write_buf))]
struct Variant {
    pub ident: Ident,
    pub discriminant: Option<syn::Expr>,
    pub skip: Option<bool>,
}

#[derive(Debug, FromField)]
#[darling(attributes(read_write_buf))]
struct Field {
    pub ident: Option<Ident>,
    pub pad_before: Option<usize>,
    pub pad_after: Option<usize>,
    pub skip: Option<bool>,
    pub ty: Type,
}

#[proc_macro_derive(ReadWriteBuf, attributes(read_write_buf))]
/// Derive a basic ReadWriteBuf implementation for either:
/// 1. Structs
///    Assumes all fields also implement ReadWriteBuf
///    Fields may have padding before or after using #[read_write_buf(pad_after=2)]
///    Fields may be skipped by supplying #[read_write_buf(skip)]
/// 2. Enums which are repr(typ) and have a supplied discriminant
///    Variants may be skipped using #[read_write_buf(skip)]
pub fn derive_read_write_buf(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = Receiver::from_derive_input(&input).unwrap();
    receiver.parse()
}
