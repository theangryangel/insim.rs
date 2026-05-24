//! Small packet-building helpers.

use insim::{
    identifiers::ConnectionId,
    insim::{Msl, Mst, Mtc, SoundType},
};

/// Build a message-to-connection packet.
///
/// - `ucid: Some(u)` targets a specific UCID (use `0` for host console only, `255` for all).
/// - `ucid: None` sends an [`Msl`] to the local computer (handy when there's no
///   particular connection to address - e.g. a periodic ticker on a host bot).
pub fn mtc(text: impl Into<String>, ucid: Option<ConnectionId>) -> Vec<insim::Packet> {
    text.into()
        .lines()
        .map(|chunk| match ucid {
            Some(u) => insim::Packet::from(Mtc {
                ucid: u,
                text: chunk.to_string(),
                sound: SoundType::Silent,
                ..Default::default()
            }),
            None => insim::Packet::from(Msl {
                msg: chunk.to_string(),
                sound: SoundType::Silent,
                ..Default::default()
            }),
        })
        .collect()
}

/// Build a host-command packet (`Mst`) for `/command`-style server commands
/// such as `/kick <name>`, `/track <track>`, `/restart`, etc.
pub fn host_command(text: impl Into<String>) -> insim::Packet {
    insim::Packet::from(Mst {
        msg: text.into(),
        ..Default::default()
    })
}
