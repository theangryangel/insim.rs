use insim::insim::BtnStyle;
use kitcar::ui;

use super::{OptionsMsg, ToolboxMsg, panels};
use crate::ui::{ExpandedSection, PrefabViewProps};

pub(super) fn toolbox_tab(
    props: &PrefabViewProps,
    expanded_section: ExpandedSection,
) -> ui::Node<ToolboxMsg> {
    let prefabs_section_style = if matches!(expanded_section, ExpandedSection::Prefabs) {
        BtnStyle::default().yellow().light().clickable()
    } else {
        BtnStyle::default().pale_blue().light().clickable()
    };
    let nudge_section_style = if matches!(expanded_section, ExpandedSection::Nudge) {
        BtnStyle::default().yellow().light().clickable()
    } else {
        BtnStyle::default().pale_blue().light().clickable()
    };

    let mut children = Vec::with_capacity(10);
    children.push(
        ui::clickable(
            "Prefabs",
            prefabs_section_style,
            ToolboxMsg::ExpandToolboxSection(ExpandedSection::Prefabs),
        )
        .mt(1.)
        .h(5.),
    );

    if matches!(expanded_section, ExpandedSection::Prefabs) {
        children.push(panels::prefabs_panel(&props.prefabs));
    }

    children.extend([
        ui::typein(
            "Spline Distribution (m)",
            BtnStyle::default().black().light(),
            32,
            ToolboxMsg::SplineDistribInput,
        )
        .mt(1.)
        .block()
        .h(5.),
        ui::typein(
            "Paint Text",
            BtnStyle::default().black().light(),
            64,
            ToolboxMsg::PaintedTextInput,
        )
        .mt(1.)
        .block()
        .h(5.),
        ui::typein(
            "Rotate Selection (deg)",
            BtnStyle::default().black().light(),
            16,
            ToolboxMsg::RotateInput,
        )
        .mt(1.)
        .block()
        .h(5.),
        ui::clickable(
            "Jiggle Selection",
            BtnStyle::default().pale_blue().light().clickable(),
            ToolboxMsg::JiggleSelection,
        )
        .mt(1.)
        .h(5.),
        ui::clickable(
            "Nudge Selection",
            nudge_section_style,
            ToolboxMsg::ExpandToolboxSection(ExpandedSection::Nudge),
        )
        .mt(1.)
        .h(5.),
    ]);

    if matches!(expanded_section, ExpandedSection::Nudge) {
        children.push(panels::nudge_panel(props.nudge_distance_metres));
    }

    ui::container()
        .flex()
        .flex_col()
        .w(48.)
        .with_children(children)
}

pub(super) fn options_tab(
    compass_visible: bool,
    display_selection_info: bool,
) -> ui::Node<OptionsMsg> {
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
