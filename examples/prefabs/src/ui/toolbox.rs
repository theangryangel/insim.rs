use insim::{
    core::heading::Heading,
    insim::{BtnStyle, ObjectInfo, PmoAction},
};
use kitcar::ui;

use super::{PrefabListItem, PrefabViewProps};
use crate::{Command, SpawnOrigin, State, tools};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExpandedSection {
    #[default]
    None,
    Prefabs,
    Nudge,
}

#[derive(Debug, Clone)]
pub enum ToolboxMsg {
    ExpandToolboxSection(ExpandedSection),
    ReloadYaml,
    SavePrefab(String),
    SpawnPrefab(usize),
    PaintedTextInput(String),
    RotateInput(String),
    SplineDistribInput(String),
    NudgeDistanceInput(String),
    Nudge(Heading),
    JiggleSelection,
}

#[derive(Debug, Default)]
pub(super) struct Toolbox {
    expanded_section: ExpandedSection,
}

impl ui::Component for Toolbox {
    type Props = PrefabViewProps;
    type Message = ToolboxMsg;

    fn update(&mut self, msg: Self::Message) {
        if let ToolboxMsg::ExpandToolboxSection(section) = msg {
            self.expanded_section = if self.expanded_section == section {
                ExpandedSection::None
            } else {
                section
            };
        }
    }

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        let prefabs_section_style = if matches!(self.expanded_section, ExpandedSection::Prefabs) {
            BtnStyle::default().yellow().light().clickable()
        } else {
            BtnStyle::default().pale_blue().light().clickable()
        };
        let nudge_section_style = if matches!(self.expanded_section, ExpandedSection::Nudge) {
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

        if matches!(self.expanded_section, ExpandedSection::Prefabs) {
            children.push(prefabs_panel(&props.prefabs));
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

        if matches!(self.expanded_section, ExpandedSection::Nudge) {
            children.push(nudge_panel(props.nudge_distance_metres));
        }

        ui::container()
            .flex()
            .flex_col()
            .w(48.)
            .with_children(children)
    }
}

fn prefabs_panel(prefabs: &[PrefabListItem]) -> ui::Node<ToolboxMsg> {
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
                        ToolboxMsg::ReloadYaml,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::typein(
                        "Save Selection",
                        BtnStyle::default().green().light(),
                        64,
                        ToolboxMsg::SavePrefab,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                ),
        )
        .with_children(prefabs.iter().enumerate().map(|(idx, prefab)| {
            ui::clickable(
                format!("{} [{}]", prefab.name, prefab.count),
                BtnStyle::default().black().light().align_left(),
                ToolboxMsg::SpawnPrefab(idx),
            )
            .key(format!("prefab-{idx}"))
            .h(5.)
        }))
}

fn nudge_panel(nudge_distance_metres: f64) -> ui::Node<ToolboxMsg> {
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
                ToolboxMsg::NudgeDistanceInput,
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
                        ToolboxMsg::Nudge(Heading::NORTH),
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "W",
                        BtnStyle::default().pale_blue().light(),
                        ToolboxMsg::Nudge(Heading::WEST),
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "S",
                        BtnStyle::default().pale_blue().light(),
                        ToolboxMsg::Nudge(Heading::SOUTH),
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::clickable(
                        "E",
                        BtnStyle::default().pale_blue().light(),
                        ToolboxMsg::Nudge(Heading::EAST),
                    )
                    .flex_grow(1.0)
                    .h(5.),
                ),
        )
}

