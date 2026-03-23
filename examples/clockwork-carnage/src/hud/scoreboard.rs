use std::{collections::BTreeSet, sync::Arc, time::Duration};

use insim::{core::vehicle::Vehicle, insim::BtnStyle};
use kitcar::ui;

use super::{
    format_duration,
    theme::{hud_active, hud_text},
};

/// (uname, pname, best_delta)
pub type MetronomeLeaderboard = Arc<[(String, String, Duration)]>;

/// (uname, pname, vehicle, best_time)
pub type ChallengeLeaderboard = Arc<[(String, String, Vehicle, Duration)]>;

/// (uname, pname, checkpoint_count, survival_ms) — session best
pub type BombLeaderboard = Arc<[(String, String, i64, i64)]>;

fn row_style(uname: &str, current_uname: &str) -> BtnStyle {
    if uname == current_uname {
        hud_active()
    } else {
        hud_text()
    }
}

fn visible_indices(total: usize, player_pos: Option<usize>) -> Vec<usize> {
    if total <= 7 {
        (0..total).collect()
    } else {
        let mut positions: BTreeSet<usize> = [0, 1, 2, total - 1].into_iter().collect();
        if let Some(p) = player_pos {
            let _ = positions.insert(p);
            if p > 0 {
                let _ = positions.insert(p - 1);
            }
            if p < total - 1 {
                let _ = positions.insert(p + 1);
            }
        }
        while positions.len() < 7 {
            let next = (3..total).find(|i| !positions.contains(i));
            match next {
                Some(i) => {
                    let _ = positions.insert(i);
                },
                None => break,
            }
        }
        positions.into_iter().collect()
    }
}

pub fn bomb_scoreboard<Msg>(
    leaderboard: &BombLeaderboard,
    current_uname: &str,
) -> Vec<ui::Node<Msg>> {
    let total = leaderboard.len();
    let player_pos = leaderboard
        .iter()
        .position(|(uname, _, _, _)| uname == current_uname);

    let indices_to_show = visible_indices(total, player_pos);

    indices_to_show
        .into_iter()
        .map(|index| {
            let (uname, pname, cps, survival_ms) = &leaderboard[index];
            let rank = format!("#{}", index + 1);
            let cps_str = format!("{cps} cps");
            let survival_str = format_duration(Duration::from_millis(*survival_ms as u64));
            let style = row_style(uname, current_uname);

            ui::container().flex().flex_row().with_children([
                ui::text(rank, style).w(5.).h(5.),
                ui::text(pname.as_str(), style.align_left()).w(20.).h(5.),
                ui::text(cps_str, style.align_right()).w(8.).h(5.),
                ui::text(survival_str, style.align_right()).w(8.).h(5.),
            ])
        })
        .collect()
}

pub fn metronome_scoreboard<Msg>(
    leaderboard: &MetronomeLeaderboard,
    current_uname: &str,
) -> Vec<ui::Node<Msg>> {
    let total = leaderboard.len();
    let player_pos = leaderboard
        .iter()
        .position(|(uname, _, _)| uname == current_uname);

    let indices_to_show = visible_indices(total, player_pos);

    indices_to_show
        .into_iter()
        .map(|index| {
            let (uname, pname, delta) = &leaderboard[index];
            let rank = format!("#{}", index + 1);
            let delta_str = format_duration(*delta);
            let style = row_style(uname, current_uname);

            ui::container().flex().flex_row().with_children([
                ui::text(rank, style).w(5.).h(5.),
                ui::text(pname.as_str(), style.align_left()).w(20.).h(5.),
                ui::text(delta_str, style.align_right()).w(10.).h(5.),
            ])
        })
        .collect()
}

pub fn challenge_scoreboard<Msg>(
    leaderboard: &ChallengeLeaderboard,
    current_uname: &str,
) -> Vec<ui::Node<Msg>> {
    let total = leaderboard.len();
    let player_pos = leaderboard
        .iter()
        .position(|(uname, _, _, _)| uname == current_uname);

    let indices_to_show = visible_indices(total, player_pos);

    indices_to_show
        .into_iter()
        .map(|index| {
            let (uname, pname, vehicle, time) = &leaderboard[index];
            let rank = format!("#{}", index + 1);
            let vehicle_str = format!("{}", vehicle);
            let time_str = format_duration(*time);
            let style = row_style(uname, current_uname);

            ui::container().flex().flex_row().with_children([
                ui::text(rank, style).w(5.).h(5.),
                ui::text(pname.as_str(), style.align_left()).w(20.).h(5.),
                ui::text(vehicle_str, style).w(5.).h(5.),
                ui::text(time_str, style.align_right()).w(10.).h(5.),
            ])
        })
        .collect()
}
