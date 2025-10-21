use std::time::Duration;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
/// Combo Extension, for use with kitcar::combos::ComboList
pub struct ComboExt {
    /// Lap count
    pub laps: Option<u8>,
    /// What time do we need to hit?
    #[serde(with = "humantime_serde")]
    pub target_time: Duration,
    /// Cooldown - restart after
    #[serde(with = "humantime_serde")]
    pub restart_after: Duration,
    /// Number of rounds for this combo
    pub rounds: u32,
}
