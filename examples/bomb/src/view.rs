//! What each player sees, drawn straight from the game's state.

use insim::{Colour, identifiers::ConnectionId, insim::BtnStyle};
use insim_extra::{
    kitcar::Context,
    ui::{self, Child, Component, Node, Ui},
};

use crate::game::{Bomb, Msg};

/// Dialog keys, so chat commands can open them.
pub const HELP: &str = "help";
pub const ABOUT: &str = "about";

const HELP_LINES: &[&str] = &[
    " - Hit ^2checkpoint^7 objects before the timer expires or your run ends (BOOM).",
    " - Each checkpoint shrinks the window: ^1next window = current window - penalty^7.",
    " - Hit a ^3finish^7 object to fully reset the window back to the base time.",
    " - ^1Resetting your car^7 deducts the penalty directly from your remaining time.",
    " - ^1Pitting^7 ends your run immediately - commit to your fuel before you start.",
    " - ^1Collisions^7 cost time - harder impacts cost more, up to the collision max penalty.",
    " - Score = checkpoints hit. Survival time breaks ties.",
    "",
    "Good luck.",
];

const ABOUT_LINES: &[&str] = &[
    "Bomb is an in-memory fork of the ^2anomalous^7 mini-game,",
    "built with ^2kitcar^7: one service, a view drawn from its",
    "state, and self-contained components like this dialog.",
    "",
    "Type ^2!help^7 for the rules, or click ^3?^7 at the top.",
];

fn title() -> BtnStyle {
    BtnStyle::default().dark().yellow()
}
fn text() -> BtnStyle {
    BtnStyle::default().dark().light_grey()
}
fn active() -> BtnStyle {
    BtnStyle::default().dark().white()
}
fn muted() -> BtnStyle {
    BtnStyle::default().dark().grey()
}

/// A dialog that opens on `Show` and closes itself. The host keeps one per
/// player, so the game never tracks whether it's open.
#[derive(Default)]
pub struct Dialog {
    open: bool,
}

#[derive(Clone, Debug)]
pub enum DialogMsg {
    Show,
    Hide,
}

pub struct DialogProps<'a> {
    pub title: &'a str,
    pub lines: &'a [&'a str],
}

impl Component for Dialog {
    type Msg = DialogMsg;
    type Out = ();
    type Props<'a> = DialogProps<'a>;

    fn update(&mut self, msg: DialogMsg) -> Option<()> {
        self.open = matches!(msg, DialogMsg::Show);
        None
    }

    fn render(&self, props: DialogProps<'_>, _: &mut Ui<'_, DialogMsg>) -> Node<DialogMsg> {
        if !self.open {
            return ui::empty();
        }
        ui::col([
            ui::background(BtnStyle::default().dark().light_grey())
                .w(100)
                .flex()
                .flex_col()
                .p(1)
                .with_child(
                    ui::text(props.title, BtnStyle::default().align_left().yellow())
                        .h(8)
                        .mb(2)
                        .w_auto(),
                )
                .with_children(props.lines.iter().map(|line| {
                    ui::text(*line, BtnStyle::default().align_left().white())
                        .w_auto()
                        .h(6)
                })),
            ui::clickable("Close", BtnStyle::default().green().dark(), DialogMsg::Hide)
                .self_end()
                .w(12)
                .h(8)
                .mt(2),
        ])
    }
}

/// `text` scrolling through a window `width` characters wide, by elapsed time.
fn marquee(text: &str, width: usize, elapsed: std::time::Duration) -> String {
    let padded: Vec<char> = format!("{text}{}", " ".repeat(width)).chars().collect();
    let offset = (elapsed.as_millis() / 200) as usize % padded.len();
    padded.iter().cycle().skip(offset).take(width).collect()
}

