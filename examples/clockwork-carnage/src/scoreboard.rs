use std::collections::BTreeSet;

use insim::{core::string::colours::Colourify, insim::BtnStyle};
use kitcar::ui;

/// (uname, pname, pts)
pub type EnrichedLeaderboard = Vec<(String, String, u32)>;

pub fn scoreboard<Msg>(leaderboard: &EnrichedLeaderboard, current_uname: &str) -> Vec<ui::Node<Msg>> {
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
                Some(i) => { let _ = positions.insert(i); }
                None => break,
            }
        }
        positions.into_iter().collect()
    };

    indices_to_show
        .into_iter()
        .map(|index| {
            let (_, pname, pts) = &leaderboard[index];
            let rank = format!("#{}", index + 1);
            let pts_str = format!("{}", pts);
            let (rank, name, pts_str): (String, String, String) = match index {
                0 => (rank.yellow(), pname.yellow(), pts_str.yellow()),
                1 => (rank.white(), pname.white(), pts_str.white()),
                2 => (rank.red(), pname.red(), pts_str.red()),
                _ => (rank, pname.clone(), pts_str),
            };
            let style = BtnStyle::default().dark();
            ui::container().flex().flex_row().with_children([
                ui::text(rank, style).w(5.).h(5.),
                ui::text(name, style.align_left()).w(25.).h(5.),
                ui::text(pts_str, style).w(5.).h(5.),
            ])
        })
        .collect()
}
