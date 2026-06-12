//! Admin command methods for [`Presence`].

use insim::{identifiers::ConnectionId, insim::PenaltyInfo};

use super::Presence;
use crate::util::host_command;

impl Presence {
    /// Returns a `/kick` packet for the given UCID, or `None` if not found.
    pub fn kick(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/kick {}", conn.uname)))
    }

    /// Returns a `/ban` packet. `ban_days = 0` means 12 hours (LFS convention).
    pub fn ban(&self, ucid: ConnectionId, ban_days: u32) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/ban {} {ban_days}", conn.uname)))
    }

    /// Returns an `/unban` packet.
    pub fn unban(&self, uname: impl Into<String>) -> insim::Packet {
        host_command(format!("/unban {}", uname.into()))
    }

    /// Returns a `/spec` packet for the given UCID, or `None` if not found.
    pub fn spec(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/spec {}", conn.uname)))
    }

    /// Returns a `/pitlane` packet for the given UCID, or `None` if not found.
    pub fn pitlane(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/pitlane {}", conn.uname)))
    }

    /// Returns a `/p_clear` packet for the given UCID, or `None` if not found.
    pub fn clear_penalty(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/p_clear {}", conn.uname)))
    }

    /// Returns the packets needed to set and display a Race Control Message.
    ///
    /// Pass [`ConnectionId::ALL`] to broadcast to all connections.
    /// Returns up to 2 packets; send them all.
    pub fn send_rcm(&self, message: &str, ucid: ConnectionId) -> Vec<insim::Packet> {
        let mut packets = vec![host_command(format!("/rcm {message}"))];
        if ucid == ConnectionId::ALL {
            packets.push(host_command("/rcm_all"));
        } else if let Some(conn) = self.get(ucid) {
            packets.push(host_command(format!("/rcm_ply {}", conn.uname)));
        }
        packets
    }

    /// Returns the packets needed to clear a Race Control Message.
    ///
    /// Pass [`ConnectionId::ALL`] to clear for everyone.
    pub fn clear_rcm(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        if ucid == ConnectionId::ALL {
            return Some(host_command("/rcc_all"));
        }
        let conn = self.get(ucid)?;
        Some(host_command(format!("/rcc_ply {}", conn.uname)))
    }

    /// Returns a penalty packet for the given UCID. Returns `None` if the
    /// UCID is not found or the penalty variant is not issueable.
    pub fn give_penalty(&self, ucid: ConnectionId, penalty: PenaltyInfo) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        let cmd = match penalty {
            PenaltyInfo::Dt => format!("/p_dt {}", conn.uname),
            PenaltyInfo::Sg => format!("/p_sg {}", conn.uname),
            PenaltyInfo::Seconds30 => format!("/p_30 {}", conn.uname),
            PenaltyInfo::Seconds45 => format!("/p_45 {}", conn.uname),
            _ => return None,
        };
        Some(host_command(cmd))
    }
}
