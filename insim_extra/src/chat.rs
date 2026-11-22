//! Generic chat message parser.
//!
//! [`ChatParser`] parses `Mso`/`Iii` packets into any `C` that implements
//! [`std::str::FromStr`]. No `Sender` or framework needed.
//!
//! Write `C` by hand, or derive it with [`ChatCommand`](macro@ChatCommand):
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
/// Parses `Mso` (user chat) and `Iii` (text input) packets into `C` via
/// [`FromStr`], after stripping prefix characters such as `!`. Messages that
/// don't parse are skipped silently.
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
    /// Create a parser that strips `prefixes` before parsing.
    pub fn new(prefixes: &[char]) -> Self {
        Self {
            prefixes: prefixes.to_vec(),
            _marker: PhantomData,
        }
    }

    /// Parse a packet into a [`ChatEvent`]. `None` if it isn't user chat or
    /// doesn't parse.
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
/// Each variant is one command, matched case-insensitively against the
/// message's first word:
///
/// - Name: the variant in `snake_case`. Override with `#[chat(rename = "...")]`;
///   add more with `#[chat(alias = "...")]`.
/// - Unit variants take no arguments; struct variants take one word per field,
///   in order, via [`FromStr`].
/// - `Option<T>` fields are optional and must follow every required one.
/// - A last `String` or `Option<String>` field takes the rest of the line,
///   spaces and all.
/// - Leftover words are an error ([`ChatParseError::TooMany`]).
/// - The variant's doc comment is its [`CommandUsage::help`].
/// - `#[chat(admin)]` marks it admin-only ([`CommandUsage::admin`]). Parsing
///   ignores this; the caller enforces it, as kitcar's `Router::commands` does.
///
/// `Option` and `String` are matched by name, so type aliases of them are
/// ordinary arguments. There's no quoting, so name players by username: player
/// names can contain spaces.
pub trait ChatCommand: FromStr<Err = ChatParseError> {
    /// Every command, in declaration order.
    const COMMANDS: &'static [CommandUsage];

    /// Look up a command by name or alias, ignoring case.
    fn usage(name: &str) -> Option<&'static CommandUsage> {
        Self::COMMANDS.iter().find(|c| c.matches(name))
    }
}

/// One [`ChatCommand`]: name, arguments and help. Displays as a usage line,
/// such as `ban <uname> [mins]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandUsage {
    name: &'static str,
    aliases: &'static [&'static str],
    args: &'static str,
    help: &'static str,
    admin: bool,
}

impl CommandUsage {
    #[doc(hidden)]
    pub const fn __new(
        name: &'static str,
        aliases: &'static [&'static str],
        args: &'static str,
        help: &'static str,
        admin: bool,
    ) -> Self {
        Self {
            name,
            aliases,
            args,
            help,
            admin,
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
    /// The arguments, such as `<uname> [mins] [reason...]`. `...` marks the
    /// rest of the line.
    pub fn args(&self) -> &'static str {
        self.args
    }
    /// The variant's doc comment, on one line.
    pub fn help(&self) -> &'static str {
        self.help
    }
    /// Admin-only, via `#[chat(admin)]`.
    pub fn admin(&self) -> bool {
        self.admin
    }
    /// Whether `name` is this command's name or an alias, ignoring case.
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

/// Why a message didn't parse as a [`ChatCommand`]. Errors about a known
/// command carry its name, so you can look up its [`ChatCommand::usage`].
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
    /// Words were left after the last argument.
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
