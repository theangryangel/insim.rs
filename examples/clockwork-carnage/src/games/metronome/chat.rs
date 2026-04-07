//! Event mode chat commands

use insim::{builder::InsimTask, insim::Mso};
use insim_extras::chat::Parse;
use tokio::task::JoinHandle;

use crate::ChatError;

#[derive(Debug, Clone, PartialEq, insim_extras::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum EventChatMsg {
    /// Echo a string back from the server
    Echo { message: String },
    /// Help
    Help,
}

pub type EventChat = insim_extras::chat::Chat<EventChatMsg>;

pub fn spawn(insim: InsimTask) -> (EventChat, JoinHandle<Result<(), ChatError>>) {
    insim_extras::chat::spawn_with_handler(insim, 100, handle_event_chat)
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
    }
    Ok(())
}
