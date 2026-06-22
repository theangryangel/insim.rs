use kitcar::ui::{self, Component, Ui, View};

use super::state::ShortcutGlobal;
use crate::components::{hud_active, hud_muted, hud_text, hud_title};

#[derive(Clone, Default, Debug)]
pub(super) struct ShortcutConnectionProps {
    pub(super) uname: String,
    pub(super) in_run: bool,
    pub(super) best_time_ms: Option<i64>,
}

pub(super) struct ShortcutView;

impl Component for ShortcutView {
    type Message = ();
    type Props<'a> = (&'a ShortcutGlobal, &'a ShortcutConnectionProps);

    fn render(&self, (global, player): Self::Props<'_>) -> ui::Node<Self::Message> {
        let status_str = if player.in_run {
            "In progress".to_string()
        } else {
            match player.best_time_ms {
                Some(ms) => {
                    let mins = ms / 60_000;
                    let secs = (ms % 60_000) / 1000;
                    let millis = ms % 1000;
                    format!("PB: {mins}:{secs:02}.{millis:03}")
                },
                None => "Waiting for start".to_string(),
            }
        };
        let status_style = if player.in_run {
            hud_active()
        } else {
            hud_muted()
        };

        let lb_rows: Vec<ui::Node<()>> = global
            .leaderboard
            .iter()
            .take(8)
            .enumerate()
            .map(|(i, (uname, pname, time_ms))| {
                let rank = format!("#{}", i + 1);
                let mins = time_ms / 60_000;
                let secs = (time_ms % 60_000) / 1000;
                let millis = time_ms % 1000;
                let time_str = format!("{mins}:{secs:02}.{millis:03}");
                let style = if uname.as_str() == player.uname.as_str() {
                    hud_active()
                } else {
                    hud_text()
                };
                ui::container().flex().flex_row().with_children([
                    ui::text(rank, style).w(5.0).h(5.0),
                    ui::text(pname.as_str(), style.align_left()).w(20.0).h(5.0),
                    ui::text(time_str, style.align_right()).w(15.0).h(5.0),
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
            .with_child(ui::text("Best Times", hud_title()).w(40.0).h(5.0))
            .with_children(lb_rows);

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                ui::container()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .with_child(
                        ui::text(&format!("Shortcut - {}", global.phase), hud_title())
                            .w(40.0)
                            .h(5.0),
                    )
                    .with_child(ui::text(status_str, status_style).w(20.0).h(5.0)),
            )
            .with_child(scoreboard)
    }
}

impl View for ShortcutView {
    type Global = ShortcutGlobal;
    type Connection = ShortcutConnectionProps;

    fn mount(
        _ucid: insim::identifiers::ConnectionId,
        _handle: ui::ViewHandle<Self::Message>,
    ) -> Self {
        ShortcutView
    }

    fn props<'a>(
        global: &'a ShortcutGlobal,
        connection: &'a ShortcutConnectionProps,
    ) -> Self::Props<'a> {
        (global, connection)
    }
}

pub(super) type ShortcutUi = Ui<ShortcutView>;
