use std::time::Instant;

use insim::Colour;
use kitcar::ui::{self, Component, Ui};

use super::state::BombGlobal;
use crate::components::{
    Dialog, DialogMsg, DialogProps, hud_active, hud_muted, hud_text, hud_title, topbar,
};

#[derive(Clone, Default, Debug)]
pub(super) struct BombConnectionProps {
    pub(super) uname: String,
    pub(super) in_run: bool,
}

const BOMB_HELP_LINES: &[&str] = &[
    " - Hit ^2checkpoint^7 objects before the timer expires or your run ends (BOOM).",
    " - Each checkpoint shrinks the window: ^1next window = current window - penalty^7.",
    " - Hit a ^3finish^7 object to fully reset the window back to the base time.",
    " - ^1Resetting your car^7 deducts the penalty directly from your remaining time.",
    " - ^1Pitting^7 ends your run immediately - commit to your fuel before you start.",
    " - ^1Collisions^7 cost time - harder impacts cost more, up to the collision max penalty.",
    " - Score = checkpoints hit. Survival time breaks ties.",
    " - Your best run is recorded on the leaderboard.",
    "",
    "Good luck.",
];

#[derive(Clone, Debug)]
pub(super) enum BombMsg {
    Help(DialogMsg),
}

pub(super) struct BombView {
    pub(super) _tick_handle: tokio::task::JoinHandle<()>,
    pub(super) help: Dialog,
}

impl Component for BombView {
    type Message = BombMsg;
    type Props<'a> = (&'a BombGlobal, &'a BombConnectionProps);

    fn update(&mut self, msg: Self::Message) {
        match msg {
            BombMsg::Help(help_msg) => {
                Component::update(&mut self.help, help_msg);
            },
        }
    }

    fn render(&self, (global, player): Self::Props<'_>) -> ui::Node<Self::Message> {
        if self.help.is_visible() {
            return self
                .help
                .render(DialogProps {
                    title: "Bomb",
                    lines: BOMB_HELP_LINES,
                })
                .map(BombMsg::Help);
        }

        let status_str = if player.in_run {
            "In run"
        } else {
            "Cross any checkpoint to start"
        };
        let status_style = if player.in_run {
            hud_active()
        } else {
            hud_muted()
        };

        let now = Instant::now();
        let active_run_rows: Vec<ui::Node<BombMsg>> = global
            .active_runs
            .iter()
            .map(|(uname, pname, cps, deadline, current_timeout)| {
                let secs_left = deadline.saturating_duration_since(now).as_secs_f64();
                let fraction = if current_timeout.is_zero() {
                    0.0
                } else {
                    (secs_left / current_timeout.as_secs_f64()).clamp(0.0, 1.0)
                };
                let cps_str = format!("{cps} cps");
                let time_str = format!("{secs_left:.1}s");
                const BAR_LEN: usize = 8;
                let available = (fraction * BAR_LEN as f64).round() as usize;
                let consumed = BAR_LEN - available;
                let consumed_part = "·".repeat(consumed).black();
                let available_part = if fraction > 0.25 {
                    "·".repeat(available).light_green()
                } else if fraction > 0.05 {
                    "·".repeat(available).yellow()
                } else {
                    "·".repeat(available).red()
                };
                let bar = format!("{available_part}{consumed_part}");
                let style = if uname.as_str() == player.uname.as_str() {
                    hud_active()
                } else {
                    hud_text()
                };
                ui::container().flex().flex_row().with_children([
                    ui::text(pname.as_str(), style.align_left()).w(15.0).h(5.0),
                    ui::text(cps_str, style.align_right()).w(8.0).h(5.0),
                    ui::text(time_str, style.align_right()).w(10.0).h(5.0),
                    ui::text(bar, style).w(10.0).h(5.0),
                ])
            })
            .collect();

        let leaderboard_rows: Vec<ui::Node<BombMsg>> = global
            .leaderboard
            .iter()
            .take(7)
            .enumerate()
            .map(|(i, (uname, pname, cps, ms))| {
                let style = if uname.as_str() == player.uname.as_str() {
                    hud_active()
                } else {
                    hud_text()
                };
                let rank = format!("#{}", i + 1);
                let cps_str = format!("{cps} cps");
                let survival_str = format!("{:.1}s", *ms as f64 / 1000.0);
                ui::container().flex().flex_row().with_children([
                    ui::text(rank, style).w(5.0).h(5.0),
                    ui::text(pname.as_str(), style.align_left()).w(20.0).h(5.0),
                    ui::text(cps_str, style.align_right()).w(8.0).h(5.0),
                    ui::text(survival_str, style.align_right()).w(10.0).h(5.0),
                ])
            })
            .collect();

        let scoreboard = ui::container()
            .flex()
            .pl(5.0)
            .w(200.0)
            .mt(10.0)
            .flex_col()
            .items_start()
            .with_child(ui::text("Active Runs", hud_title()).w(43.0).h(5.0))
            .with_children(active_run_rows)
            .with_child(ui::text("Session Best", hud_title()).w(43.0).h(5.0))
            .with_children(leaderboard_rows);

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                topbar(&format!("Bomb - {}", global.phase))
                    .with_child(ui::text(status_str, status_style).w(45.0).h(5.0)),
            )
            .with_child(scoreboard)
    }
}

pub(super) type BombUi = Ui<BombView, BombGlobal, BombConnectionProps>;
