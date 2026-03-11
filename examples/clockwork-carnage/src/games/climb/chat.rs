//! Climb mode chat commands

use insim::{builder::InsimTask, insim::Mso};
use kitcar::chat::Parse;
use tokio::task::JoinHandle;

use crate::ChatError;

#[derive(Debug, Clone, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum ClimbChatMsg {
    /// Help
    Help,
    /// End the climb session
    End,
    /// Quit
    Quit,
}

pub type ClimbChat = kitcar::chat::Chat<ClimbChatMsg>;

pub fn spawn(insim: InsimTask) -> (ClimbChat, JoinHandle<Result<(), ChatError>>) {
    kitcar::chat::spawn_with_handler(insim, 100, handle_climb_chat)
}

async fn handle_climb_chat(
    insim: InsimTask,
    mso: Mso,
    msg: ClimbChatMsg,
) -> Result<(), ChatError> {
    if msg == ClimbChatMsg::Help {
        insim.send_message("Available commands:", mso.ucid).await?;
        for cmd in ClimbChatMsg::help() {
            insim.send_message(cmd, mso.ucid).await?;
        }
    }
    Ok(())
}
