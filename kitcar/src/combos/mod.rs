//! Track, vehicle combos
use insim::core::{track::Track, vehicle::Vehicle};
use rand::prelude::*;

/// Vehicle with Restriction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct VehicleWithRestrictions {
    vehicle: Vehicle,
    h_mass: Option<u8>,
    h_tres: Option<u8>,
}

impl VehicleWithRestrictions {
    /// Create a new vehicle with restrictions
    pub fn new(vehicle: Vehicle, h_mass: Option<u8>, h_tres: Option<u8>) -> Self {
        Self {
            vehicle,
            h_mass,
            h_tres,
        }
    }

    /// Get the vehicle
    pub fn vehicle(&self) -> &Vehicle {
        &self.vehicle
    }

    /// Get the mass handicap
    pub fn h_mass(&self) -> Option<u8> {
        self.h_mass
    }

    /// Get the intake restriction
    pub fn h_tres(&self) -> Option<u8> {
        self.h_tres
    }
}

/// A combo. The generic S may be anything. The extensions are annotated with serde flatten,
#[derive(Debug, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(bound(
    deserialize = "S: serde::Deserialize<'de>",
    serialize = "S: serde::Serialize"
))]
pub struct Combo<S> {
    track: Track,
    layout: Option<String>,
    allowed_vehicles: Vec<VehicleWithRestrictions>,
    #[serde(flatten)]
    extensions: S,
}

impl<S> Combo<S> {
    /// Create a new combo
    pub fn new(
        track: Track,
        layout: Option<String>,
        allowed_vehicles: Vec<VehicleWithRestrictions>,
        extra: S,
    ) -> Self {
        Self {
            track,
            layout,
            allowed_vehicles,
            extensions: extra,
        }
    }

    /// Get the track
    pub fn track(&self) -> &Track {
        &self.track
    }

    /// Get the layout
    pub fn layout(&self) -> Option<&str> {
        self.layout.as_deref()
    }

    /// Get the allowed vehicles
    pub fn allowed_vehicles(&self) -> &[VehicleWithRestrictions] {
        &self.allowed_vehicles
    }

    /// Extra info / user defined extensions
    pub fn extensions(&self) -> &S {
        &self.extensions
    }
}

/// A list of combos
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(bound(
    deserialize = "S: serde::Deserialize<'de>",
    serialize = "S: serde::Serialize"
))]
#[serde(transparent)]
pub struct ComboList<S> {
    inner: Vec<Combo<S>>,
    #[serde(skip)]
    current: usize,
}

impl<S> ComboList<S> {
    /// Create a new combo list
    pub fn new(combos: Vec<Combo<S>>) -> Self {
        Self {
            inner: combos,
            current: 0,
        }
    }

    /// Next in the combo list
    pub fn advance(&mut self) -> Option<&Combo<S>> {
        if self.inner.is_empty() {
            return None;
        }
        self.current = (self.current + 1) % self.inner.len();
        Some(&self.inner[self.current])
    }

    /// Current in the combo list
    pub fn current(&self) -> Option<&Combo<S>> {
        if self.inner.is_empty() {
            None
        } else {
            Some(&self.inner[self.current])
        }
    }

