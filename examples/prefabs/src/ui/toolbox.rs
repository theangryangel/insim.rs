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
    Grid,
    Nudge,
    Options,
}

impl InspectorTool {
    fn title(self) -> &'static str {
        match self {
            Self::Prefabs => "Prefabs",
            Self::Ramp => "Ramp Tool",
            Self::Grid => "Grid Tool",
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
    GridMode(tools::grid::GridMode),
    GridWidthInput(String),
    GridRowsInput(String),
    GridColSpacingInput(String),
    GridRowSpacingInput(String),
    GridLateralOffsetInput(String),
    BuildGrid,
    NudgeDistanceInput(String),
    Nudge(Heading),
    JiggleSelection,
    ToggleTopDown,
    ToggleSideView,
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
            ToolboxScreen::Launcher => launcher_screen(&props),
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

fn launcher_screen(props: &ToolboxProps) -> ui::Node<ToolboxMsg> {
    let has_selection = props.selection_count > 0;
    let selection_btn_style = if has_selection {
        BtnStyle::style_interactive()
    } else {
        BtnStyle::style_unavailable()
    };

    let mut ramp_tool_btn = launcher_button("Ramp Tool", InspectorTool::Ramp);
    if !has_selection {
        *ramp_tool_btn.bstyle_mut() = BtnStyle::style_unavailable();
    }

    let mut nudge_selection_btn = launcher_button("Nudge Selection", InspectorTool::Nudge);
    if !has_selection {
        *nudge_selection_btn.bstyle_mut() = BtnStyle::style_unavailable();
    }

    let mut ramp_tool_btn = launcher_button("Ramp Tool", InspectorTool::Ramp);
    if !has_selection {
        *ramp_tool_btn.bstyle_mut() = BtnStyle::style_unavailable();
    }

    let mut grid_tool_btn = launcher_button("Grid Tool", InspectorTool::Grid);
    if !has_selection {
        *grid_tool_btn.bstyle_mut() = BtnStyle::style_unavailable();
    }

    let mut nudge_selection_btn = launcher_button("Nudge Selection", InspectorTool::Nudge);
    if !has_selection {
        *nudge_selection_btn.bstyle_mut() = BtnStyle::style_unavailable();
    }

    ui::container().flex().flex_col().w(48.).with_children([
        launcher_button("Prefabs", InspectorTool::Prefabs),
        ui::typein(
            "Spline Distribution (m)",
            selection_btn_style,
            32,
            ToolboxMsg::SplineDistribInput,
        )
        .block()
        .h(5.),
        ui::typein(
            "Paint Text",
            selection_btn_style,
            64,
            ToolboxMsg::PaintedTextInput,
        )
        .block()
        .h(5.),
        ui::typein(
            "Rotate Selection (deg)",
            selection_btn_style,
            16,
            ToolboxMsg::RotateInput,
        )
        .block()
        .h(5.),
        ramp_tool_btn,
        grid_tool_btn,
        nudge_selection_btn,
        ui::clickable(
            "Jiggle Selection",
            selection_btn_style,
            ToolboxMsg::JiggleSelection,
        )
        .h(5.),
        ui::clickable("Top Down View", selection_btn_style, ToolboxMsg::ToggleTopDown).h(5.),
        ui::clickable("Side View", selection_btn_style, ToolboxMsg::ToggleSideView).h(5.),
        launcher_button("Options", InspectorTool::Options),
    ])
}

fn inspector_screen(tool: InspectorTool, props: &ToolboxProps) -> ui::Node<ToolboxMsg> {
    let body = match tool {
        InspectorTool::Prefabs => prefabs_panel(&props.prefabs),
        InspectorTool::Ramp => ramp_panel(props.ramp_mode, props.ramp_roll_degrees),
        InspectorTool::Grid => grid_panel(
            props.grid_mode,
            props.grid_width,
            props.grid_rows,
            props.grid_col_spacing,
            props.grid_row_spacing,
            props.grid_lateral_offset,
        ),
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
                prefab.name.clone(),
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

fn grid_panel(
    grid_mode: tools::grid::GridMode,
    grid_width: usize,
    grid_rows: usize,
    grid_col_spacing: f64,
    grid_row_spacing: f64,
    grid_lateral_offset: f64,
) -> ui::Node<ToolboxMsg> {
    let mode_label = match grid_mode {
        tools::grid::GridMode::StartGrid => "Mode: Start Grid",
        tools::grid::GridMode::Pit => "Mode: Pit",
        tools::grid::GridMode::PitBox => "Mode: Pit Box",
    };

    ui::container()
        .flex()
        .flex_col()
        .with_child(
            ui::clickable(
                mode_label,
                BtnStyle::style_active(),
                ToolboxMsg::GridMode(grid_mode.cycled()),
            )
            .h(5.),
        )
        .with_child(
            ui::container()
                .flex()
                .flex_row()
                .with_child(
                    ui::typein(
                        format!("Width ({grid_width})"),
                        BtnStyle::style_interactive(),
                        8,
                        ToolboxMsg::GridWidthInput,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::typein(
                        format!("Rows ({grid_rows})"),
                        BtnStyle::style_interactive(),
                        8,
                        ToolboxMsg::GridRowsInput,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                ),
        )
        .with_child(
            ui::container()
                .flex()
                .flex_row()
                .with_child(
                    ui::typein(
                        format!("Col Spacing ({grid_col_spacing:.1}m)"),
                        BtnStyle::style_interactive(),
                        8,
                        ToolboxMsg::GridColSpacingInput,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                )
                .with_child(
                    ui::typein(
                        format!("Row Spacing ({grid_row_spacing:.1}m)"),
                        BtnStyle::style_interactive(),
                        8,
                        ToolboxMsg::GridRowSpacingInput,
                    )
                    .flex_grow(1.0)
                    .h(5.),
                ),
        )
        .with_child(
            ui::typein(
                format!("Lateral Offset ({grid_lateral_offset:.1}m)"),
                BtnStyle::style_interactive(),
                8,
                ToolboxMsg::GridLateralOffsetInput,
            )
            .block()
            .h(5.),
        )
        .with_child(
            ui::clickable("Build Grid", BtnStyle::style_interactive(), ToolboxMsg::BuildGrid)
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
            if idx >= state.prefabs.entries.len() {
                return None;
            }
            Some(Command::SpawnPrefab(idx))
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
        ToolboxMsg::GridMode(mode) => {
            state.grid_mode = mode;
            None
        },
        ToolboxMsg::GridWidthInput(input) => {
            match input.trim().parse::<usize>() {
                Ok(value) if value > 0 => {
                    state.grid_width = value;
                },
                Ok(_) => tracing::warn!("grid width skipped: must be at least 1"),
                Err(_) => tracing::warn!("grid width skipped: not a valid integer"),
            }
            None
        },
        ToolboxMsg::GridRowsInput(input) => {
            match input.trim().parse::<usize>() {
                Ok(value) if value > 0 => {
                    state.grid_rows = value;
                },
                Ok(_) => tracing::warn!("grid rows skipped: must be at least 1"),
                Err(_) => tracing::warn!("grid rows skipped: not a valid integer"),
            }
            None
        },
        ToolboxMsg::GridColSpacingInput(input) => {
            match input.trim().parse::<f64>() {
                Ok(value) if value.is_finite() && value > 0.0 => {
                    state.grid_col_spacing = value;
                },
                Ok(_) => tracing::warn!("grid col spacing skipped: must be a positive finite number"),
                Err(_) => tracing::warn!("grid col spacing skipped: not a number"),
            }
            None
        },
        ToolboxMsg::GridRowSpacingInput(input) => {
            match input.trim().parse::<f64>() {
                Ok(value) if value.is_finite() && value > 0.0 => {
                    state.grid_row_spacing = value;
                },
                Ok(_) => tracing::warn!("grid row spacing skipped: must be a positive finite number"),
                Err(_) => tracing::warn!("grid row spacing skipped: not a number"),
            }
            None
        },
        ToolboxMsg::GridLateralOffsetInput(input) => {
            match input.trim().parse::<f64>() {
                Ok(value) if value.is_finite() => {
                    state.grid_lateral_offset = value;
                },
                Ok(_) => tracing::warn!("grid lateral offset skipped: must be finite"),
                Err(_) => tracing::warn!("grid lateral offset skipped: not a number"),
            }
            None
        },
        ToolboxMsg::BuildGrid => {
            match tools::grid::build(
                &state.selection,
                tools::grid::BuildConfig {
                    mode: state.grid_mode,
                    width: state.grid_width,
                    rows: state.grid_rows,
                    col_spacing: state.grid_col_spacing,
                    row_spacing: state.grid_row_spacing,
                    lateral_offset: state.grid_lateral_offset,
                },
            ) {
                Ok(objects) => Some(Command::SpawnObjects {
                    objects,
                    action: PmoAction::AddObjects,
                    origin: SpawnOrigin::Grid {
                        mode: state.grid_mode,
                        width: state.grid_width,
                        rows: state.grid_rows,
                    },
                }),
                Err(err) => {
                    tracing::warn!("grid skipped: {err}");
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
        ToolboxMsg::ToggleTopDown => {
            if state.selection.is_empty() {
                return None;
            }
            tools::camera::get_top_down_view(&state.selection, &state.last_cpp)
                .map(Command::CameraMove)
        },
        ToolboxMsg::ToggleSideView => {
            if state.selection.is_empty() {
                return None;
            }

            tools::camera::get_side_view(&state.selection, &state.last_cpp)
                .map(Command::CameraMove)
        },
    }
}
