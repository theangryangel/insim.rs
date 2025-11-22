use jiff::Span;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
/// Combo Extension, for use with kitcar::combos::ComboList
pub struct ComboExt {
    /// Name for the combo
    pub name: String,
    /// What time do we need to hit?
    pub target_time: Span,
    /// Cooldown - restart after
    pub restart_after: Span,
    /// Number of rounds for this combo
    pub rounds: u32,
}
