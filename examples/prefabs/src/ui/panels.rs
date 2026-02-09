use insim::insim::BtnStyle;
use kitcar::ui;

use super::{PrefabListItem, PrefabViewMessage};

pub(super) fn prefabs_panel(prefabs: &[PrefabListItem]) -> ui::Node<PrefabViewMessage> {
    ui::container()
        .mt(1.)
        .mb(2.)
        .flex()
        .flex_col()
        .with_child(
            ui::container()
                .flex()
                .flex_row()
                .with_child(
                    ui::clickable(
                        "Reload YAML",
                        BtnStyle::default().pale_blue().light(),
                        PrefabViewMessage::ReloadYaml,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::typein(
                        "Save Selection",
                        BtnStyle::default().green().light(),
                        64,
                        PrefabViewMessage::SavePrefab,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                ),
        )
        .with_children(prefabs.iter().enumerate().map(|(idx, prefab)| {
            ui::clickable(
                format!("{} [{}]", prefab.name, prefab.count),
                BtnStyle::default().black().light().align_left(),
                PrefabViewMessage::SpawnPrefab(idx),
            )
            .key(format!("prefab-{idx}"))
            .h(5.)
        }))
}

pub(super) fn nudge_panel(nudge_distance_metres: f64) -> ui::Node<PrefabViewMessage> {
    ui::container()
        .mt(1.)
        .mb(2.)
        .flex()
        .flex_col()
        .with_child(
            ui::typein(
                format!("Nudge Distance ({:.2}m)", nudge_distance_metres),
                BtnStyle::default().black().light(),
                16,
                PrefabViewMessage::NudgeDistanceInput,
            )
            .block()
            .h(5.),
        )
        .with_child(
            ui::container()
                .mt(1.)
                .flex()
                .flex_row()
                .with_child(
                    ui::clickable(
                        "N",
                        BtnStyle::default().pale_blue().light(),
                        PrefabViewMessage::NudgeNorth,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "W",
                        BtnStyle::default().pale_blue().light(),
                        PrefabViewMessage::NudgeWest,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "S",
                        BtnStyle::default().pale_blue().light(),
                        PrefabViewMessage::NudgeSouth,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "E",
                        BtnStyle::default().pale_blue().light(),
                        PrefabViewMessage::NudgeEast,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                ),
        )
}

pub(super) fn options_panel(
    compass_visible: bool,
    display_selection_info: bool,
) -> ui::Node<PrefabViewMessage> {
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
                PrefabViewMessage::ToggleCompass,
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
                PrefabViewMessage::ToggleSelectionInfo,
            )
            .mt(1.)
            .h(5.),
        )
}
