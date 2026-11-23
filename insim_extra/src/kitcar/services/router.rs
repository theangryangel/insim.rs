//! [`Router`]: a [`Service`] built from handlers. Ideal for bots that mostly
//! react to events and chat commands.
//!
//! ```ignore
//! #[derive(ChatCommand)]
//! enum Cmd {
//!     /// Say hello.
//!     Hello,
//!     /// Kick a player by username.
//!     #[chat(admin)]
//!     Kick { uname: String },
//! }
//!
//! struct Bot { greeted: u32 }
//!
//! fn joined(bot: &mut Bot, event: &Connected, _: &Context) -> Effects<()> {
//!     bot.greeted += 1;
//!     Effects::new().message("Welcome!", event.info.ucid)
//! }
//!
//! fn command(bot: &mut Bot, caller: Caller, cmd: Cmd, _: &Context) -> Effects<()> {
//!     match cmd {
//!         Cmd::Hello => caller.reply(format!("hi {}", caller.uname)),
//!         Cmd::Kick { uname } => Effects::new().packet(Mst::new(format!("/kick {uname}"))),
//!     }
//! }
//!
//! host.start(Router::new(Bot { greeted: 0 }).on(joined).commands(command)).await?;
//! ```
//!
//! Every matching handler runs, in the order added, and their effects combine.
//! Test a `Router` through [`Service::update`], or test the handlers directly.

use std::fmt;

use insim::{Packet, identifiers::ConnectionId, insim::MsoUserType};

use crate::{
    chat::{ChatCommand, ChatParseError},
    kitcar::{Context, Effects, Input, Service, input::FromInput},
    ui::{Node, Ui},
};

type Handler<S, M> = Box<dyn FnMut(&mut S, &Input<M>, &Context<'_>) -> Effects<M> + Send>;
type View<S, M> =
    Box<dyn Fn(&S, ConnectionId, &mut Ui<'_, M>, &Context<'_>) -> Option<Node<M>> + Send>;

/// A [`Service`] built from handlers over state `S`, with messages `M`.
///
/// Add handlers with [`on`](Self::on) (packets, world events, [`Started`],
/// [`Stopping`]), [`on_msg`](Self::on_msg) (the router's own messages: timers,
/// tasks, button clicks) and [`commands`](Self::commands) (chat commands).
/// Draw buttons with [`view`](Self::view).
///
/// Handlers run one at a time, in order, with `&mut S`, so the state needs no
/// locks. They return anything [`IntoEffects`]: [`Effects`], `()` or
/// `Option<Effects>`.
///
/// [`Started`]: crate::kitcar::input::Started
/// [`Stopping`]: crate::kitcar::input::Stopping
pub struct Router<S, M = ()> {
    state: S,
    handlers: Vec<Handler<S, M>>,
    view: Option<View<S, M>>,
}

impl<S: fmt::Debug, M> fmt::Debug for Router<S, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Router")
            .field("state", &self.state)
            .field("handlers", &self.handlers.len())
            .field("view", &self.view.is_some())
            .finish()
    }
}

impl<S: Send + 'static> Router<S> {
    /// A router with no messages. For timers, tasks or buttons, use
    /// [`with_messages`](Self::with_messages).
    pub fn new(state: S) -> Self {
        Self::with_messages(state)
    }
}

impl<S: Send + 'static, M: Clone + Send + 'static> Router<S, M> {
    /// A router with messages `M`: `Router::<_, Msg>::with_messages(state)`.
    pub fn with_messages(state: S) -> Self {
        Self {
            state,
            handlers: Vec::new(),
            view: None,
        }
    }

    /// The state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// The state, mutably.
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    /// Drop the handlers and keep the state.
    pub fn into_state(self) -> S {
        self.state
    }

