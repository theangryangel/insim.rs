use insim::insim::BtnStyle;

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

pub fn hud_panel_border() -> BtnStyle {
    BtnStyle::default().dark().grey()
}

pub fn hud_panel_bg() -> BtnStyle {
    BtnStyle::default().dark().light_grey()
}

pub fn hud_overlay_title() -> BtnStyle {
    BtnStyle::default().yellow()
}

pub fn hud_overlay_text() -> BtnStyle {
    BtnStyle::default().light_grey()
}

pub fn hud_overlay_muted() -> BtnStyle {
    BtnStyle::default().grey()
}

pub fn hud_overlay_action() -> BtnStyle {
    BtnStyle::default().pale_blue()
}