/// A countdown bar: green, then yellow, then red as the window runs out.
fn bar(fraction: f64) -> String {
    const LEN: usize = 8;
    let left = (fraction * LEN as f64).round() as usize;
    let used = "·".repeat(LEN - left).black();
    let remaining = "·".repeat(left);
    let remaining = if fraction > 0.25 {
        remaining.light_green()
    } else if fraction > 0.05 {
        remaining.yellow()
    } else {
        remaining.red()
    };
    format!("{remaining}{used}")
}

pub fn view(bomb: &Bomb, ucid: ConnectionId, ui: &mut Ui<'_, Msg>, cx: &Context<'_>) -> Node<Msg> {
    let now = ui.now();
    let me = cx
        .world()
        .connection(ucid)
        .map(|c| c.uname.as_str())
        .unwrap_or_default();
    let in_run = bomb.rules.runs.values().any(|run| run.uname == me);
    let style_for = |uname: &str| if uname == me { active() } else { text() };

    let mut runs: Vec<_> = bomb.rules.runs.values().collect();
    runs.sort_by_key(|run| std::cmp::Reverse(run.checkpoints));
    let run_rows = runs.into_iter().map(|run| {
        let left = run.time_left(now);
        let fraction = if run.window.is_zero() {
            0.0
        } else {
            (left.as_secs_f64() / run.window.as_secs_f64()).clamp(0.0, 1.0)
        };
        let style = style_for(&run.uname);
        ui::row([
            ui::text(run.pname.as_str(), style.align_left()).w(15).h(5),
            ui::text(format!("{} cps", run.checkpoints), style.align_right())
                .w(8)
                .h(5),
            ui::text(format!("{:.1}s", left.as_secs_f64()), style.align_right())
                .w(10)
                .h(5),
            ui::text(bar(fraction), style).w(10).h(5),
        ])
        .key(format!("run-{}", run.uname))
    });

    let board_rows = bomb.board.iter().take(7).enumerate().map(|(i, standing)| {
        let style = style_for(&standing.uname);
        ui::row([
            ui::text(format!("#{}", i + 1), style).w(5).h(5),
            ui::text(standing.pname.as_str(), style.align_left())
                .w(20)
                .h(5),
            ui::text(format!("{} cps", standing.checkpoints), style.align_right())
                .w(8)
                .h(5),
            ui::text(
                format!("{:.1}s", standing.survival.as_secs_f64()),
                style.align_right(),
            )
            .w(10)
            .h(5),
        ])
    });

    let (status, status_style) = if in_run {
        ("In run", active())
    } else {
        ("Cross any checkpoint to start", muted())
    };

    let topbar = ui::row([
        ui::text(
            if cx.world().session().is_some() {
                "Bomb - racing"
            } else {
                "Bomb - waiting for a race"
            },
            title(),
        )
        .w(30)
        .h(5),
        ui::text(status, status_style).w(45).h(5),
        ui::text(
            marquee("in-memory build - no database", 20, now - bomb.started),
            muted(),
        )
        .w(25)
        .h(5),
        ui::clickable("?", title(), Msg::OpenHelp(ucid)).w(5).h(5),
    ])
    .justify_center();

    let scoreboard = ui::container()
        .flex()
        .flex_col()
        .pl(5)
        .mt(10)
        .items_start()
        .with_child(ui::text("Active Runs", title()).w(43).h(5))
        .with_children(run_rows)
        .with_child(ui::text("Session Best", title()).w(43).h(5))
        .with_children(board_rows);

    // Dialogs float centred over everything. Each is its own component, so it
    // opens and closes per player.
    let dialogs = ui::row([
        ui.child(
            Child::with_props(
                DialogProps {
                    title: "Bomb - Help",
                    lines: HELP_LINES,
                },
                Dialog::default,
            )
            .key(HELP),
        ),
        ui.child(
            Child::with_props(
                DialogProps {
                    title: "Bomb - About",
                    lines: ABOUT_LINES,
                },
                Dialog::default,
            )
            .key(ABOUT),
        ),
    ])
    .position_absolute()
    .w(200)
    .h(200)
    .justify_center()
    .items_center();

    ui::col([topbar, scoreboard, dialogs]).w(200)
}
