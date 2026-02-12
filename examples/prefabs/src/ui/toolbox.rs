use insim::{
    core::heading::Heading,
    insim::{BtnStyle, ObjectInfo, PmoAction},
};
use kitcar::ui;

use super::{OptionsMsg, PrefabSummary, ToolboxProps, options};
use crate::{Command, SpawnOrigin, State, tools};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorTool {
    Prefabs,
    Ramp,
    Nudge,
    Options,
}

impl InspectorTool {
    fn title(self) -> &'static str {
        match self {
            Self::Prefabs => "Prefabs",
            Self::Ramp => "Ramp Tool",
            Self::Nudge => "Nudge Selection",
            Self::Options => "Options",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToolboxScreen {
    #[default]
    Launcher,
    Inspector(InspectorTool),
}

#[derive(Debug, Clone)]
pub enum ToolboxMsg {
    OpenInspector(InspectorTool),
    BackToLauncher,
    Options(OptionsMsg),
    ReloadYaml,
    SavePrefab(String),
    SpawnPrefab(usize),
    PaintedTextInput(String),
    RotateInput(String),
    SplineDistribInput(String),
    ToggleRampMode,
    RampRollInput(String),
    BuildRamp,
    NudgeDistanceInput(String),
    Nudge(Heading),
    JiggleSelection,
}

#[derive(Debug, Default)]
pub struct Toolbox {
    screen: ToolboxScreen,
}

impl ui::Component for Toolbox {
    type Props = ToolboxProps;
    type Message = ToolboxMsg;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            ToolboxMsg::OpenInspector(tool) => {
                self.screen = match self.screen {
                    ToolboxScreen::Inspector(current) if current == tool => ToolboxScreen::Launcher,
                    _ => ToolboxScreen::Inspector(tool),
                };
            },
            ToolboxMsg::BackToLauncher => {
                self.screen = ToolboxScreen::Launcher;
            },
            _ => {},
        }
    }

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        if !props.ui_visible {
            return ui::empty();
        }

        let content = match self.screen {
            ToolboxScreen::Launcher => launcher_screen(),
            ToolboxScreen::Inspector(tool) => inspector_screen(tool, &props),
        };

        ui::container()
            .flex()
            .flex_col()
            .w(170.)
            .pt(7.)
            .items_end()
            .with_child(if props.display_selection_info {
                ui::text(
                    format!("Selection: {} object(s)", props.selection_count),
                    BtnStyle::style_readonly(),
                )
                .w(48.)
                .h(5.)
            } else {
                ui::empty()
            })
            .with_child(if let Some(compass_text) = props.compass_text.as_ref() {
                ui::text(compass_text, BtnStyle::style_readonly())
                    .w(48.)
                    .h(5.)
            } else {
                ui::empty()
            })
            .with_child(content)
    }
}

fn launcher_button(label: &'static str, tool: InspectorTool) -> ui::Node<ToolboxMsg> {
    ui::clickable(
        label,
        BtnStyle::style_interactive(),
        ToolboxMsg::OpenInspector(tool),
    )
    .h(5.)
}

fn launcher_screen() -> ui::Node<ToolboxMsg> {
    ui::container().flex().flex_col().w(48.).with_children([
        launcher_button("Prefabs", InspectorTool::Prefabs),
        ui::typein(
            "Spline Distribution (m)",
            BtnStyle::style_interactive(),
            32,
            ToolboxMsg::SplineDistribInput,
        )
        .block()
        .h(5.),
        ui::typein(
            "Paint Text",
            BtnStyle::style_interactive(),
            64,
            ToolboxMsg::PaintedTextInput,
        )
        .block()
        .h(5.),
        ui::typein(
            "Rotate Selection (deg)",
            BtnStyle::style_interactive(),
            16,
            ToolboxMsg::RotateInput,
        )
        .block()
        .h(5.),
        launcher_button("Ramp Tool", InspectorTool::Ramp),
        launcher_button("Nudge Selection", InspectorTool::Nudge),
        ui::clickable(
            "Jiggle Selection",
            BtnStyle::style_interactive(),
            ToolboxMsg::JiggleSelection,
        )
        .h(5.),
        launcher_button("Options", InspectorTool::Options),
    ])
}

fn inspector_screen(tool: InspectorTool, props: &ToolboxProps) -> ui::Node<ToolboxMsg> {
    let body = match tool {
        InspectorTool::Prefabs => prefabs_panel(&props.prefabs),
        InspectorTool::Ramp => ramp_panel(props.ramp_mode, props.ramp_roll_degrees),
        InspectorTool::Nudge => nudge_panel(props.nudge_distance_metres),
        InspectorTool::Options => {
            options::panel(props.compass_visible, props.display_selection_info)
                .map(ToolboxMsg::Options)
        },
    };

    ui::container()
        .flex()
        .flex_col()
        .w(48.)
        .with_child(
            ui::container()
                .flex()
                .flex_row()
                .with_child(
                    ui::clickable(
                        "Back",
                        BtnStyle::style_interactive(),
                        ToolboxMsg::BackToLauncher,
                    )
                    .w(12.)
                    .h(5.),
                )
                .with_child(
                    ui::text(tool.title(), BtnStyle::style_title())
                        .h(5.)
                        .flex_grow(1.0),
                ),
        )
        .with_child(body)
}

