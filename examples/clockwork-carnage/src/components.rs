use insim::insim::BtnStyle;
use kitcar::ui;

pub fn hud_title() -> BtnStyle {
    BtnStyle::default().dark().yellow()
}

pub fn hud_text() -> BtnStyle {
    BtnStyle::default().dark().light_grey()
}

pub fn hud_active() -> BtnStyle {
    BtnStyle::default().dark().white()
}

pub fn hud_muted() -> BtnStyle {
    BtnStyle::default().dark().grey()
}

pub fn hud_panel_bg() -> BtnStyle {
    BtnStyle::default().dark().light_grey()
}

pub fn hud_overlay_text() -> BtnStyle {
    BtnStyle::default().light_grey()
}

pub fn hud_overlay_action() -> BtnStyle {
    BtnStyle::default().pale_blue()
}

pub fn topbar<Msg>(title: &str) -> ui::Node<Msg> {
    ui::container()
        .flex()
        .flex_row()
        .justify_center()
        .with_child(ui::text(title, hud_title()).w(30.0).h(5.0))
}

#[derive(Default)]
pub struct Dialog {
    pub visible: bool,
}

impl Dialog {
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

#[derive(Clone, Debug)]
pub enum DialogMsg {
    Show,
    Hide,
}

pub struct DialogProps<'a> {
    pub title: &'a str,
    pub lines: &'a [&'a str],
}

impl ui::Component for Dialog {
    type Message = DialogMsg;
    type Props<'a> = DialogProps<'a>;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            DialogMsg::Show => self.visible = true,
            DialogMsg::Hide => self.visible = false,
        }
    }

    fn render(&self, props: Self::Props<'_>) -> ui::Node<Self::Message> {
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
                                ui::text(props.title, hud_overlay_text().align_left().yellow())
                                    .h(8.)
                                    .mb(2.)
                                    .w_auto(),
                            )
                            .with_children(props.lines.iter().map(|t| {
                                ui::text(t.to_owned(), hud_overlay_text().align_left().white())
                                    .w_auto()
                                    .h(6.)
                            })),
                    )
                    .with_child(
                        ui::clickable(
                            "Close",
                            hud_overlay_action().green().dark(),
                            DialogMsg::Hide,
                        )
                        .self_end()
                        .w(12.)
                        .h(8.)
                        .mt(2.)
                        .key("help-close"),
                    ),
            )
    }
}
