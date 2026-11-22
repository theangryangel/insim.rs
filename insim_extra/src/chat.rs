//! Generic chat message parser.
//!
//! [`ChatParser`] parses every `Mso`/`Iii` packet into a typed value `C` via
//! [`std::str::FromStr`]. No `Sender` or framework dependency.
//!
//! `C` can be written by hand, or derived with [`ChatCommand`](macro@ChatCommand):
//!
//! ```
//! use insim_extra::chat::ChatCommand;
//!
//! #[derive(Debug, PartialEq, ChatCommand)]
//! enum Cmd {
//!     /// Show the help dialog.
//!     Help,
//!     /// Kick a player by username.
//!     Kick { uname: String },
//!     /// Ban a player for some minutes.
//!     #[chat(alias = "b")]
//!     Ban { uname: String, mins: Option<u32> },
//!     /// Say something to everyone.
//!     Echo { msg: Option<String> },
//! }
//!
//! assert_eq!("help".parse(), Ok(Cmd::Help));
//! assert_eq!("b bob 10".parse(), Ok(Cmd::Ban { uname: "bob".into(), mins: Some(10) }));
//! assert_eq!(
//!     "echo hello there".parse(),
//!     Ok(Cmd::Echo { msg: Some("hello there".into()) })
//! );
//! assert_eq!(Cmd::usage("ban").unwrap().to_string(), "ban <uname> [mins]");
//! ```

use std::{any::Any, fmt, marker::PhantomData, str::FromStr};

use insim::{identifiers::ConnectionId, insim::MsoUserType};
/// Derive [`FromStr`] and [`ChatCommand`](trait@ChatCommand) for an enum of
/// chat commands. See [`ChatCommand`](trait@ChatCommand) for the rules.
pub use insim_extra_macros::ChatCommand;

/// A successfully parsed chat message.
#[derive(Debug, Clone)]
pub struct ChatEvent<C>
where
    C: FromStr + Any + Send + Sync + 'static,
{
    /// Connection that sent the message.
    pub ucid: ConnectionId,
    /// Parsed message value.
    pub parsed: C,
    /// Original raw packet.
    pub original: insim::Packet,
}

/// Generic chat message parser.
///
/// Parses every `Mso` (user chat) or `Iii` (in-game text input) packet body
/// into a typed value `C` via [`FromStr`]. On parse failure no event is emitted
/// and no error is returned.
///
/// Prefix characters are stripped before parsing (e.g. `!`, `/`).
#[derive(Debug, Clone)]
pub struct ChatParser<C>
where
    C: FromStr + Any + Send + Sync + 'static,
{
    prefixes: Vec<char>,
    _marker: PhantomData<C>,
}

impl<C> ChatParser<C>
where
    C: FromStr + Any + Send + Sync + 'static,
{
    /// Create a new parser that strips the given prefix characters before
    /// passing the message body to `C::from_str`.
    pub fn new(prefixes: &[char]) -> Self {
        Self {
            prefixes: prefixes.to_vec(),
            _marker: PhantomData,
        }
    }

    /// Parse one raw packet into a [`ChatEvent`], or `None` if the packet is
    /// not a user chat message or the body fails to parse as `C`.
    pub fn parse(&self, packet: &insim::Packet) -> Option<ChatEvent<C>> {
        let (msg, ucid) = match packet {
            insim::Packet::Mso(mso)
                if matches!(mso.usertype, MsoUserType::User | MsoUserType::Prefix) =>
            {
                (mso.msg_from_textstart().trim(), mso.ucid)
            },
            insim::Packet::Iii(iii) => (iii.msg.trim(), iii.ucid),
            _ => return None,
        };
        let msg = msg.trim_start_matches(&self.prefixes[..]);
        msg.parse::<C>().ok().map(|parsed| ChatEvent {
            parsed,
            original: packet.clone(),
            ucid,
        })
    }
}

/// An enum of chat commands, usually derived with
/// [`#[derive(ChatCommand)]`](macro@ChatCommand).
///
/// Each variant is one command, matched case-insensitively against the first
/// word of the message:
///
/// - The name is the variant name in `snake_case`. Change it with
///   `#[chat(rename = "...")]`, and add others with `#[chat(alias = "...")]`.
/// - Unit variants take no arguments. Struct variants take one word per field,
///   in order, each parsed with [`FromStr`].
/// - `Option<T>` fields are optional. They must come after every required one.
/// - A last field of type `String` or `Option<String>` takes the rest of the
///   line, spaces included.
/// - Anything left over is an error ([`ChatParseError::TooMany`]).
/// - The variant's doc comment becomes its [`CommandUsage::help`].
///
/// `Option` and `String` are recognised by name, so type aliases of them are
/// treated as ordinary arguments. There is no quoting: use a username, not a
/// player name, to name a player.
pub trait ChatCommand: FromStr<Err = ChatParseError> {
    /// Every command, in declaration order.
    const COMMANDS: &'static [CommandUsage];

