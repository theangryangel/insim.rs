use insim::{core::string::colours::Colourify, insim::BtnStyle};
use kitcar::ui;

pub fn topbar<Msg>(state: &str) -> ui::Node<Msg> {
    ui::container()
        .with_child(ui::text(state, BtnStyle::default().dark()).w(38.).h(5.))
        .with_child(
            ui::text(
                format!("{} {}", "Welcome to Clockwork".white(), "Carnage".red()),
                BtnStyle::default().dark(),
            )
            .w(38.)
            .h(5.),
        )
        .flex()
        .flex_row()
        .justify_center()
        .w(200.)
}
