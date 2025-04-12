//! Macros for insim
extern crate proc_macro;

use darling::{ast, util, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(hello), supports(struct_named))]
struct Receiver {
    pub ident: Ident,
    data: ast::Data<util::Ignored, Field>,
}

#[derive(Debug, FromField)]
#[darling(attributes(fromtobytes))]
struct Field {
    pub ident: Option<Ident>,
    pub pad_before: Option<usize>,
    pub pad_after: Option<usize>,
    pub skip: Option<bool>,
    pub ty: Type,
}

#[proc_macro_derive(FromToBytes, attributes(fromtobytes))]
/// Derive a basic FromToBytes implementation which assumes all fields also implement FromToBytes.
/// Fields may be skipped by supplying #[fromtobytes(skip)]
/// Fields may have padding before or after using #[fromtobytes(pad_after=2)]
pub fn derive_from_to_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let receiver = Receiver::from_derive_input(&input).unwrap();
    let name = &receiver.ident;
    let fields = &receiver
        .data
        .as_ref()
        .take_struct()
        .expect("Should never be an enum")
        .fields;

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
            self.#field_name.to_bytes(buf)?;
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
            let #field_name = #field_type::from_bytes(buf)?;
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
        impl ::insim_core::FromToBytes for #name {
            /// Read
            fn from_bytes(buf: &mut ::bytes::Bytes) -> Result<Self, ::insim_core::Error> {
                #(#from_bytes_fields)*
                Ok(Self {
                    #(#from_bytes_fields_init),*
                })
            }

            /// Write
            fn to_bytes(&self, buf: &mut ::bytes::BytesMut) -> Result<(), ::insim_core::Error> {
                #(#to_bytes_fields)*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
