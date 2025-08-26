use insim::core::vehicle::Vehicle;

#[derive(Debug, serde::Deserialize, Clone)]
/// Combo
pub struct Combo {
    /// Name
    pub name: String,
    /// Track to load
    pub track: String,
    /// Track layout
    pub layout: String,
    /// Lap count
    pub laps: u8,
    /// Valid vehicles
    pub vehicles: Vec<Vehicle>,
}

/// Config
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Insim IName
    pub iname: String,
    /// Server address
    pub addr: String,
    /// admin password
    pub admin: Option<String>,
    /// Combination
    pub combo: Vec<Combo>,
    // tick rate
    pub tick_rate: Option<u64>,
}
