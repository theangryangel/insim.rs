//! Chat commands

use insim::{builder::InsimTask, insim::Mso};
use kitcar::chat::Parse;
use tokio::task::JoinHandle;

/// Chat runtime error.
pub type ChatError = kitcar::chat::RuntimeError;

// -- Event mode ---------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum EventChatMsg {
    /// Only valid during Idle scene - start the game
    Start,
    /// Only valid during Event scene - stops the game
    End,
    /// Echo a string back from the server
    Echo { message: String },
    /// Help
    Help,
    /// Quit
    Quit,
}

pub type EventChat = kitcar::chat::Chat<EventChatMsg>;

pub fn spawn_event(insim: InsimTask) -> (EventChat, JoinHandle<Result<(), ChatError>>) {
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

// -- Challenge mode -----------------------------------------------------------

#[derive(Debug, Clone, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum ChallengeChatMsg {
    /// Help
    Help,
    /// End the challenge
    End,
    /// Quit
    Quit,
}

pub type ChallengeChat = kitcar::chat::Chat<ChallengeChatMsg>;

pub fn spawn_challenge(insim: InsimTask) -> (ChallengeChat, JoinHandle<Result<(), ChatError>>) {
    kitcar::chat::spawn_with_handler(insim, 100, handle_challenge_chat)
}

async fn handle_challenge_chat(
    insim: InsimTask,
    mso: Mso,
    msg: ChallengeChatMsg,
) -> Result<(), ChatError> {
    match msg {
        ChallengeChatMsg::Help => {
            insim.send_message("Available commands:", mso.ucid).await?;
            for cmd in ChallengeChatMsg::help() {
                insim.send_message(cmd, mso.ucid).await?;
            }
        },
        _ => {},
    }
    Ok(())
}