    /// Shuffle the combos
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.inner.shuffle(&mut rng);
        self.current = 0;
    }

    /// Reset to the first combo
    pub fn reset(&mut self) {
        self.current = 0;
    }

    /// Get the number of combos
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns a random combo from the collection
    pub fn random(&self) -> Option<&Combo<S>> {
        self.inner.choose(&mut rand::rng())
    }

    /// Iter
    pub fn iter(&self) -> std::slice::Iter<'_, Combo<S>> {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_combo(track: Track) -> Combo<()> {
        Combo::new(track, Some("layout1".to_string()), vec![], ())
    }

    #[test]
    fn test_combo_list_creation() {
        let combos = vec![create_test_combo(Track::Bl1), create_test_combo(Track::Bl2)];
        let list = ComboList::new(combos);

        assert_eq!(list.len(), 2);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_combo_list_empty() {
        let list = ComboList::<()>::new(vec![]);
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
        assert_eq!(list.current(), None);
    }

    #[test]
    fn test_current_combo() {
        let combos = vec![create_test_combo(Track::Bl1), create_test_combo(Track::Bl2)];
        let list = ComboList::new(combos);

        assert!(list.current().is_some());
    }

    #[test]
    fn test_next_combo() {
        let combos = vec![
            create_test_combo(Track::Bl1),
            create_test_combo(Track::Bl2),
            create_test_combo(Track::Bl3),
        ];
        let mut list = ComboList::new(combos);

        // First call to next moves to index 1
        let first_next = list.advance();
        assert!(first_next.is_some());

        // Second call moves to index 2
        let second_next = list.advance();
        assert!(second_next.is_some());

        // Third call wraps around to index 0
        let third_next = list.advance();
        assert!(third_next.is_some());
    }

    #[test]
    fn test_next_empty_list() {
        let mut list = ComboList::<()>::new(vec![]);
        assert_eq!(list.advance(), None);
    }

    #[test]
    fn test_next_wraps_around() {
        let combos = vec![create_test_combo(Track::Bl1), create_test_combo(Track::Bl2)];
        let mut list = ComboList::new(combos);

        assert!(list.advance().is_some()); // index 1
        let wrapped = list.advance(); // should wrap to index 0

        assert!(wrapped.is_some());
        assert_eq!(list.current().is_some(), true);
    }

    #[test]
    fn test_reset() {
        let combos = vec![
            create_test_combo(Track::Bl1),
            create_test_combo(Track::Bl2),
            create_test_combo(Track::Bl3),
        ];
        let mut list = ComboList::new(combos);

        assert!(list.advance().is_some());
        assert!(list.advance().is_some());
        assert_eq!(list.current, 2);

        list.reset();
        assert_eq!(list.current, 0);
    }

    #[test]
    fn test_shuffle_maintains_count() {
        let combos = vec![
            create_test_combo(Track::Bl1),
            create_test_combo(Track::Bl2),
            create_test_combo(Track::Bl3),
        ];
        let mut list = ComboList::new(combos);
        let original_len = list.len();

        list.shuffle();
        assert_eq!(list.len(), original_len);
    }

    #[test]
    fn test_shuffle_resets_position() {
        let combos = vec![create_test_combo(Track::Bl1), create_test_combo(Track::Bl2)];
        let mut list = ComboList::new(combos);

        assert!(list.advance().is_some());
        assert_ne!(list.current, 0);

        list.shuffle();
        assert_eq!(list.current, 0);
    }

    #[test]
    fn test_vehicle_with_restrictions() {
        let vehicle = VehicleWithRestrictions::new(Vehicle::Xfg, Some(50), Some(30));

        assert_eq!(vehicle.h_mass(), Some(50));
        assert_eq!(vehicle.h_tres(), Some(30));
    }

    #[test]
    fn test_combo_accessors() {
        let combo = create_test_combo(Track::Bl1);
        assert_eq!(combo.layout(), Some("layout1"));
        assert_eq!(combo.allowed_vehicles().len(), 0);
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
    struct ComboExtensions {
        lap_count: usize,
        difficulty: String,
        rewards: Vec<String>,
    }

    #[test]
    fn test_flatten_with_typed_extensions() {
        let yaml = r#"
track: BL1
layout: null
allowed_vehicles: []
lap_count: 1
difficulty: hard
rewards:
  - gold
  - experience
"#;

        let combo: Combo<ComboExtensions> = serde_norway::from_str(yaml).unwrap();

        assert_eq!(combo.track, Track::Bl1);
        assert_eq!(combo.extensions.lap_count, 1);
        assert_eq!(combo.extensions.difficulty, "hard");
        assert_eq!(combo.extensions.rewards, vec!["gold", "experience"]);
    }

    #[test]
    fn test_flatten_with_unit_type() {
        let yaml = r#"
track: BL1
layout: null
allowed_vehicles: []
lap_count: 1
extra1: thing
extra2: thing2
"#;

        // Using () ignores all unknown fields
        let combo: Combo<()> = serde_norway::from_str(yaml).unwrap();

        assert_eq!(combo.track, Track::Bl1);
        assert_eq!(combo.extensions, ());
    }
}
