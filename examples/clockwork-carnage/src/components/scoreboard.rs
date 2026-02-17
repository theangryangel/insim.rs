use std::{collections::BTreeSet, sync::Arc};

use insim::insim::BtnStyle;
use kitcar::ui;

use super::theme::{hud_active, hud_text};

/// (uname, pname, pts)
pub type EnrichedLeaderboard = Arc<[(String, String, u32)]>;

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
