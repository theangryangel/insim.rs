//! Small helpers used by examples.

use insim::{
    identifiers::ConnectionId,
    insim::{Msl, Mtc, SoundType},
};

/// Build a message-to-connection packet.
///
/// - `ucid: Some(u)` targets a specific UCID (use `0` for host console only, `255` for all).
/// - `ucid: None` sends an [`Msl`] to the local computer (handy when there's no
///   particular connection to address - e.g. a periodic ticker on a host bot).
pub fn mtc(text: impl Into<String>, ucid: Option<ConnectionId>) -> insim::Packet {
    let text = text.into();
    match ucid {
        Some(u) => insim::Packet::from(Mtc {
            ucid: u,
            text,
            sound: SoundType::Silent,
            ..Default::default()
        }),
        None => insim::Packet::from(Msl {
            msg: text,
            sound: SoundType::Silent,
            ..Default::default()
        }),
    }
}
