use insim::core::string::colours::Colourify;
use kitcar::ui::{wrap_text, Element, Styled};

pub(crate) fn topbar(text: &str) -> Element {
    // top bar
    Element::container()
        .flex()
        .flex_row()
        .justify_center()
        .with_child(
            Element::button(
                "welcome",
                &format!(
                    "{} {} {}",
                    "Welcome to the".white(),
                    "20".red(),
                    "second league".white()
                ),
            )
            .w(38.)
            .h(5.)
            .dark(),
        )
        .with_child(
            Element::button(
                "countdown",
                // ,
                &text,
            )
            .w(33.)
            .h(5.)
            .dark(),
        )
}

pub(crate) fn motd() -> Element {
    let text: Vec<Element> = wrap_text(
        "Welcome drivers!
Forget being the fastest, the goal is to be the most precise. Finish in as close to 20secs as possible!
Full contact is allowed.
Just remember: Don't be a dick. We're all here to have fun!",
        5,
        78
    ).enumerate().map(|(i, line)| {
        Element::button(&format!("motd_text_{}", i), line).h(5.).text_align_start()
    }).collect();

    Element::container().flex().flex_grow(1.0).with_child(
        Element::button("motd", "")
            .flex()
            .flex_col()
            .w(80.)
            .p(1.)
            .light()
            .my_auto()
            .mx_auto()
            .with_child(
                Element::button("motd_inner", "")
                    .flex()
                    .flex_col()
                    .dark()
                    .p(1.)
                    .with_children(text),
            )
            .with_child(
                Element::button("motd_close", &"Got it!".light_green())
                    .mt(2.)
                    .h(5.)
                    .green()
                    .dark()
                    .clickable(),
            ),
    )
}
