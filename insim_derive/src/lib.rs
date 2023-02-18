#[macro_use]
extern crate quote;

use darling::{ast, FromDeriveInput, FromField, FromVariant};
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod decode;
mod encode;

#[derive(FromDeriveInput)]
#[darling(attributes(insim), supports(struct_any, enum_any), forward_attrs(repr))]
pub(crate) struct Receiver {
    pub ident: syn::Ident,

    /// Forwarding all attrs so that we can find the repr for the enum discriminant type
    pub attrs: Vec<syn::Attribute>,

    /// Field and Variant information
    pub data: ast::Data<VariantData, FieldData>,

    /// A "magic" value that must appear at the start of this struct/enum's data
    pub magic: Option<syn::LitByteStr>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(insim))]
struct VariantData {
    pub ident: syn::Ident,
    pub fields: ast::Fields<FieldData>,
    pub discriminant: Option<syn::Expr>,
}

#[derive(Debug, FromField)]
#[darling(attributes(insim))]
pub(crate) struct FieldData {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,

    // escape hatch incase we need to skip
    #[darling(default)]
    pub skip: bool,

    // pad for X bytes after this field
    pub pad_bytes_after: Option<usize>,

    // pad for X bytes before this field
    pub pad_bytes_before: Option<usize>,

    // assume a fixed byte size and automatically truncate
    pub bytes: Option<usize>,

    // count
    pub count: Option<String>,
}

fn extract_type(ty: &syn::Type) -> proc_macro2::TokenStream {
    // converts a Vec<u8> into Vec::<u8> for usage in the decoding calls
    // if it doesnt match that format then just output the original type
    // wrapped in angled brackets

    if let syn::Type::Path(syn::TypePath {
        qself: None,
        ref path,
        ..
    }) = ty
    {
        if path.leading_colon.is_none() && path.segments.len() == 1 {
            if let syn::PathArguments::AngleBracketed(ang) = &path.segments[0].arguments {
                let ident = &path.segments[0].ident;
                return quote! { #ident::#ang };
            }
        }
    }

    quote! { <#ty> }
}

fn extract_repr_type(attrs: &[syn::Attribute]) -> Option<syn::Ident> {
    attrs.iter().find_map(|attr| {
        if attr.path.is_ident("repr") {
            match attr.parse_args::<syn::Ident>() {
                Ok(ident) => Some(ident),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn gen_field_name(ident: &Option<syn::Ident>, index: usize) -> proc_macro2::TokenStream {
    let name = match ident {
        Some(ident) => ident.to_string(),
        None => format!("{index}"),
    };

    name.parse().unwrap()
}

#[proc_macro_derive(InsimEncode, attributes(insim))]
#[proc_macro_error]
pub fn insim_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Darling ensures that we only support named structs, and extracts the relevant fields
    match Receiver::from_derive_input(&input) {
        Ok(receiver) => {
            let mut tokens = proc_macro2::TokenStream::new();

            let ident = &receiver.ident;
            let encode = receiver.to_encode_tokens();

            tokens.extend(quote! {
                impl ::insim_core::Encodable for #ident {
                    fn encode(
                        &self,
                        buf: &mut ::insim_core::bytes::BytesMut,
                        limit: Option<::insim_core::ser::Limit>,
                    ) -> Result<(), ::insim_core::EncodableError>
                    {
                        #encode

                        Ok(())
                    }
                }
            });

            tokens.into()
        }
        Err(err) => err.write_errors().into(),
    }
}

#[proc_macro_derive(InsimDecode, attributes(insim))]
#[proc_macro_error]
pub fn insim_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Darling ensures that we only support the relevant types
    match Receiver::from_derive_input(&input) {
        Ok(receiver) => {
            let mut tokens = proc_macro2::TokenStream::new();

            let ident = &receiver.ident;
            let decode = receiver.to_decode_tokens();

            tokens.extend(quote! {
                impl ::insim_core::Decodable for #ident {
                    fn decode(
                        buf: &mut ::insim_core::bytes::BytesMut,
                        limit: Option<::insim_core::ser::Limit>,
                    ) -> Result<Self, ::insim_core::DecodableError>
                    {
                        #decode
                    }
                }
            });

            tokens.into()
        }
        Err(err) => err.write_errors().into(),
    }
}
