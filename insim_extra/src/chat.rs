//! Generic chat message parser.
//!
//! [`ChatParser`] parses every `Mso`/`Iii` packet into a typed value `C` via
//! [`std::str::FromStr`]. No `Sender` or framework dependency.

use std::{any::Any, marker::PhantomData, str::FromStr};

use insim::{identifiers::ConnectionId, insim::MsoUserType};

/// A successfully parsed chat message.
#[derive(Debug, Clone)]
pub struct ChatEvent<C>
where
    C: FromStr + Any + Send + Sync + 'static,
{
    /// Connection that sent the message.
    pub ucid: ConnectionId,
    /// Parsed message value.
    pub parsed: C,
    /// Original raw packet.
    pub original: insim::Packet,
}

/// Generic chat message parser.
///
/// Parses every `Mso` (user chat) or `Iii` (in-game text input) packet body
/// into a typed value `C` via [`FromStr`]. On parse failure no event is emitted
/// and no error is returned.
///
/// Prefix characters are stripped before parsing (e.g. `!`, `/`).
#[derive(Debug, Clone)]
pub struct ChatParser<C>
where
    C: FromStr + Any + Send + Sync + 'static,
{
    prefixes: Vec<char>,
    _marker: PhantomData<C>,
}

impl<C> ChatParser<C>
where
    C: FromStr + Any + Send + Sync + 'static,
{
    /// Create a new parser that strips the given prefix characters before
    /// passing the message body to `C::from_str`.
    pub fn new(prefixes: &[char]) -> Self {
        Self {
            prefixes: prefixes.to_vec(),
            _marker: PhantomData,
        }
    }

    /// Parse one raw packet into a [`ChatEvent`], or `None` if the packet is
    /// not a user chat message or the body fails to parse as `C`.
    pub fn parse(&self, packet: &insim::Packet) -> Option<ChatEvent<C>> {
        let (msg, ucid) = match packet {
            insim::Packet::Mso(mso)
                if matches!(mso.usertype, MsoUserType::User | MsoUserType::Prefix) =>
            {
                (mso.msg_from_textstart().trim(), mso.ucid)
            },
            insim::Packet::Iii(iii) => (iii.msg.trim(), iii.ucid),
            _ => return None,
        };
        let msg = msg.trim_start_matches(&self.prefixes[..]);
        msg.parse::<C>().ok().map(|parsed| ChatEvent {
            parsed,
            original: packet.clone(),
            ucid,
        })
    }
}
