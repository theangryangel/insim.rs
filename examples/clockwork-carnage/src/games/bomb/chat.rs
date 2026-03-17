//! Bomb mode chat commands

use insim::{builder::InsimTask, insim::Mso};
use kitcar::chat::Parse;
use tokio::task::JoinHandle;

use crate::ChatError;

#[derive(Debug, Clone, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum BombChatMsg {
    /// Help
    Help,
    /// End the bomb session
    End,
    /// Quit
    Quit,
}

pub type BombChat = kitcar::chat::Chat<BombChatMsg>;

pub fn spawn(insim: InsimTask) -> (BombChat, JoinHandle<Result<(), ChatError>>) {
    kitcar::chat::spawn_with_handler(insim, 100, handle_bomb_chat)
}

async fn handle_bomb_chat(insim: InsimTask, mso: Mso, msg: BombChatMsg) -> Result<(), ChatError> {
    if msg == BombChatMsg::Help {
        insim.send_message("Available commands:", mso.ucid).await?;
        for cmd in BombChatMsg::help() {
            insim.send_message(cmd, mso.ucid).await?;
        }
    }
    Ok(())
}
