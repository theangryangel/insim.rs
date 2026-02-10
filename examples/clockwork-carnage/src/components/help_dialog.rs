use kitcar::ui;

use super::{hud_action, hud_text, hud_title};

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
            .with_child(ui::text("HELP", hud_title()).w(50.).h(5.))
            .with_child(
                ui::text("Get as close as possible to the target time!", hud_text())
                    .w(100.)
                    .h(5.),
            )
            .with_child(
                ui::text("Closest time gets more points!", hud_text())
                    .w(100.)
                    .h(5.),
            )
            .with_child(
                ui::clickable("Close", hud_action(), HelpDialogMsg::Hide)
                    .w(100.)
                    .h(5.),
            )
    }
}
