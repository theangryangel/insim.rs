use kitcar::ui;

use super::{
    hud_overlay_action, hud_overlay_muted, hud_overlay_text, hud_overlay_title, hud_panel_bg,
    hud_panel_border,
};

const PANEL_WIDTH: f32 = 132.;
const PANEL_HEIGHT: f32 = 68.;
const PANEL_BORDER: f32 = 2.;
const INNER_WIDTH: f32 = PANEL_WIDTH - (PANEL_BORDER * 2.);
const INNER_HEIGHT: f32 = PANEL_HEIGHT - (PANEL_BORDER * 2.);
const PADDING_X: f32 = 6.;
const PADDING_TOP: f32 = 4.;
const CONTENT_WIDTH: f32 = INNER_WIDTH - (PADDING_X * 2.);

const TITLE_HEIGHT: f32 = 5.;
const DESCRIPTION_HEIGHT: f32 = 4.;
const ROW_HEIGHT: f32 = 4.;
const FOOTER_HEIGHT: f32 = 6.;
const FOOTER_ACTION_WIDTH: f32 = 24.;
const FOOTER_HINT_WIDTH: f32 = CONTENT_WIDTH - FOOTER_ACTION_WIDTH;

const GAP_SMALL: f32 = 1.;
const GAP_MEDIUM: f32 = 2.;

fn spacer<Msg>(height: f32) -> ui::Node<Msg> {
    ui::text("", hud_overlay_text()).w(CONTENT_WIDTH).h(height)
}

fn section_row<Msg>(text: &'static str) -> ui::Node<Msg> {
    ui::text(text, hud_overlay_action().align_left())
        .w(CONTENT_WIDTH)
        .h(ROW_HEIGHT)
}

fn body_row<Msg>(text: &'static str) -> ui::Node<Msg> {
    ui::text(text, hud_overlay_text().align_left())
        .w(CONTENT_WIDTH)
        .h(ROW_HEIGHT)
}

fn header_row<Msg>() -> ui::Node<Msg> {
    ui::container()
        .flex()
        .flex_col()
        .w(CONTENT_WIDTH)
        .with_child(
            ui::text("Clockwork Carnage", hud_overlay_title().align_left())
                .w(CONTENT_WIDTH)
                .h(TITLE_HEIGHT),
        )
        .with_child(
            ui::text(
                "Precision racing mode quick guide",
                hud_overlay_muted().align_left(),
            )
            .w(CONTENT_WIDTH)
            .h(DESCRIPTION_HEIGHT),
        )
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
        .with_child(
            ui::clickable(
                "Close",
                hud_overlay_action().align_right(),
                HelpDialogMsg::Hide,
            )
            .w(FOOTER_ACTION_WIDTH)
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
            .h(100.)
            .with_child(
                ui::text("", hud_panel_border())
                    .w(PANEL_WIDTH)
                    .h(PANEL_HEIGHT),
            )
            .with_child(
                ui::text("", hud_panel_bg())
                    .w(INNER_WIDTH)
                    .h(INNER_HEIGHT)
                    .mt(-(PANEL_HEIGHT - PANEL_BORDER)),
            )
            .with_child(
                ui::container()
                    .flex()
                    .flex_col()
                    .w(INNER_WIDTH)
                    .h(INNER_HEIGHT)
                    .mt(-INNER_HEIGHT)
                    .pt(PADDING_TOP)
                    .with_child(
                        ui::container()
                            .flex()
                            .flex_col()
                            .ml(PADDING_X)
                            .w(CONTENT_WIDTH)
                            .with_children([
                                header_row(),
                                spacer(GAP_MEDIUM),
                                section_row("OBJECTIVE"),
                                body_row("- Match the target lap time as closely as possible."),
                                spacer(GAP_SMALL),
                                section_row("ROUND FLOW"),
                                body_row("- CP1 starts your timed attempt."),
                                body_row("- FINISH locks the run and sends you to spec."),
                                spacer(GAP_SMALL),
                                section_row("SCORING"),
                                body_row("- Lower delta ranks higher and earns points."),
                                spacer(GAP_MEDIUM),
                                footer_row(),
                            ]),
                    ),
            )
    }
}
