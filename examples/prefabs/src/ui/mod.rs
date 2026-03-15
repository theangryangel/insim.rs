mod options;
mod toolbox;

pub use options::OptionsMsg;
pub use toolbox::{Toolbox, ToolboxMsg};

#[derive(Debug, Clone, Default)]
pub struct PrefabSummary {
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct ToolboxProps {
    pub ui_visible: bool,
    pub display_selection_info: bool,
    pub selection_count: usize,
    pub prefabs: Vec<PrefabSummary>,
    pub nudge_distance_metres: f64,
    pub ramp_mode: crate::tools::ramp::RampMode,
    pub ramp_roll_degrees: f64,
    pub grid_mode: crate::tools::grid::GridMode,
    pub grid_width: usize,
    pub grid_rows: usize,
    pub grid_col_spacing: f64,
    pub grid_row_spacing: f64,
    pub grid_lateral_offset: f64,
    pub compass_visible: bool,
    pub compass_text: Option<String>,
}

pub(crate) fn reduce_message(state: &mut crate::State, msg: ToolboxMsg) -> Option<crate::Command> {
    toolbox::reduce(state, msg)
}
