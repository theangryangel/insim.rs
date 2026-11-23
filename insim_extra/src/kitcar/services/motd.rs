//! [`Motd`]: a message of the day, shown to each player as they connect.
//!
//! A panel in the middle of the screen with a green button beneath it. It stays up
//! until the player clicks the button, so everyone reads it at least once a
//! visit. Players reopen it with [`MotdCommand::Motd`] (`!motd`, with your
//! InSim prefix). Start it next to your game:
//!
//! ```ignore
//! let _motd = host.start(
//!     Motd::new("ZINC Is Not CTRA")
//!         .line("A tribute to clean community racing.")
//!         .line("Type ^3!help^7 for commands.")
//!         .button("LET'S RACE"),
//! ).await?;
//! ```
//!
//! Everything sits on one panel, with the button joined to its foot:
//!
//! ```text
//!  ┌──────────────────────────────────────────┐
//!  │ ZINC Is Not CTRA                         │   title
//!  │ A tribute to clean community racing.     │   lines
//!  │ Type !help for commands.                 │
//!  ├──────────────────────────────────────────┤
//!  │               LET'S RACE                 │   the button, in green
//!  └──────────────────────────────────────────┘
//! ```
//!
//! The button is clickable, so LFS shows the mouse cursor until it's clicked.

use std::collections::HashSet;

use insim::{identifiers::ConnectionId, insim::BtnStyle};

use crate::{
    chat::ChatCommand,
    kitcar::{Context, Effects, Input, Service, services::router::command_text},
    ui::{self, Node, Ui},
    world::WorldEvent,
};

/// The [`Motd`]'s chat command. List [`MotdCommand::COMMANDS`] in your own
/// help to advertise it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ChatCommand)]
pub enum MotdCommand {
    /// Show the welcome message again.
    Motd,
}

/// The panel's width, in UI units of the 200 x 200 screen.
const WIDTH: u16 = 120;
/// Padding inside the panel.
const PAD_X: u16 = 2;
const PAD_Y: u16 = 2;
/// Height of a line, and of the header and button.
const ROW: u16 = 5;
const HEADER: u16 = 8;
/// Space between the title and the lines.
const GAP: u16 = 2;

/// The panel and button background. Dark keeps white text readable over a
/// bright track; `light()` with dark text is LFS's own look.
fn panel_style() -> BtnStyle {
    BtnStyle::default().dark()
}

/// A player closed the message. The [`Motd`]'s own message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseMotd(ConnectionId);

/// Shows a message of the day to each player as they connect, including those
/// already connected when it starts. It stays until they click its button.
#[derive(Debug, Clone)]
pub struct Motd {
    title: String,
    lines: Vec<String>,
    button: String,
    /// Who still has it open.
    open: HashSet<ConnectionId>,
}

impl Motd {
    /// A message headed `title`. LFS colour codes (`^3`) work in every piece
    /// of text.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
            button: "CLOSE".to_string(),
            open: HashSet::new(),
        }
    }

    /// A line of text under the header. One button per line, so keep each
    /// to about 50 characters.
    pub fn line(mut self, line: impl Into<String>) -> Self {
        self.lines.push(line.into());
        self
    }

    /// The button's label. `CLOSE` by default.
    pub fn button(mut self, label: impl Into<String>) -> Self {
        self.button = label.into();
        self
    }

    /// Whether `ucid` still has it open.
    pub fn is_open(&self, ucid: ConnectionId) -> bool {
        self.open.contains(&ucid)
    }

    /// The title and lines on one panel.
    fn panel(&self) -> Node<CloseMotd> {
        let inner = WIDTH - 2 * PAD_X;
        let plain = BtnStyle::default().align_left();
        let title = ui::text(self.title.as_str(), plain.yellow())
            .w(inner)
            .h(HEADER)
            .mb(GAP);
        let lines = self
            .lines
            .iter()
            .map(|line| ui::text(line.as_str(), plain.white()).w(inner).h(ROW));
        ui::background(panel_style())
            .w(WIDTH)
            .flex()
            .flex_col()
            .px(PAD_X)
            .py(PAD_Y)
            .with_child(title)
            .with_children(lines)
    }
}

impl Service for Motd {
    type Msg = CloseMotd;

    fn update(&mut self, input: Input<CloseMotd>, _: &Context<'_>) -> Effects<CloseMotd> {
        match input {
            Input::World(WorldEvent::Connected(connected)) if !connected.info.ucid.local() => {
                let _ = self.open.insert(connected.info.ucid);
            },
            Input::World(WorldEvent::Disconnected(left)) => {
                let _ = self.open.remove(&left.ucid);
            },
            Input::Msg(CloseMotd(ucid)) => {
                let _ = self.open.remove(&ucid);
            },
            input => {
                if let Some((ucid, _, text)) = command_text(&input)
                    && let Ok(MotdCommand::Motd) = text.parse()
                {
                    let _ = self.open.insert(ucid);
                }
            },
        }
        Effects::none()
    }