    /// The command called `name` (or with `name` as an alias), ignoring case.
    fn usage(name: &str) -> Option<&'static CommandUsage> {
        Self::COMMANDS.iter().find(|c| c.matches(name))
    }
}

/// Describes one [`ChatCommand`]: its name, arguments and help text. Displays
/// as a usage line such as `ban <uname> [mins]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandUsage {
    name: &'static str,
    aliases: &'static [&'static str],
    args: &'static str,
    help: &'static str,
}

impl CommandUsage {
    #[doc(hidden)]
    pub const fn __new(
        name: &'static str,
        aliases: &'static [&'static str],
        args: &'static str,
        help: &'static str,
    ) -> Self {
        Self {
            name,
            aliases,
            args,
            help,
        }
    }
    /// The command's name.
    pub fn name(&self) -> &'static str {
        self.name
    }
    /// Other names for the command.
    pub fn aliases(&self) -> &'static [&'static str] {
        self.aliases
    }
    /// The arguments, such as `<uname> [mins] [reason...]`: `<required>`,
    /// `[optional]`, and `...` for the rest of the line.
    pub fn args(&self) -> &'static str {
        self.args
    }
    /// The variant's doc comment, joined into one line.
    pub fn help(&self) -> &'static str {
        self.help
    }
    /// Whether `name` is this command's name or one of its aliases, ignoring case.
    pub fn matches(&self, name: &str) -> bool {
        std::iter::once(&self.name)
            .chain(self.aliases)
            .any(|n| n.eq_ignore_ascii_case(name))
    }
}

impl fmt::Display for CommandUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)?;
        if !self.args.is_empty() {
            write!(f, " {}", self.args)?;
        }
        Ok(())
    }
}

/// Why a message did not parse as a [`ChatCommand`]. Errors about a known
/// command carry its name, for looking up its [`ChatCommand::usage`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ChatParseError {
    /// The message was empty.
    #[error("empty command")]
    Empty,
    /// No command has this name.
    #[error("unknown command `{0}`")]
    Unknown(String),
    /// A required argument was not given.
    #[error("`{command}` needs <{arg}>")]
    Missing {
        /// The command's name.
        command: &'static str,
        /// The missing argument.
        arg: &'static str,
    },
    /// An argument did not parse.
    #[error("`{command}`: <{arg}> cannot be `{value}`")]
    Invalid {
        /// The command's name.
        command: &'static str,
        /// The argument.
        arg: &'static str,
        /// What was given.
        value: String,
    },
    /// There was more after the last argument.
    #[error("too many arguments for `{command}`")]
    TooMany {
        /// The command's name.
        command: &'static str,
    },
}

#[doc(hidden)]
pub mod __private {
    //! Used by the code `#[derive(ChatCommand)]` generates. Not public API.
    use std::str::FromStr;

    use super::ChatParseError;

    /// The words of a message, read from the front.
    #[derive(Debug)]
    pub struct Args<'a>(&'a str);

    impl<'a> Args<'a> {
        pub fn new(input: &'a str) -> Self {
            Self(input)
        }

        fn word(&mut self) -> Option<&'a str> {
            let s = self.0.trim_start();
            if s.is_empty() {
                self.0 = s;
                return None;
            }
            let end = s.find(char::is_whitespace).unwrap_or(s.len());
            let (word, rest) = s.split_at(end);
            self.0 = rest;
            Some(word)
        }

        pub fn command(&mut self) -> Result<&'a str, ChatParseError> {
            self.word().ok_or(ChatParseError::Empty)
        }

        fn parse<T: FromStr>(
            command: &'static str,
            arg: &'static str,
            word: &str,
        ) -> Result<T, ChatParseError> {
            word.parse().map_err(|_| ChatParseError::Invalid {
                command,
                arg,
                value: word.to_owned(),
            })
        }

        pub fn required<T: FromStr>(
            &mut self,
            command: &'static str,
            arg: &'static str,
        ) -> Result<T, ChatParseError> {
            let word = self
                .word()
                .ok_or(ChatParseError::Missing { command, arg })?;
            Self::parse(command, arg, word)
        }

        pub fn optional<T: FromStr>(
            &mut self,
            command: &'static str,
            arg: &'static str,
        ) -> Result<Option<T>, ChatParseError> {
            self.word()
                .map(|word| Self::parse(command, arg, word))
                .transpose()
        }

        pub fn rest(
            &mut self,
            command: &'static str,
            arg: &'static str,
        ) -> Result<String, ChatParseError> {
            self.rest_optional()
                .ok_or(ChatParseError::Missing { command, arg })
        }

        pub fn rest_optional(&mut self) -> Option<String> {
            let rest = self.0.trim();
            self.0 = "";
            (!rest.is_empty()).then(|| rest.to_owned())
        }

        pub fn finish(self, command: &'static str) -> Result<(), ChatParseError> {
            if self.0.trim().is_empty() {
                Ok(())
            } else {
                Err(ChatParseError::TooMany { command })
            }
        }
    }
}
