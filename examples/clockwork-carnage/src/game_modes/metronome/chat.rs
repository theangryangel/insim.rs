//! Event mode chat commands

use insim::{builder::InsimTask, insim::Mso};
use kitcar::chat::Parse;
use tokio::task::JoinHandle;

use crate::ChatError;

#[derive(Debug, Clone, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum EventChatMsg {
    /// Echo a string back from the server
    Echo { message: String },
    /// Help
    Help,
    /// Quit
    Quit,
}

pub type EventChat = kitcar::chat::Chat<EventChatMsg>;

pub fn spawn(insim: InsimTask) -> (EventChat, JoinHandle<Result<(), ChatError>>) {
    kitcar::chat::spawn_with_handler(insim, 100, handle_event_chat)
}

async fn handle_event_chat(insim: InsimTask, mso: Mso, msg: EventChatMsg) -> Result<(), ChatError> {
    match msg {
        EventChatMsg::Echo { message } => {
            insim
                .send_message(format!("Echo: {message}"), mso.ucid)
                .await?;
        },
        EventChatMsg::Help => {
            insim.send_message("Available commands:", mso.ucid).await?;
            for cmd in EventChatMsg::help() {
                insim.send_message(cmd, mso.ucid).await?;
            }
        },
        _ => {},
    }
    Ok(())
}
