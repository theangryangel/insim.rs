use std::{
    collections::BTreeSet,
    sync::Arc,
    time::{Duration, SystemTime},
};

use insim::{core::vehicle::Vehicle, insim::BtnStyle};
use kitcar::ui;

use super::theme::{hud_active, hud_text};

/// (uname, pname, pts)
pub type EnrichedLeaderboard = Arc<[(String, String, u32)]>;

/// (uname, pname, vehicle, best_time, set_at)
pub type ChallengeLeaderboard = Arc<[(String, String, Vehicle, Duration, SystemTime)]>;

fn row_style(uname: &str, current_uname: &str) -> BtnStyle {
    if uname == current_uname {
        hud_active()
    } else {
        hud_text()
    }
}

pub fn scoreboard<Msg>(
    leaderboard: &EnrichedLeaderboard,
    current_uname: &str,
) -> Vec<ui::Node<Msg>> {
    let total = leaderboard.len();
    let player_pos = leaderboard
        .iter()
        .position(|(uname, _, _)| uname == current_uname);

    let indices_to_show: Vec<usize> = if total <= 7 {
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
    };

    indices_to_show
        .into_iter()
        .map(|index| {
            let (uname, pname, pts) = &leaderboard[index];
            let rank = format!("#{}", index + 1);
            let pts_str = format!("{}", pts);
            let style = row_style(uname, current_uname);

            ui::container().flex().flex_row().with_children([
                ui::text(rank, style).w(5.).h(5.),
                ui::text(pname.as_str(), style.align_left()).w(25.).h(5.),
                ui::text(pts_str, style.align_right()).w(5.).h(5.),
            ])
        })
        .collect()
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

pub fn challenge_scoreboard<Msg>(
    leaderboard: &ChallengeLeaderboard,
    current_uname: &str,
) -> Vec<ui::Node<Msg>> {
    let total = leaderboard.len();
    let player_pos = leaderboard
        .iter()
        .position(|(uname, _, _, _, _)| uname == current_uname);

    let indices_to_show = visible_indices(total, player_pos);

    indices_to_show
        .into_iter()
        .map(|index| {
            let (uname, pname, vehicle, time, _set_at) = &leaderboard[index];
            let rank = format!("#{}", index + 1);
            let vehicle_str = format!("{}", vehicle);
            let time_str = format!("{:.2?}", time);
            let style = row_style(uname, current_uname);

            ui::container().flex().flex_row().with_children([
                ui::text(rank, style).w(5.).h(5.),
                ui::text(pname.as_str(), style.align_left()).w(20.).h(5.),
                ui::text(vehicle_str, style).w(5.).h(5.),
                ui::text(time_str, style.align_right()).w(5.).h(5.),
            ])
        })
        .collect()
}