pub(super) fn reduce(state: &mut State, msg: ToolboxMsg) -> Option<Command> {
    match msg {
        ToolboxMsg::ExpandToolboxSection(_) => None,
        ToolboxMsg::ReloadYaml => Some(Command::ReloadPrefabs),
        ToolboxMsg::SavePrefab(name) => Some(Command::SavePrefabs(name.trim().to_string())),
        ToolboxMsg::SpawnPrefab(idx) => {
            let Some(prefab) = state.prefabs.data.get(idx) else {
                return None;
            };

            let anchor = state
                .selection
                .first()
                .map(|obj| *obj.position())
                .unwrap_or_default();

            Some(Command::SpawnObjects {
                objects: prefab.place_at_anchor(anchor),
                action: PmoAction::Selection,
                origin: SpawnOrigin::Prefab,
            })
        },
        ToolboxMsg::PaintedTextInput(text) => {
            let text = text.trim().to_string();
            if text.is_empty() {
                tracing::warn!("paint skipped: text input is empty");
                return None;
            }

            let anchor = state
                .selection
                .first()
                .map(|obj| *obj.position())
                .unwrap_or_default();
            let heading = state
                .selection
                .first()
                .and_then(ObjectInfo::heading)
                .unwrap_or_default();
            let objects = tools::painted_letters::build(&text, anchor, heading);

            if objects.is_empty() {
                tracing::warn!("paint skipped: text has no supported painted-letter characters");
                None
            } else {
                Some(Command::SpawnObjects {
                    objects,
                    action: PmoAction::Selection,
                    origin: SpawnOrigin::PaintedText,
                })
            }
        },
        ToolboxMsg::SplineDistribInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("spacing skipped: input is empty");
                return None;
            }

            match trimmed.parse::<f64>() {
                Ok(value) if value > 0.0 => {
                    match tools::spline_distrib::build(&state.selection, value, None) {
                        Ok(objects) => Some(Command::SpawnObjects {
                            objects,
                            action: PmoAction::AddObjects,
                            origin: SpawnOrigin::SplineDistrib {
                                spacing_metres: value,
                            },
                        }),
                        Err(err) => {
                            tracing::warn!("spacing skipped: {err}");
                            None
                        },
                    }
                },
                Ok(_) => {
                    tracing::warn!("spacing skipped: value must be greater than zero");
                    None
                },
                Err(_) => {
                    tracing::warn!("spacing skipped: input is not a number");
                    None
                },
            }
        },
        ToolboxMsg::RotateInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("rotation skipped: input is empty");
                return None;
            }

            match trimmed.parse::<f64>() {
                Ok(value) if value.is_finite() => {
                    match tools::rotate::build(&state.selection, value) {
                        Ok(objects) => Some(Command::SpawnObjects {
                            objects,
                            action: PmoAction::AddObjects,
                            origin: SpawnOrigin::Rotate { degrees: value },
                        }),
                        Err(err) => {
                            tracing::warn!("rotation skipped: {err}");
                            None
                        },
                    }
                },
                Ok(_) => {
                    tracing::warn!("rotation skipped: value must be finite");
                    None
                },
                Err(_) => {
                    tracing::warn!("rotation skipped: input is not a number");
                    None
                },
            }
        },
        ToolboxMsg::NudgeDistanceInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("nudge skipped: input is empty");
                return None;
            }

            match trimmed.parse::<f64>() {
                Ok(value) if value.is_finite() && value > 0.0 => {
                    state.nudge_distance_metres = value;
                    tracing::info!("Set nudge distance to {value} metres");
                },
                Ok(_) => {
                    tracing::warn!("nudge skipped: value must be finite and greater than zero");
                },
                Err(_) => {
                    tracing::warn!("nudge skipped: input is not a number");
                },
            }

            None
        },
        ToolboxMsg::Nudge(heading) => Some(Command::SpawnObjects {
            objects: tools::nudge::nudge(
                &state.selection,
                heading.clone(),
                state.nudge_distance_metres,
            ),
            action: PmoAction::AddObjects,
            origin: SpawnOrigin::Nudge {
                heading,
                distance_metres: state.nudge_distance_metres,
            },
        }),
        ToolboxMsg::JiggleSelection => {
            if state.selection.is_empty() {
                tracing::warn!("jiggle skipped: selection is empty");
                None
            } else {
                Some(Command::SpawnObjects {
                    objects: tools::jiggle::jiggle(&state.selection, 5.0, 3.5),
                    action: PmoAction::AddObjects,
                    origin: SpawnOrigin::JiggleSelection,
                })
            }
        },
    }
}
