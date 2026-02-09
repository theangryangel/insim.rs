use insim::{core::heading::Heading, insim::BtnStyle};
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
                        PrefabViewMessage::Nudge(Heading::NORTH),
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "W",
                        BtnStyle::default().pale_blue().light(),
                        PrefabViewMessage::Nudge(Heading::WEST),
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "S",
                        BtnStyle::default().pale_blue().light(),
                        PrefabViewMessage::Nudge(Heading::SOUTH),
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "E",
                        BtnStyle::default().pale_blue().light(),
                        PrefabViewMessage::Nudge(Heading::EAST),
                    )
                    .flex_grow(1.0)
                    .h(5.),
                ),
        )
}
