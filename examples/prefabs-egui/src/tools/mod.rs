//! Tools

/// ToolKind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    /// Selection
    Select,
    /// Place
    Place,
    /// Spline Path
    SplinePath,
    /// Ramp Generator
    RampGen,
}

/// Selection tool state.
#[derive(Debug, Default, PartialEq)]
pub struct SelectState {
    /// Selected object IDs.
    pub selected_object_ids: Vec<u64>,
}

/// Placement tool state.
#[derive(Debug, PartialEq)]
pub struct PlaceState {
    /// Search query.
    pub search_query: String,
    /// Selected object type.
    pub selected_object_type: u8,
}

impl Default for PlaceState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            selected_object_type: 16,
        }
    }
}

/// Spline path tool state.
#[derive(Debug, Default, PartialEq)]
pub struct SplinePathState;

/// Ramp generator tool state.
#[derive(Debug, Default, PartialEq)]
pub struct RampGenState;

/// All editor tool state.
#[derive(Debug, Default, PartialEq)]
pub struct Tools {
    /// Active tool.
    pub active: ToolKind,
    /// Selection state.
    pub select: SelectState,
    /// Placement state.
    pub place: PlaceState,
    /// Spline path state.
    pub spline_path: SplinePathState,
    /// Ramp generator state.
    pub ramp_gen: RampGenState,
}

impl Tools {
    /// Activates a tool.
    pub fn activate(&mut self, kind: ToolKind) {
        self.active = kind;
    }
}

impl Default for ToolKind {
    fn default() -> Self {
        Self::Select
    }
}
