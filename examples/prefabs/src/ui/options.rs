use insim::insim::BtnStyle;
use kitcar::ui;

use crate::{Command, State};

#[derive(Debug, Clone, Copy)]
pub enum OptionsMsg {
    ToggleCompass,
    ToggleSelectionInfo,
}

pub(super) fn panel(compass_visible: bool, display_selection_info: bool) -> ui::Node<OptionsMsg> {
    ui::container()
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
                    BtnStyle::style_active()
                } else {
                    BtnStyle::style_interactive()
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
                    BtnStyle::style_active()
                } else {
                    BtnStyle::style_interactive()
                },
                OptionsMsg::ToggleSelectionInfo,
            )
            .h(5.),
        )
}

pub(super) fn reduce(state: &mut State, msg: OptionsMsg) -> Option<Command> {
    match msg {
        OptionsMsg::ToggleSelectionInfo => {
            state.display_selection_info = !state.display_selection_info;
            tracing::info!(
                "Selection info {}",
                if state.display_selection_info {
                    "enabled"
                } else {
                    "disabled"
                }
            );

            None
        },
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
