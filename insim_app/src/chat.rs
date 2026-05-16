//! [`chat_parser`] - generic handler that parses every `Mso` into a typed
//! value `C` via [`FromStr`] and emits the parsed value as a synthetic event.

use std::{any::Any, str::FromStr};

use insim::insim::{Mso, MsoUserType};

use crate::{AppError, Packet, Sender};

/// Generic handler that parses every `Mso` body into a typed value `C` via
/// [`FromStr`] and emits the parsed value as a synthetic event.
///
/// Register with `app.handler(chat_parser::<MyCmd>)`. Pair with `Event<C>`
/// handlers to react to typed commands. On parse failure no event is emitted.
///
/// The parser runs once per `Mso` packet regardless of how many `Event<C>`
/// handlers are registered downstream — they all see the same emitted value.
pub async fn chat_parser<C>(Packet(mso): Packet<Mso>, sender: Sender) -> Result<(), AppError>
where
    C: FromStr + Any + Send + Sync + 'static,
{
    if !matches!(mso.usertype, MsoUserType::User | MsoUserType::Prefix) {
        return Ok(());
    }
    if let Ok(c) = mso.msg_from_textstart().trim().parse::<C>() {
        let _ = sender.event(c);
    }
    Ok(())
}
