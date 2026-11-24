//! The `!top` leaderboard: ZINC's standings, a page at a time.
//!
//! The page number is the component's own, per player; the standings come
//! from ZINC on every render, so they stay current while it's open.

use insim_extra::ui::{self, Component, Effects, Node, Ui, components::style};

use crate::zinc::Zinc;

/// Drivers per page.
const PER_PAGE: usize = 8;
/// The panel's width, in UI units of the 200 x 200 screen.
const WIDTH: u16 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopMsg {
    Prev,
    Next,
    Close,
}

/// The leaderboard. Reports `()` when closed.
#[derive(Debug, Default)]
pub struct Leaderboard {
    /// 0-based. Clamped to the pages there are when drawn.
    page: usize,
}

/// How many pages `drivers` fill: at least one, even when empty.
fn pages(drivers: usize) -> usize {
    drivers.div_ceil(PER_PAGE).max(1)
}

impl Component for Leaderboard {
    type Msg = TopMsg;
    type Out = ();
    type Props<'a> = &'a Zinc;

    fn update(&mut self, msg: TopMsg) -> Effects<TopMsg> {
        match msg {
            TopMsg::Prev => self.page = self.page.saturating_sub(1),
            // Only clickable when there's a next page.
            TopMsg::Next => self.page += 1,
            TopMsg::Close => return Effects::new().emit(()),
        }
        Effects::none()
    }

    fn render(&self, zinc: &Zinc, _: &mut Ui<'_, TopMsg>) -> Node<TopMsg> {
        let ranked = zinc.ranked();
        let pages = pages(ranked.len());
        let page = self.page.min(pages - 1);
        let rows: Vec<String> = if ranked.is_empty() {
            vec!["^7No points yet - finish a race cleanly!".to_string()]
        } else {
            ranked
                .iter()
                .enumerate()
                .skip(page * PER_PAGE)
                .take(PER_PAGE)
                .map(|(i, (uname, driver))| format!("^7{}. {uname} ^8- ^2{}", i + 1, driver.points))
                .collect()
        };
        // An arrow that can't go anywhere is drawn, but not clickable.
        let arrow = |label: &str, enabled: bool, msg: TopMsg| {
            if enabled {
                ui::clickable(label, style::panel(), msg)
            } else {
                ui::text(label, style::muted())
            }
            .w(10)
        };
        ui::components::dialog("Top drivers")
            .width(WIDTH)
            .aside(format!("page {} of {pages}", page + 1))
            .lines(rows)
            .button(arrow("<", page > 0, TopMsg::Prev))
            .button(ui::clickable("Close", style::action(), TopMsg::Close))
            .button(arrow(">", page + 1 < pages, TopMsg::Next))
            .build()
            .position_absolute()
            .top(50)
            .left((200 - WIDTH) / 2)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{Packet, identifiers::ConnectionId};
    use insim_extra::ui::{Child, Screen, id_pool::ClickIdPool};

    use super::*;

    /// The button texts of `board` drawn over `zinc`.
    fn shown(zinc: &Zinc, page: usize) -> Vec<String> {
        Screen::<()>::new(ConnectionId(2))
            .render(&mut ClickIdPool::new(), Instant::now(), |ui| {
                Some(ui.child(Child::with_props(zinc, || Leaderboard { page })))
            })
            .into_iter()
            .filter_map(|p| match p {
                Packet::Btn(btn) => Some(btn.text),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn pages_through_the_standings() {
        let mut zinc = Zinc::new(crate::LICENCES);
        for n in 0..20 {
            zinc.set(&format!("d{n:02}"), n);
        }
        let second = shown(&zinc, 1);
        assert!(second.contains(&"page 2 of 3".to_string()), "{second:?}");
        assert!(
            second.iter().any(|t| t.starts_with("^79. d11")),
            "{second:?}"
        );
        // Past the end shows the last page.
        assert!(shown(&zinc, 9).contains(&"page 3 of 3".to_string()));

        let mut board = Leaderboard::default();
        let _ = board.update(TopMsg::Next);
        assert_eq!(board.page, 1);
        let _ = board.update(TopMsg::Prev);
        let _ = board.update(TopMsg::Prev);
        assert_eq!(board.page, 0);
        assert_eq!(board.update(TopMsg::Close).emitted().count(), 1);
    }
}
