use kitcar::ui;

use super::theme::hud_title;

pub fn topbar<Msg>(state: &str) -> ui::Node<Msg> {
    ui::container()
        .flex()
        .flex_row()
        .justify_center()
        .with_child(ui::text(state, hud_title()).w(30.).h(5.))
}
