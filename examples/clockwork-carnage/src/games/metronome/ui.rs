use kitcar::ui::{self, Component, Ui, View};

use super::state::MetronomeGlobal;
use crate::components::{hud_active, hud_muted, hud_text, hud_title};

#[derive(Clone, Default, Debug)]
pub(super) struct MetronomeConnectionProps {
    pub(super) uname: String,
    pub(super) in_run: bool,
    pub(super) best_delta_ms: Option<i64>,
}

pub(super) struct MetronomeView;

impl Component for MetronomeView {
    type Message = ();
    type Props<'a> = (&'a MetronomeGlobal, &'a MetronomeConnectionProps);

    fn render(&self, (global, player): Self::Props<'_>) -> ui::Node<Self::Message> {
        let status_str = if player.in_run {
            "In progress".to_string()
        } else {
            match player.best_delta_ms {
                Some(ms) => format!("Best: {:.3}s", ms as f64 / 1000.0),
                None => "Waiting for start".to_string(),
            }
        };
        let status_style = if player.in_run {
            hud_active()
        } else {
            hud_muted()
        };

        let target_str = format!("Target: {:.2}s", global.target.as_secs_f64());

        let lb_rows: Vec<ui::Node<()>> = global
            .leaderboard
            .iter()
            .take(8)
            .enumerate()
            .map(|(i, (uname, pname, delta_ms))| {
                let rank = format!("#{}", i + 1);
                let delta_str = format!("{:.3}s", *delta_ms as f64 / 1000.0);
                let style = if uname.as_str() == player.uname.as_str() {
                    hud_active()
                } else {
                    hud_text()
                };
                ui::row([
                    ui::text(rank, style).w(5.0).h(5.0),
                    ui::text(pname.as_str(), style.align_left()).w(20.0).h(5.0),
                    ui::text(delta_str, style.align_right()).w(12.0).h(5.0),
                ])
            })
            .collect();

        let scoreboard = ui::col([ui::text("Best Deltas", hud_title()).w(37.0).h(5.0)])
            .pl(5.0)
            .w(200.0)
            .mt(10.0)
            .items_start()
            .with_children(lb_rows);

        ui::col([
            ui::row([
                ui::text(&target_str, hud_title()).w(30.0).h(5.0),
                ui::text(status_str, status_style).w(20.0).h(5.0),
            ])
            .justify_center(),
            scoreboard,
        ])
    }
}

impl View for MetronomeView {
    type Global = MetronomeGlobal;
    type Connection = MetronomeConnectionProps;

    fn mount(
        _ucid: insim::identifiers::ConnectionId,
        _handle: ui::ViewHandle<Self::Message>,
    ) -> Self {
        MetronomeView
    }

    fn props<'a>(
        global: &'a MetronomeGlobal,
        connection: &'a MetronomeConnectionProps,
    ) -> Self::Props<'a> {
        (global, connection)
    }
}

pub(super) type MetronomeUi = Ui<MetronomeView>;
