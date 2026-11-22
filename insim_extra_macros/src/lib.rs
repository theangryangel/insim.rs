//! Proc macros for `insim_extra`. Use them through `insim_extra`, which
//! re-exports them alongside the traits they implement.
extern crate proc_macro;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod chat_command;

/// Derive `FromStr` and `ChatCommand` for an enum of chat commands.
///
/// See `insim_extra::chat::ChatCommand` for the parsing rules.
#[proc_macro_derive(ChatCommand, attributes(chat))]
pub fn derive_chat_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = match chat_command::Receiver::from_derive_input(&input) {
        Ok(r) => r,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    TokenStream::from(match receiver.expand() {
        Ok(tokens) => tokens,
        Err(e) => e.write_errors(),
    })
}
