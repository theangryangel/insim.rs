use kitcar::ui;

use super::theme::{
    hud_overlay_action, hud_overlay_text, hud_panel_bg,
};

#[derive(Default)]
pub struct HelpDialog {
    visible: bool,
}

impl HelpDialog {
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

#[derive(Clone, Debug)]
pub enum HelpDialogMsg {
    Show,
    Hide,
}

impl ui::Component for HelpDialog {
    type Message = HelpDialogMsg;
    type Props = ();

    fn update(&mut self, msg: Self::Message) {
        match msg {
            HelpDialogMsg::Show => self.visible = true,
            HelpDialogMsg::Hide => self.visible = false,
        }
    }

    fn render(&self, _props: Self::Props) -> ui::Node<Self::Message> {
        if !self.visible {
            return ui::Node::empty();
        }

        ui::container()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .w(200.)
            .h(200.)
            .with_child(
                ui::container()
                .flex()
                .flex_col()
                .with_child(
                    ui::background(hud_panel_bg())
                    .w(100.0)
                    .flex()
                    .flex_col()
                    .p(1.)
                    .with_child(
                        ui::text("Welcome to Clockwork Carnage", hud_overlay_text().align_left().yellow()).h(8.).mb(2.).w_auto(),
                    )
                    .with_children([
                        " - Match the target lap time as closely as possible.",
                        " - Crossing the first checkpoint starts your timed attempt.",
                        " - Find one of the finishes as close to the target time as possible.",
                        " - Full contact is permitted.",
                        " - Don't be a dick.",
                        " - Lower delta ranks higher and earns more points.",
                        " - Retry as many times as you want each round.",
                        "",
                        "Good luck.",
                    ].iter().map(|t| {
                        ui::text(t.to_owned(), hud_overlay_text().align_left().white())
                            .w_auto()
                            .h(6.)
                    }))
                )
                .with_child(
                    ui::clickable(
                        "Close",
                        hud_overlay_action().green().dark(),
                        HelpDialogMsg::Hide,
                    )
                    .self_end()
                    .w(12.)
                    .h(8.)
                    .mt(2.)
                )

            )
    }
}