    fn handler(
        mut self,
        handler: impl FnMut(&mut S, &Input<M>, &Context<'_>) -> Effects<M> + Send + 'static,
    ) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    /// Run `handler` for every input that carries a `T`: a packet type such as
    /// [`Uco`](insim::insim::Uco), a world event such as
    /// [`Connected`](crate::world::event::Connected), or [`Started`](crate::kitcar::input::Started)
    /// and [`Stopping`](crate::kitcar::input::Stopping). See [`FromInput`].
    pub fn on<T, R>(
        self,
        mut handler: impl FnMut(&mut S, &T, &Context<'_>) -> R + Send + 'static,
    ) -> Self
    where
        T: FromInput + ?Sized,
        R: IntoEffects<M>,
    {
        self.handler(move |state, input, cx| match input.get::<T>() {
            Some(value) => handler(state, value, cx).into_effects(),
            None => Effects::none(),
        })
    }

    /// Run `handler` for the router's own messages: timers, task results and
    /// button clicks.
    pub fn on_msg<R>(
        self,
        mut handler: impl FnMut(&mut S, M, &Context<'_>) -> R + Send + 'static,
    ) -> Self
    where
        R: IntoEffects<M>,
    {
        self.handler(move |state, input, cx| match input {
            Input::Msg(msg) => handler(state, msg.clone(), cx).into_effects(),
            _ => Effects::none(),
        })
    }

    /// Run `handler` for each chat command `C`.
    ///
    /// A command is any `/i` message, or chat starting with the InSim prefix
    /// ([`Isi::prefix`](insim::insim::Isi::prefix)). Names `C` doesn't know are
    /// ignored, so several routers can each handle their own commands.
    ///
    /// Instead of calling `handler`, the router replies to the sender when a
    /// non-admin uses a `#[chat(admin)]` command, or shows the usage when the
    /// arguments don't parse.
    pub fn commands<C, R>(
        self,
        mut handler: impl FnMut(&mut S, Caller, C, &Context<'_>) -> R + Send + 'static,
    ) -> Self
    where
        C: ChatCommand,
        R: IntoEffects<M>,
    {
        self.handler(move |state, input, cx| {
            let Some((ucid, prefix, text)) = command_text(input) else {
                return Effects::none();
            };
            let Some(usage) = text.split_whitespace().next().and_then(C::usage) else {
                return Effects::none();
            };
            let Some(caller) = Caller::new(ucid, cx) else {
                return Effects::none();
            };
            if usage.admin() && !caller.admin {
                return caller.reply(format!("^1{prefix}{} is for admins only", usage.name()));
            }
            match text.parse::<C>() {
                Ok(cmd) => handler(state, caller, cmd, cx).into_effects(),
                Err(ChatParseError::Unknown(_) | ChatParseError::Empty) => Effects::none(),
                Err(err) => caller.reply(format!("^1{err}^8 - usage: {prefix}{usage}")),
            }
        })
    }

    /// Draw one player's buttons from the state. See [`Service::view`].
    pub fn view(
        mut self,
        view: impl Fn(&S, ConnectionId, &mut Ui<'_, M>, &Context<'_>) -> Option<Node<M>>
        + Send
        + 'static,
    ) -> Self {
        self.view = Some(Box::new(view));
        self
    }
}

/// The sender, the prefix typed (for usage messages) and the command text.
pub(crate) fn command_text<M>(input: &Input<M>) -> Option<(ConnectionId, String, &str)> {
    match input.get::<Packet>()? {
        Packet::Mso(mso) if mso.usertype == MsoUserType::Prefix => {
            let msg = mso.msg_from_textstart().trim_start();
            let prefix = msg.chars().next()?;
            Some((mso.ucid, prefix.to_string(), &msg[prefix.len_utf8()..]))
        },
        Packet::Iii(iii) => Some((iii.ucid, "/i ".to_owned(), iii.msg.as_str())),
        _ => None,
    }
}

impl<S: Send + 'static, M: Clone + Send + 'static> Service for Router<S, M> {
    type Msg = M;

    fn update(&mut self, input: Input<M>, cx: &Context<'_>) -> Effects<M> {
        let state = &mut self.state;
        self.handlers
            .iter_mut()
            .fold(Effects::new(), |effects, handler| {
                effects.and(handler(state, &input, cx))
            })
    }

    fn view(&self, ucid: ConnectionId, ui: &mut Ui<'_, M>, cx: &Context<'_>) -> Option<Node<M>> {
        self.view
            .as_ref()
            .and_then(|view| view(&self.state, ucid, ui, cx))
    }
}

/// Who sent a chat command.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Caller {
    /// Their connection.
    pub ucid: ConnectionId,
    /// Their LFS username.
    pub uname: String,
    /// Whether they are an admin. Always true for the local connection.
    pub admin: bool,
}

impl Caller {
    fn new(ucid: ConnectionId, cx: &Context<'_>) -> Option<Self> {
        let conn = cx.world().connection(ucid);
        if conn.is_none() && !ucid.local() {
            return None;
        }
        Some(Self {
            ucid,
            uname: conn.map(|c| c.uname.clone()).unwrap_or_default(),
            admin: ucid.local() || conn.is_some_and(|c| c.admin),
        })
    }

    /// Reply to the sender in chat.
    pub fn reply<M: Send + 'static>(&self, text: impl Into<String>) -> Effects<M> {
        Effects::new().message(text, self.ucid)
    }
}

/// What a [`Router`] handler can return.
pub trait IntoEffects<M> {
    /// Convert into [`Effects`].
    fn into_effects(self) -> Effects<M>;
}

impl<M> IntoEffects<M> for Effects<M> {
    fn into_effects(self) -> Effects<M> {
        self
    }
}

impl<M> IntoEffects<M> for () {
    fn into_effects(self) -> Effects<M> {
        Effects::default()
    }
}

impl<M> IntoEffects<M> for Option<Effects<M>> {
    fn into_effects(self) -> Effects<M> {
        self.unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::insim::{Iii, Mso, Ncn};

    use super::*;
    use crate::{
        kitcar::input::Started,
        world::{World, event::Connected},
    };

    // Fields are only read through `Debug`, in the log.
    #[allow(dead_code)]
    #[derive(Debug, crate::chat::ChatCommand)]
    enum Cmd {
        Hello,
        Ban {
            uname: String,
            mins: Option<u32>,
        },
        #[chat(admin)]
        Restart,
    }

    #[derive(Debug, Default)]
    struct Bot {
        log: Vec<String>,
    }

    impl Bot {
        fn started(&mut self, _: &Started, _: &Context<'_>) {
            self.log.push("started".into());
        }
        fn joined(&mut self, joined: &Connected, _: &Context<'_>) -> Effects<u8> {
            self.log.push(format!("joined {}", joined.info.uname));
            Effects::new().after(std::time::Duration::from_secs(1), 7)
        }
        fn command(&mut self, caller: Caller, cmd: Cmd, _: &Context<'_>) -> Effects<u8> {
            self.log.push(format!("{} {cmd:?}", caller.uname));
            caller.reply("ok")
        }
    }

    fn world() -> World {
        let mut world = World::new();
        for (ucid, uname, admin) in [(2, "bob", false), (3, "root", true)] {
            let _ = world.apply(
                &Ncn {
                    ucid: ConnectionId(ucid),
                    uname: uname.into(),
                    admin,
                    ..Default::default()
                }
                .into(),
            );
        }
        world
    }

    fn router() -> Router<Bot, u8> {
        Router::with_messages(Bot::default())
            .on(Bot::started)
            .on(Bot::joined)
            .on_msg(|bot: &mut Bot, msg, _: &Context<'_>| bot.log.push(format!("msg {msg}")))
            .commands(Bot::command)
    }

    fn chat(ucid: u8, usertype: MsoUserType, msg: &str) -> Input<u8> {
        Input::Packet(
            Mso {
                ucid: ConnectionId(ucid),
                usertype,
                msg: msg.into(),
                ..Default::default()
            }
            .into(),
        )
    }

    fn replies(effects: &Effects<u8>) -> Vec<String> {
        effects
            .packets_sent()
            .filter_map(|p| match p {
                Packet::Mtc(mtc) => Some(mtc.text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn routes_markers_world_events_and_messages() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut router = router();

        assert!(router.update(Input::Started, &cx).is_empty());
        let joined = world.connection(ConnectionId(2)).unwrap().clone();
        let effects = router.update(
            Input::World(crate::world::WorldEvent::Connected(Connected {
                info: joined,
                existing: false,
            })),
            &cx,
        );
        assert_eq!(effects.timers().map(|(_, m)| *m).collect::<Vec<_>>(), [7]);
        let _ = router.update(Input::Msg(7), &cx);
        assert_eq!(router.state().log, ["started", "joined bob", "msg 7"]);
    }

    #[test]
    fn runs_commands_typed_with_the_prefix() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut router = router();

        let effects = router.update(chat(2, MsoUserType::Prefix, "!ban alice 5"), &cx);
        assert_eq!(replies(&effects), ["ok"]);
        let effects = router.update(
            Input::Packet(
                Iii {
                    ucid: ConnectionId(2),
                    msg: "hello".into(),
                    ..Default::default()
                }
                .into(),
            ),
            &cx,
        );
        assert_eq!(replies(&effects), ["ok"]);
        assert_eq!(
            router.state().log,
            [r#"bob Ban { uname: "alice", mins: Some(5) }"#, "bob Hello"]
        );
    }

    #[test]
    fn ignores_plain_chat_and_other_commands() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut router = router();

        for input in [
            chat(2, MsoUserType::User, "hello"),
            chat(2, MsoUserType::Prefix, "!nope"),
            chat(2, MsoUserType::Prefix, "!"),
            // Not a known connection.
            chat(9, MsoUserType::Prefix, "!hello"),
        ] {
            assert!(router.update(input, &cx).is_empty());
        }
        assert!(router.state().log.is_empty());
    }

    #[test]
    fn replies_with_usage_or_refusal_instead_of_running() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut router = router();

        let effects = router.update(chat(2, MsoUserType::Prefix, "!ban"), &cx);
        assert_eq!(
            replies(&effects),
            ["^1`ban` needs <uname>^8 - usage: !ban <uname> [mins]"]
        );
        let effects = router.update(chat(2, MsoUserType::Prefix, "!restart"), &cx);
        assert_eq!(replies(&effects), ["^1!restart is for admins only"]);
        assert!(router.state().log.is_empty());

        let effects = router.update(chat(3, MsoUserType::Prefix, "!restart"), &cx);
        assert_eq!(replies(&effects), ["ok"]);
        assert_eq!(router.state().log, ["root Restart"]);
    }
}
