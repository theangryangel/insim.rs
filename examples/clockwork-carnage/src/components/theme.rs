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

pub fn hud_action() -> BtnStyle {
    BtnStyle::default().dark().pale_blue()
}