fn prefabs_panel(prefabs: &[PrefabSummary]) -> ui::Node<ToolboxMsg> {
    ui::container()
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
                BtnStyle::style_interactive().align_left(),
                ToolboxMsg::SpawnPrefab(idx),
            )
            .key(format!("prefab-{idx}"))
            .h(5.)
        }))
}

fn nudge_panel(nudge_distance_metres: f64) -> ui::Node<ToolboxMsg> {
    let blank_cell = || {
        ui::text(" ", BtnStyle::style_unavailable())
            .flex_grow(1.0)
            .h(5.)
    };

    let nudge_cell = |label: &'static str, heading: Heading| {
        ui::clickable(
            label,
            BtnStyle::style_interactive(),
            ToolboxMsg::Nudge(heading),
        )
        .flex_grow(1.0)
        .h(5.)
    };

    ui::container()
        .flex()
        .flex_col()
        .with_child(
            ui::typein(
                format!("Nudge Distance ({:.2}m)", nudge_distance_metres),
                BtnStyle::style_interactive(),
                16,
                ToolboxMsg::NudgeDistanceInput,
            )
            .block()
            .h(5.),
        )
        .with_child(
            ui::container()
                .flex()
                .flex_row()
                .with_child(blank_cell())
                .with_child(nudge_cell("N", Heading::NORTH))
                .with_child(blank_cell()),
        )
        .with_child(
            ui::container()
                .flex()
                .flex_row()
                .with_child(nudge_cell("W", Heading::WEST))
                .with_child(blank_cell())
                .with_child(nudge_cell("E", Heading::EAST)),
        )
        .with_child(
            ui::container()
                .flex()
                .flex_row()
                .with_child(blank_cell())
                .with_child(nudge_cell("S", Heading::SOUTH))
                .with_child(blank_cell()),
        )
}

fn ramp_panel(ramp_mode: tools::ramp::RampMode, ramp_roll_degrees: f64) -> ui::Node<ToolboxMsg> {
    let mode_label = match ramp_mode {
        tools::ramp::RampMode::AlongPath => "Mode: Grade (Along Path)",
        tools::ramp::RampMode::AcrossPath => "Mode: Bank (Across Path)",
    };

    ui::container()
        .flex()
        .flex_col()
        .with_child(
            ui::clickable(
                mode_label,
                BtnStyle::style_active(),
                ToolboxMsg::ToggleRampMode,
            )
            .h(5.),
        )
        .with_child(
            ui::typein(
                format!("Roll Degrees ({ramp_roll_degrees:.1})"),
                BtnStyle::style_interactive(),
                16,
                ToolboxMsg::RampRollInput,
            )
            .block()
            .h(5.),
        )
        .with_child(
            ui::clickable(
                "Build Ramp",
                BtnStyle::style_interactive(),
                ToolboxMsg::BuildRamp,
            )
            .h(5.),
        )
}

pub(super) fn reduce(state: &mut State, msg: ToolboxMsg) -> Option<Command> {
    match msg {
        ToolboxMsg::OpenInspector(_) | ToolboxMsg::BackToLauncher => None,
        ToolboxMsg::Options(options_msg) => options::reduce(state, options_msg),
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
        ToolboxMsg::ToggleRampMode => {
            state.ramp_mode = state.ramp_mode.toggled();
            tracing::info!(
                "Ramp mode set to {}",
                match state.ramp_mode {
                    tools::ramp::RampMode::AlongPath => "grade",
                    tools::ramp::RampMode::AcrossPath => "bank",
                }
            );
            None
        },
        ToolboxMsg::RampRollInput(input) => {
            let trimmed = input.trim();

            if trimmed.is_empty() {
                tracing::warn!("ramp roll skipped: input is empty");
                return None;
            }

            match trimmed.parse::<f64>() {
                Ok(value) if value.is_finite() && value.abs() <= 90.0 => {
                    state.ramp_roll_degrees = value;
                    tracing::info!("Set ramp roll to {value} degrees");
                },
                Ok(_) => {
                    tracing::warn!(
                        "ramp roll skipped: value must be finite and between -90 and 90"
                    );
                },
                Err(_) => {
                    tracing::warn!("ramp roll skipped: input is not a number");
                },
            }

            None
        },
        ToolboxMsg::BuildRamp => {
            match tools::ramp::build(
                &state.selection,
                tools::ramp::BuildConfig {
                    mode: state.ramp_mode,
                    roll_degrees: state.ramp_roll_degrees,
                    steps_per_segment: None,
                },
            ) {
                Ok(objects) => Some(Command::SpawnObjects {
                    objects,
                    action: PmoAction::AddObjects,
                    origin: SpawnOrigin::Ramp {
                        mode: state.ramp_mode,
                        roll_degrees: state.ramp_roll_degrees,
                    },
                }),
                Err(err) => {
                    tracing::warn!("ramp skipped: {err}");
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
