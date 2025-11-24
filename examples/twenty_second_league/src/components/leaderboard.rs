use insim::core::string::colours::Colourify;
use kitcar::ui::{Element, Scope, component};

use crate::db::models::LeaderboardEntry;

#[component]
pub fn Leaderboard(entries: Vec<LeaderboardEntry>) -> Option<Element> {
    if entries.is_empty() {
        return None;
    }

    let rows: Vec<Element> = entries
        .iter()
        .map(|entry| {
            cx.container()
                .flex()
                .flex_row()
                .with_child(
                    cx.button(format!("{}.", entry.position).white())
                        .w(5.0)
                        .h(4.0),
                )
                .with_child(
                    cx.button(entry.pname.clone().white())
                        .w(20.0)
                        .h(4.0)
                        .text_align_start(),
                )
                .with_child(
                    cx.button(format!("{}", entry.total_points).white())
                        .w(5.0)
                        .h(4.0)
                        .text_align_end(),
                )
        })
        .collect();

    Some(
        cx.container()
            .flex()
            .flex_col()
            .p(1.0)
            .with_child(cx.button("Leaderboard".yellow()).w(30.0).h(5.0))
            .with_children(rows),
    )
}
