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
