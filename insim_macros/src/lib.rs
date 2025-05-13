//! Macros for insim
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
extern crate proc_macro;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod encode;

#[proc_macro_derive(Encode, attributes(insim))]
/// Derive a basic Encode implementation for either:
/// 1. Structs
///    Assumes all fields also implement Encode
///    Fields may have padding before or after using #[insim(pad_after=2)]
///    Fields may be skipped by supplying #[insim(skip)]
///    Fields which are strings must have either acsii, or codepage directives provided.
/// 2. Enums which are repr(typ) and have a supplied discriminant
///    Variants may be skipped using #[insim(skip)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = match encode::Receiver::from_derive_input(&input) {
        Ok(r) => r,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    TokenStream::from(match receiver.encode() {
        Ok(tokens) => tokens,
        Err(e) => e.write_errors(),
    })
}

#[proc_macro_derive(Decode, attributes(insim))]
/// Derive a basic Decode implementation for either:
/// 1. Structs
///    Assumes all fields also implement Decode
///    Fields may have padding before or after using #[insim(pad_after=2)]
///    Fields may be skipped by supplying #[insim(skip)]
///    Fields which are strings must have either acsii, or codepage directives provided.
/// 2. Enums which are repr(typ) and have a supplied discriminant
///    Variants may be skipped using #[insim(skip)]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = match encode::Receiver::from_derive_input(&input) {
        Ok(r) => r,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    TokenStream::from(match receiver.decode() {
        Ok(tokens) => tokens,
        Err(e) => e.write_errors(),
    })
}
