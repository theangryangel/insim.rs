//! Macros for insim
extern crate proc_macro;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod read_write_buf;

#[proc_macro_derive(ReadWriteBuf, attributes(read_write_buf))]
/// Derive a basic ReadWriteBuf implementation for either:
/// 1. Structs
///    Assumes all fields also implement ReadWriteBuf
///    Fields may have padding before or after using #[read_write_buf(pad_after=2)]
///    Fields may be skipped by supplying #[read_write_buf(skip)]
///    Fields which are strings must have either acsii, or codepage directives provided.
/// 2. Enums which are repr(typ) and have a supplied discriminant
///    Variants may be skipped using #[read_write_buf(skip)]
pub fn derive_read_write_buf(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = match read_write_buf::Receiver::from_derive_input(&input) {
        Ok(r) => r,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    TokenStream::from(match receiver.parse() {
        Ok(tokens) => tokens,
        Err(e) => e.write_errors(),
    })
}
