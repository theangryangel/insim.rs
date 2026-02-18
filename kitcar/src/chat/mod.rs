//! Chat command parsing and runtime helpers.
//!
//! The derive macro handles command parsing, while [`spawn_with_handler`] gives you a
//! reusable runtime loop that parses `Mso` packets, runs optional side-effects, and
//! publishes parsed commands on a broadcast bus.
//!
//! ```rust,ignore
//! use insim::{builder::InsimTask, insim::Mso};
//! use kitcar::chat::{self, Parse};
//! use tokio::task::JoinHandle;
//!
//! #[derive(Debug, Clone, PartialEq, kitcar::chat::Parse)]
//! #[chat(prefix = '!')]
//! enum ChatMsg {
//!     Start,
//!     Echo { message: String },
//!     Help,
//! }
//!
//! type Chat = chat::Chat<ChatMsg>;
//!
//! fn spawn_chat(insim: InsimTask) -> (Chat, JoinHandle<Result<(), chat::RuntimeError>>) {
//!     chat::spawn_with_handler(insim, 100, handle_chat_command)
//! }
//!
//! async fn handle_chat_command(
//!     insim: InsimTask,
//!     mso: Mso,
//!     msg: ChatMsg,
//! ) -> Result<(), chat::RuntimeError> {
//!     match msg {
//!         ChatMsg::Echo { message } => {
//!             insim.send_message(format!("Echo: {message}"), mso.ucid).await?;
//!         },
//!         ChatMsg::Help => {
//!             insim.send_message("Available commands:", mso.ucid).await?;
//!             for line in ChatMsg::help() {
//!                 insim.send_message(line, mso.ucid).await?;
//!             }
//!         },
//!         ChatMsg::Start => {},
//!     }
//!
//!     Ok(())
//! }
//! ```
use std::future::Future;

use insim::{builder::InsimTask, identifiers::ConnectionId};
pub use kitcar_macros::ParseChat as Parse;
use tokio::{sync::broadcast, task::JoinHandle};

use crate::{presence, scenes::SceneError};

#[derive(Debug, Clone)]
pub struct Chat<C> {
    broadcast: broadcast::Sender<(ConnectionId, C)>,
}

impl<C> Chat<C> {
    pub fn subscribe(&self) -> broadcast::Receiver<(ConnectionId, C)> {
        self.broadcast.subscribe()
    }
}

impl<C: Clone> Chat<C> {
    /// Wait for an admin to send a specific chat command.
    /// Reminder, you should probably pin this if its use in a loop { tokio::select! { .. } }
    pub async fn wait_for_admin_cmd<F>(
        &self,
        presence: presence::Presence,
        matches: F,
    ) -> Result<(), SceneError>
    where
        F: Fn(&C) -> bool,
    {
        let mut chat = self.subscribe();

        loop {
            match chat.recv().await {
                Ok((ucid, msg)) if matches(&msg) => {
                    if let Some(conn) =
                        presence
                            .connection(&ucid)
                            .await
                            .map_err(|cause| SceneError::Custom {
                                scene: "wait_for_admin_cmd::connection",
                                cause: Box::new(cause),
                            })?
                        && conn.admin
                    {
                        return Ok(());
                    }
                },
                Ok(_) => {},
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Chat commands lost due to lag");
                },
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(SceneError::Custom {
                        scene: "wait_for_admin_cmd",
                        cause: Box::new(ChatError::HandleLost),
                    });
                },
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("Lost Chat channel")]
    HandleLost,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Insim: {0}")]
    Insim(#[from] insim::Error),
    #[error("Lost Insim packet stream")]
    InsimHandleLost,
}

pub type SpawnTask = JoinHandle<Result<(), RuntimeError>>;

/// Spawn chat parsing with no command side-effects.
pub fn spawn<C>(insim: InsimTask, capacity: usize) -> (Chat<C>, SpawnTask)
where
    C: Parse + Clone + Send + 'static,
{
    spawn_with_handler(insim, capacity, |_insim, _mso, _cmd| async { Ok(()) })
}

/// Spawn chat parsing with a custom per-command async handler.
pub fn spawn_with_handler<C, H, Fut>(
    insim: InsimTask,
    capacity: usize,
    mut handler: H,
) -> (Chat<C>, SpawnTask)
where
    C: Parse + Clone + Send + 'static,
    H: FnMut(InsimTask, insim::insim::Mso, C) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), RuntimeError>> + Send + 'static,
{
    let (tx, _rx) = broadcast::channel(capacity);

    let chat = Chat {
        broadcast: tx.clone(),
    };

    let handle = tokio::spawn(async move {
        let mut packets = insim.subscribe();

        loop {
            match packets.recv().await {
                Ok(insim::Packet::Mso(mso)) => match C::parse(mso.msg_from_textstart()) {
                    Ok(cmd) => {
                        handler(insim.clone(), mso.clone(), cmd.clone()).await?;
                        let _ = tx.send((mso.ucid, cmd));
                    },
                    Err(ParseError::MissingPrefix(_)) => {},
                    Err(_) => {},
                },
                Ok(_) => {},
                Err(_) => return Err(RuntimeError::InsimHandleLost),
            }
        }
    });

    (chat, handle)
}

pub trait Parse: Sized {
    fn parse(input: &str) -> Result<Self, ParseError>;
    fn help() -> Vec<&'static str>;
    fn prefix() -> Option<char>;
}

pub trait FromArg: Sized {
    fn from_arg(s: &str) -> Result<Self, String>;
}

impl FromArg for String {
    fn from_arg(s: &str) -> Result<Self, String> {
        Ok(s.to_string())
    }
}

impl FromArg for i32 {
    fn from_arg(s: &str) -> Result<Self, String> {
        s.parse()
            .map_err(|_| format!("'{}' is not a valid number", s))
    }
}

impl FromArg for f32 {
    fn from_arg(s: &str) -> Result<Self, String> {
        s.parse()
            .map_err(|_| format!("'{}' is not a valid number", s))
    }
}

impl FromArg for bool {
    fn from_arg(s: &str) -> Result<Self, String> {
        if s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes") {
            return Ok(true);
        }

        if s == "0" || s.eq_ignore_ascii_case("false") || s.eq_ignore_ascii_case("no") {
            return Ok(false);
        }

        Err(format!("'{}' is not a valid boolean", s))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    EmptyInput,
    UnknownCommand(String),
    MissingRequiredArg(String),
    InvalidArg(String, String),
    MissingPrefix(char),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Empty input"),
            ParseError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            ParseError::MissingRequiredArg(arg) => write!(f, "Missing required argument: {}", arg),
            ParseError::InvalidArg(arg, msg) => write!(f, "Invalid argument '{}': {}", arg, msg),
            ParseError::MissingPrefix(prefix) => write!(f, "Command must start with '{}'", prefix),
        }
    }
}
