use std::time::Duration;

use insim::core::vehicle::Vehicle;
use rand::seq::{IndexedRandom, SliceRandom};

#[derive(Debug, serde::Deserialize, Clone)]
/// Combo
pub struct Combo {
    /// Name
    pub name: String,
    /// Track to load
    pub track: String,
    /// Track layout
    pub layout: Option<String>,
    /// Lap count
    pub laps: Option<u8>,
    /// What time do we need to hit?
    #[serde(with = "humantime_serde")]
    pub target_time: Duration,
    /// Cooldown - restart after
    #[serde(with = "humantime_serde")]
    pub restart_after: Duration,
    /// Valid vehicles
    pub vehicles: Vec<Vehicle>,
}

#[derive(Debug, Default, Clone, serde::Deserialize)]
#[serde(transparent)]
pub struct ComboCollection {
    inner: Vec<Combo>,
    #[serde(skip)]
    offset: usize,
}

impl ComboCollection {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            offset: 0,
        }
    }

    /// Returns a random combo from the collection
    pub fn random(&self) -> Option<&Combo> {
        self.inner.choose(&mut rand::rng())
    }

    /// Returns the next combo in sequence, wrapping around to the beginning when reaching the end
    pub fn next(&mut self) -> Option<&Combo> {
        if self.inner.is_empty() {
            return None;
        }

        let combo = &self.inner[self.offset];
        self.offset = (self.offset + 1) % self.inner.len();
        Some(combo)
    }

    /// Shuffles the internal collection and resets the offset to 0
    pub fn shuffle(&mut self) {
        self.inner.shuffle(&mut rand::rng());
        self.offset = 0;
    }

    /// Resets the offset to 0 for the next() function
    pub fn reset_offset(&mut self) {
        self.offset = 0;
    }

    /// Returns the current offset position
    pub fn current_offset(&self) -> usize {
        self.offset
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Combo> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Combo> {
        self.inner.iter_mut()
    }

    pub fn push(&mut self, value: Combo) {
        self.inner.push(value);
        // Keep offset valid after push
        if self.offset > self.inner.len() {
            self.offset = 0;
        }
    }

    pub fn pop(&mut self) -> Option<Combo> {
        let result = self.inner.pop();

        // Adjust offset if it's now out of bounds
        if self.inner.is_empty() {
            self.offset = 0;
        } else if self.offset >= self.inner.len() {
            self.offset = self.inner.len() - 1;
        }

        result
    }

    /// Returns the length of the collection
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Combo {
        fn simple(name: &str) -> Self {
            Self {
                name: name.to_owned(),
                track: "test".to_owned(),
                layout: None,
                laps: Some(1),
                vehicles: vec![Vehicle::Xrt, Vehicle::Xrg],
                target_time: Duration::from_secs(20),
                restart_after: Duration::from_secs(10),
            }
        }
    }

    #[test]
    fn test_new_combos_is_empty() {
        let combos = ComboCollection::new();
        assert!(combos.is_empty());
        assert_eq!(combos.len(), 0);
        assert_eq!(combos.current_offset(), 0);
    }

    #[test]
    fn test_push_and_pop() {
        let mut combos = ComboCollection::new();
        let combo1 = Combo::simple("Monaco GP");
        let combo2 = Combo::simple("Spa Endurance");

        combos.push(combo1.clone());
        combos.push(combo2.clone());

        assert_eq!(combos.len(), 2);
        assert!(!combos.is_empty());

        let popped = combos.pop().unwrap();
        assert_eq!(popped.name, combo2.name);
        assert_eq!(popped.track, combo2.track);
        assert_eq!(combos.len(), 1);

        let popped = combos.pop().unwrap();
        assert_eq!(popped.name, combo1.name);
        assert_eq!(combos.len(), 0);
        assert!(combos.is_empty());

        assert!(combos.pop().is_none());
    }

    #[test]
    fn test_next_with_empty_collection() {
        let mut combos = ComboCollection::new();
        assert!(combos.next().is_none());
    }

    #[test]
    fn test_next_cycles_through_items() {
        let mut combos = ComboCollection::new();
        let combo1 = Combo::simple("Monaco GP");
        let combo2 = Combo::simple("Silverstone Sprint");
        let combo3 = Combo::simple("Le Mans 24h");

        combos.push(combo1.clone());
        combos.push(combo2.clone());
        combos.push(combo3.clone());

        // First cycle through
        let first = combos.next().unwrap();
        assert_eq!(first.name, combo1.name);
        assert_eq!(combos.current_offset(), 1);

        let second = combos.next().unwrap();
        assert_eq!(second.name, combo2.name);
        assert_eq!(combos.current_offset(), 2);

        let third = combos.next().unwrap();
        assert_eq!(third.name, combo3.name);
        assert_eq!(combos.current_offset(), 0); // Should wrap around

        // Second cycle
        let first_again = combos.next().unwrap();
        assert_eq!(first_again.name, combo1.name);
        assert_eq!(combos.current_offset(), 1);
    }

    #[test]
    fn test_offset_safety_after_pop() {
        let mut combos = ComboCollection::new();
        combos.push(Combo::simple("Race 1"));
        combos.push(Combo::simple("Race 2"));
        combos.push(Combo::simple("Race 3"));

        // Move offset to last position
        let _ = combos.next().expect("Expected a combo"); // offset = 1
        let _ = combos.next().expect("Expected a combo"); // offset = 2
        let _ = combos.next().expect("Expected a combo"); // offset = 0 (wrapped)
        assert_eq!(combos.current_offset(), 0);
        let _ = combos.next().expect("Expected a combo"); // offset = 1
        let _ = combos.next().expect("Expected a combo"); // offset = 2

        assert_eq!(combos.current_offset(), 2);

        // Pop the last item - offset should adjust
        let _ = combos.pop().expect("Expected a combo");
        assert_eq!(combos.current_offset(), 1); // Should be adjusted to len - 1

        // Pop another item
        let _ = combos.pop().expect("Expected a combo");
        assert_eq!(combos.current_offset(), 0); // Should be adjusted again

        // Pop the last item
        let _ = combos.pop().expect("Expected a combo");
        assert_eq!(combos.current_offset(), 0); // Should be 0 for empty collection
        assert!(combos.next().is_none());
    }

    #[test]
    fn test_offset_safety_after_push() {
        let mut combos = ComboCollection::new();

        // This shouldn't happen in normal usage, but let's test edge case
        // where offset somehow gets out of bounds
        combos.offset = 5; // Manually set invalid offset

        combos.push(Combo::simple("Test Race"));

        // Push should have corrected the invalid offset
        assert_eq!(combos.current_offset(), 0);
    }

    #[test]
    fn test_reset_offset() {
        let mut combos = ComboCollection::new();
        combos.push(Combo::simple("Monza GP"));
        combos.push(Combo::simple("Spa Championship"));

        let _ = combos.next().expect("Expected a combo");
        assert_eq!(combos.current_offset(), 1);

        combos.reset_offset();
        assert_eq!(combos.current_offset(), 0);
    }

    #[test]
    fn test_shuffle_resets_offset() {
        let mut combos = ComboCollection::new();
        for i in 1..=10 {
            combos.push(Combo::simple(&format!("Race {}", i)));
        }

        // Move offset
        let _ = combos.next().expect("Expected a combo");
        let _ = combos.next().expect("Expected a combo");
        assert_eq!(combos.current_offset(), 2);

        // Shuffle should reset offset
        combos.shuffle();
        assert_eq!(combos.current_offset(), 0);

        // Should still be able to get items after shuffle
        assert!(combos.next().is_some());
    }

    #[test]
    fn test_random_returns_some_with_items() {
        let mut combos = ComboCollection::new();
        combos.push(Combo::simple("Only Race"));

        assert!(combos.random().is_some());
    }

    #[test]
    fn test_random_returns_none_when_empty() {
        let combos = ComboCollection::new();
        assert!(combos.random().is_none());
    }

    #[test]
    fn test_iterators() {
        let mut combos = ComboCollection::new();
        let combo1 = Combo::simple("Monaco GP");
        let combo2 = Combo::simple("Silverstone Sprint");

        combos.push(combo1.clone());
        combos.push(combo2.clone());

        // Test immutable iterator
        let collected: Vec<_> = combos.iter().cloned().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].name, combo1.name);
        assert_eq!(collected[1].name, combo2.name);

        // Test mutable iterator
        for combo in combos.iter_mut() {
            combo.name.push_str("_modified");
        }

        assert_eq!(combos.iter().next().unwrap().name, "Monaco GP_modified");
        assert_eq!(
            combos.iter().nth(1).unwrap().name,
            "Silverstone Sprint_modified"
        );
    }

    #[test]
    fn test_multiple_pops_on_empty() {
        let mut combos = ComboCollection::new();
        combos.push(Combo::simple("Single Race"));

        // Pop the only item
        assert!(combos.pop().is_some());
        assert_eq!(combos.current_offset(), 0);

        // Multiple pops on empty should not panic
        assert!(combos.pop().is_none());
        assert!(combos.pop().is_none());
        assert_eq!(combos.current_offset(), 0);
    }

    #[test]
    fn test_next_after_pop_all_items() {
        let mut combos = ComboCollection::new();
        combos.push(Combo::simple("First Race"));
        combos.push(Combo::simple("Second Race"));

        // Use next a few times
        let _ = combos.next().expect("Expected a combo");
        let _ = combos.next().expect("Expected a combo");

        // Pop all items
        let _ = combos.pop().expect("Expected a combo");
        let _ = combos.pop().expect("Expected a combo");

        // next() should return None
        assert!(combos.next().is_none());
        assert_eq!(combos.current_offset(), 0);
    }
}
