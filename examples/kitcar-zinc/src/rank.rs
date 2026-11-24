//! The `!rank` card: one driver's standing, as it was when they opened it.

use insim_extra::ui::{self, Component, Effects, Node, Ui};

use crate::zinc::Zinc;

/// Where a driver stands among everyone with a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    /// 1-based, or `None` before their first point or yellow.
    pub position: Option<usize>,
    pub of: usize,
    pub points: u32,
    pub yellows: u32,
    pub collisions: u32,
    pub cars: String,
}

impl Standing {
    pub fn of(zinc: &Zinc, uname: &str) -> Self {
        let driver = zinc.driver(uname);
        let ranked = zinc.ranked();
        Self {
            position: ranked.iter().position(|(u, _)| *u == uname).map(|i| i + 1),
            of: ranked.len(),
            points: driver.points,
            yellows: driver.yellows,
            collisions: driver.collisions,
            cars: driver.car_list(zinc.ladder()),
        }
    }
}

/// Closes the card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Close;

/// Snapshots the standing on mount, so the numbers don't shift while it's open.
/// Reopen it to refresh.
#[derive(Debug, Default)]
pub struct RankCard {
    standing: Option<Standing>,
}

impl Component for RankCard {
    type Msg = Close;
    type Out = ();
    type Props<'a> = (&'a Zinc, &'a str);

    fn mounted(&mut self, (zinc, uname): &(&Zinc, &str)) -> Effects<Close> {
        self.standing = Some(Standing::of(zinc, uname));
        Effects::none()
    }

    fn update(&mut self, Close: Close) -> Effects<Close> {
        Effects::new().emit(())
    }

    fn render(&self, _: (&Zinc, &str), _: &mut Ui<'_, Close>) -> Node<Close> {
        let Some(standing) = &self.standing else {
            return ui::empty();
        };
        let position = match standing.position {
            Some(position) => format!("^7#{position} ^8of {}", standing.of),
            None => "^8Unranked - finish a race cleanly".to_string(),
        };
        ui::components::dialog("Your rank")
            .width(48)
            .line(position)
            .line(format!(
                "^2{} point(s)^7, ^3{} yellow(s)^7, ^3{} collision(s)",
                standing.points, standing.yellows, standing.collisions
            ))
            .line(format!("^7Cars: ^3{}", standing.cars))
            .button(ui::clickable(
                "Close",
                ui::components::style::action(),
                Close,
            ))
            .build()
            .position_absolute()
            .top(60)
            .left(76)
    }
}
