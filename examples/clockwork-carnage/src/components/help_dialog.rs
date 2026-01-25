use insim::{core::string::colours::Colourify, insim::BtnStyle};
use kitcar::ui;

#[derive(Default)]
pub struct HelpDialog {
    visible: bool,
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

        // Help dialog content
        ui::container()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .w(200.)
            .h(100.)
            .with_child(
                ui::text("HELP".yellow(), BtnStyle::default().dark())
                    .w(50.)
                    .h(5.),
            )
            .with_child(
                ui::text(
                    "Get as close as possible to the target time!".white(),
                    BtnStyle::default().dark(),
                )
                .w(100.)
                .h(5.),
            )
            .with_child(
                ui::text(
                    "Closest time gets more points!".white(),
                    BtnStyle::default().dark(),
                )
                .w(100.)
                .h(5.),
            )
            .with_child(
                ui::clickable(
                    "Close",
                    BtnStyle::default().dark().green(),
                    HelpDialogMsg::Hide,
                )
                .w(100.)
                .h(5.),
            )
    }
}
