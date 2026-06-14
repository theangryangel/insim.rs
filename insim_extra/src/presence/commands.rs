//! Admin command methods for [`Presence`] and [`ConnectionInfo`].

use insim::{identifiers::ConnectionId, insim::PenaltyInfo};

use super::{ConnectionInfo, Presence};
use crate::util::host_command;

impl ConnectionInfo {
    /// Returns a `/kick` packet for this connection.
    pub fn kick(&self) -> insim::Packet {
        host_command(format!("/kick {}", self.uname))
    }

    /// Returns a `/ban` packet. `ban_days = 0` means 12 hours (LFS convention).
    pub fn ban(&self, ban_days: u32) -> insim::Packet {
        host_command(format!("/ban {} {ban_days}", self.uname))
    }

    /// Returns a `/spec` packet for this connection.
    pub fn spec(&self) -> insim::Packet {
        host_command(format!("/spec {}", self.uname))
    }

    /// Returns a `/pitlane` packet for this connection.
    pub fn pitlane(&self) -> insim::Packet {
        host_command(format!("/pitlane {}", self.uname))
    }

    /// Returns a `/p_clear` packet for this connection.
    pub fn clear_penalty(&self) -> insim::Packet {
        host_command(format!("/p_clear {}", self.uname))
    }

    /// Returns a penalty packet for this connection, or `None` for unissueable
    /// penalty variants.
    pub fn give_penalty(&self, penalty: PenaltyInfo) -> Option<insim::Packet> {
        let cmd = match penalty {
            PenaltyInfo::Dt => format!("/p_dt {}", self.uname),
            PenaltyInfo::Sg => format!("/p_sg {}", self.uname),
            PenaltyInfo::Seconds30 => format!("/p_30 {}", self.uname),
            PenaltyInfo::Seconds45 => format!("/p_45 {}", self.uname),
            _ => return None,
        };
        Some(host_command(cmd))
    }

    /// Returns the packets needed to set and display a Race Control Message for
    /// this connection. Send both packets.
    pub fn send_rcm(&self, message: &str) -> Vec<insim::Packet> {
        vec![
            host_command(format!("/rcm {message}")),
            host_command(format!("/rcm_ply {}", self.uname)),
        ]
    }

    /// Returns a packet to clear the Race Control Message for this connection.
    pub fn clear_rcm(&self) -> insim::Packet {
        host_command(format!("/rcc_ply {}", self.uname))
    }
}

impl Presence {
    /// Returns an `/unban` packet.
    pub fn unban(&self, uname: impl Into<String>) -> insim::Packet {
        host_command(format!("/unban {}", uname.into()))
    }

    /// Returns a `/kick` packet for the given UCID, or `None` if not found.
    pub fn kick(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        Some(self.get(ucid)?.kick())
    }

    /// Returns a `/ban` packet. `ban_days = 0` means 12 hours (LFS convention).
    pub fn ban(&self, ucid: ConnectionId, ban_days: u32) -> Option<insim::Packet> {
        Some(self.get(ucid)?.ban(ban_days))
    }

    /// Returns a `/spec` packet for the given UCID, or `None` if not found.
    pub fn spec(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        Some(self.get(ucid)?.spec())
    }

    /// Returns a `/pitlane` packet for the given UCID, or `None` if not found.
    pub fn pitlane(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        Some(self.get(ucid)?.pitlane())
    }

    /// Returns a `/p_clear` packet for the given UCID, or `None` if not found.
    pub fn clear_penalty(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        Some(self.get(ucid)?.clear_penalty())
    }

    /// Returns a penalty packet for the given UCID. Returns `None` if the
    /// UCID is not found or the penalty variant is not issueable.
    pub fn give_penalty(&self, ucid: ConnectionId, penalty: PenaltyInfo) -> Option<insim::Packet> {
        self.get(ucid)?.give_penalty(penalty)
    }

    /// Returns the packets needed to set and display a Race Control Message.
    ///
    /// Pass [`ConnectionId::ALL`] to broadcast to all connections.
    /// Returns up to 2 packets; send them all.
    pub fn send_rcm(&self, message: &str, ucid: ConnectionId) -> Vec<insim::Packet> {
        if ucid == ConnectionId::ALL {
            return vec![
                host_command(format!("/rcm {message}")),
                host_command("/rcm_all"),
            ];
        }
        self.get(ucid)
            .map(|conn| conn.send_rcm(message))
            .unwrap_or_default()
    }

    /// Returns the packets needed to clear a Race Control Message.
    ///
    /// Pass [`ConnectionId::ALL`] to clear for everyone.
    pub fn clear_rcm(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        if ucid == ConnectionId::ALL {
            return Some(host_command("/rcc_all"));
        }
        Some(self.get(ucid)?.clear_rcm())
    }
}
