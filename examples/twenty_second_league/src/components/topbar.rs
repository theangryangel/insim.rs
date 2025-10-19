use insim::core::string::colours::Colourify;
use kitcar::ui::{Component, Element, Scope, component};

#[component]
pub fn Topbar(text: String) -> Option<Element> {
    Some(
        cx.container()
            .flex()
            .flex_row()
            .justify_center()
            .with_child(
                cx.button(format!(
                    "{} {} {}",
                    "Welcome to the".white(),
                    "20".red(),
                    "second league".white()
                ))
                .w(38.)
                .h(5.)
                .dark(),
            )
            .with_child(cx.button(text.into()).w(33.).h(5.).dark()),
    )
}
