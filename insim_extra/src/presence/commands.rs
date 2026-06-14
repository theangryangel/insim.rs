//! Admin command methods for [`ConnectionInfo`].

use insim::insim::PenaltyInfo;

use super::ConnectionInfo;
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
