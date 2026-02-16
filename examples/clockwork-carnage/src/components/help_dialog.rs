use kitcar::ui;

use super::{
    hud_overlay_action, hud_overlay_muted, hud_overlay_text, hud_overlay_title, hud_panel_bg,
    hud_panel_border,
};

const INNER_WIDTH: f32 = 132.;
const INNER_HEIGHT: f32 = 68.;
const PADDING_X: f32 = 2.;
const PADDING_TOP: f32 = 2.;
const CONTENT_WIDTH: f32 = INNER_WIDTH - (PADDING_X * 2.);

const ROW_HEIGHT: f32 = 5.;
const FOOTER_HEIGHT: f32 = 6.;
const FOOTER_ACTION_WIDTH: f32 = 24.;
const FOOTER_HINT_WIDTH: f32 = CONTENT_WIDTH - FOOTER_ACTION_WIDTH;

fn section_row<Msg>(text: &'static str) -> ui::Node<Msg> {
    ui::text(text, hud_overlay_action().align_left().white())
        .w(CONTENT_WIDTH)
        .h(ROW_HEIGHT)
}

fn body_row<Msg>(text: &'static str) -> ui::Node<Msg> {
    ui::text(text, hud_overlay_text().align_left().white())
        .w(CONTENT_WIDTH)
        .h(ROW_HEIGHT)
}

fn footer_row() -> ui::Node<HelpDialogMsg> {
    ui::container()
        .flex()
        .flex_row()
        .w(CONTENT_WIDTH)
        .with_child(
            ui::text(
                "!help to reopen  |  !start / !end admin",
                hud_overlay_muted().align_left(),
            )
            .w(FOOTER_HINT_WIDTH)
            .h(FOOTER_HEIGHT),
        )
}

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
                ui::text("", hud_panel_bg())
                    .w(INNER_WIDTH)
                    .h(INNER_HEIGHT)
                    .mt(-(INNER_HEIGHT)),
            )
            .with_child(
                ui::container()
                    .flex()
                    .flex_col()
                    .w(INNER_WIDTH)
                    .h(INNER_HEIGHT)
                    .mt(-INNER_HEIGHT)
                    .pt(PADDING_TOP)
                    .pl(1.)
                    .pr(1.)
                    .with_children([
                        ui::text("Welcome to Clockwork Carnage", hud_overlay_text().align_left().white()).h(6.).mb(2.).w_auto(),
                        section_row("OBJECTIVE"),
                        body_row("- Match the target lap time as closely as possible."),
                        section_row("ROUND FLOW"),
                        body_row("- CP1 starts your timed attempt."),
                        body_row("- FINISH locks the run and sends you to spec."),
                        section_row("SCORING"),
                        body_row("- Lower delta ranks higher and earns points."),
                        footer_row(),
                    ])
            )
            .with_child(
                ui::clickable(
                    "Close",
                    hud_overlay_action().align_right().green().dark(),
                    HelpDialogMsg::Hide,
                )
                .mt(2.)
                .self_end()
                .w(8.)
                .h(6.),
        )
    }
}
