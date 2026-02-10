use insim::insim::BtnStyle;
use kitcar::ui;

use crate::{Command, State};

#[derive(Debug, Clone, Copy)]
pub enum OptionsMsg {
    ToggleCompass,
    ToggleSelectionInfo,
}

pub(super) fn tab(compass_visible: bool, display_selection_info: bool) -> ui::Node<OptionsMsg> {
    ui::container()
        .mt(1.)
        .mb(2.)
        .flex()
        .flex_col()
        .w(48.)
        .with_child(
            ui::clickable(
                if compass_visible {
                    "Compass: On"
                } else {
                    "Compass: Off"
                },
                if compass_visible {
                    BtnStyle::default().green().light().clickable()
                } else {
                    BtnStyle::default().pale_blue().light().clickable()
                },
                OptionsMsg::ToggleCompass,
            )
            .h(5.),
        )
        .with_child(
            ui::clickable(
                if display_selection_info {
                    "Selected Objects: Yes"
                } else {
                    "Selected Objects: No"
                },
                if display_selection_info {
                    BtnStyle::default().green().light().clickable()
                } else {
                    BtnStyle::default().pale_blue().light().clickable()
                },
                OptionsMsg::ToggleSelectionInfo,
            )
            .mt(1.)
            .h(5.),
        )
}

pub(super) fn reduce(state: &mut State, msg: OptionsMsg) -> Option<Command> {
    match msg {
        OptionsMsg::ToggleSelectionInfo => None,
        OptionsMsg::ToggleCompass => {
            state.compass_visible = !state.compass_visible;
            if !state.compass_visible {
                state.compass_text = None;
            }
            tracing::info!(
                "Compass {}",
                if state.compass_visible {
                    "enabled"
                } else {
                    "disabled"
                }
            );

            None
        },
    }
}
