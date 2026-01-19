use insim::identifiers::ConnectionId;
use kitcar::{presence, scenes::SceneError};
use tokio::sync::broadcast;

use crate::chat;

/// Wait for an admin to send a specific chat command.
pub async fn wait_for_admin_cmd<F>(
    chat: &mut broadcast::Receiver<(chat::ChatMsg, ConnectionId)>,
    presence: presence::Presence,
    matches: F,
) -> Result<(), SceneError>
where
    F: Fn(&chat::ChatMsg) -> bool,
{
    loop {
        match chat.recv().await {
            Ok((msg, ucid)) if matches(&msg) => {
                if let Some(conn) = presence.connection(&ucid).await {
                    if conn.admin {
                        return Ok(());
                    }
                }
            },
            Ok(_) => {},
            Err(broadcast::error::RecvError::Lagged(_)) => {
                tracing::warn!("Chat commands lost due to lag");
            },
            Err(broadcast::error::RecvError::Closed) => {
                return Err(SceneError::Custom {
                    scene: "wait_for_admin_cmd",
                    cause: Box::new(chat::ChatError::HandleLost),
                });
            },
        }
    }
}