    fn view(
        &self,
        ucid: ConnectionId,
        _: &mut Ui<'_, CloseMotd>,
        _: &Context<'_>,
    ) -> Option<Node<CloseMotd>> {
        if !self.is_open(ucid) {
            return None;
        }
        // Joined to the panel's foot, so the two read as one.
        let button = ui::clickable(self.button.as_str(), panel_style().green(), CloseMotd(ucid))
            .w(WIDTH)
            .h(HEADER);
        // The panel and its button, centred on the screen. The full-screen
        // container is only for centring: it draws nothing.
        Some(
            ui::container()
                .position_fixed()
                .inset(0)
                .flex()
                .flex_col()
                .justify_center()
                .items_center()
                .with_child(self.panel())
                .with_child(button),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{
        Packet,
        insim::{Btn, BtnStyleFlags, Cnl, Ncn},
    };

    use super::*;
    use crate::{
        ui::{Screen, id_pool::ClickIdPool},
        world::World,
    };

    fn motd() -> Motd {
        Motd::new("ZINC")
            .line("^7Race clean.")
            .line("^7Type ^3!help^7 for commands.")
            .button("LET'S RACE")
    }

    /// Apply `packet` to `world` and feed `motd` its events.
    fn feed(world: &mut World, motd: &mut Motd, packet: Packet) {
        let events = world.apply(&packet);
        let cx = Context::new(world, Instant::now());
        for event in events {
            let _ = motd.update(Input::World(event), &cx);
        }
    }

    fn join(ucid: u8) -> Packet {
        Ncn {
            ucid: ConnectionId(ucid),
            uname: format!("driver{ucid}"),
            ..Default::default()
        }
        .into()
    }

    fn drawn(motd: &Motd, world: &World, ucid: u8) -> Vec<Btn> {
        let cx = Context::new(world, Instant::now());
        Screen::<CloseMotd>::new(ConnectionId(ucid))
            .render(&mut ClickIdPool::new(), cx.now(), |ui| {
                motd.view(ConnectionId(ucid), ui, &cx)
            })
            .into_iter()
            .filter_map(|p| match p {
                Packet::Btn(btn) => Some(btn),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn opens_on_every_connect_and_closes_on_click_or_leaving() {
        let mut world = World::new();
        let mut motd = motd();
        feed(&mut world, &mut motd, join(2));
        feed(&mut world, &mut motd, join(3));
        assert!(motd.is_open(ConnectionId(2)) && motd.is_open(ConnectionId(3)));

        let cx = Context::new(&world, Instant::now());
        let _ = motd.update(Input::Msg(CloseMotd(ConnectionId(2))), &cx);
        assert!(!motd.is_open(ConnectionId(2)));
        assert!(motd.is_open(ConnectionId(3)), "closing is per player");

        let left = Cnl {
            ucid: ConnectionId(3),
            ..Default::default()
        };
        feed(&mut world, &mut motd, left.into());
        assert!(!motd.is_open(ConnectionId(3)));

        // Coming back shows it again.
        feed(&mut world, &mut motd, join(3));
        assert!(motd.is_open(ConnectionId(3)));
    }

    #[test]
    fn draws_the_message_with_only_its_button_clickable() {
        let mut world = World::new();
        let mut motd = motd();
        feed(&mut world, &mut motd, join(2));
        let buttons = drawn(&motd, &world, 2);
        let texts: Vec<_> = buttons.iter().map(|b| b.text.as_str()).collect();
        for want in ["ZINC", "^7Race clean.", "LET'S RACE"] {
            assert!(texts.contains(&want), "{texts:?}");
        }
        let clickable: Vec<_> = buttons
            .iter()
            .filter(|b| b.bstyle.flags.contains(BtnStyleFlags::CLICK))
            .map(|b| b.text.as_str())
            .collect();
        assert_eq!(clickable, ["LET'S RACE"]);
    }

    #[test]
    fn the_motd_command_reopens_it() {
        use insim::insim::{Mso, MsoUserType};

        let mut world = World::new();
        let mut motd = motd();
        feed(&mut world, &mut motd, join(2));
        let cx = Context::new(&world, Instant::now());
        let _ = motd.update(Input::Msg(CloseMotd(ConnectionId(2))), &cx);
        assert!(!motd.is_open(ConnectionId(2)));

        let typed = |msg: &str| {
            Input::Packet(
                Mso {
                    ucid: ConnectionId(2),
                    usertype: MsoUserType::Prefix,
                    msg: msg.into(),
                    ..Default::default()
                }
                .into(),
            )
        };
        // Other commands, and chat, leave it shut.
        let _ = motd.update(typed("!rank"), &cx);
        assert!(!motd.is_open(ConnectionId(2)));

        // Whatever the prefix, and in any case.
        let _ = motd.update(typed("?MOTD"), &cx);
        assert!(motd.is_open(ConnectionId(2)));
    }
}
