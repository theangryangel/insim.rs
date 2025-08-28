//! Lease
use std::cell::RefCell;

/// A lease manager that manages `u8` identifiers.
/// Useful for button ids, request ids, etc.
#[derive(Debug)]
pub struct LeaseIdManager {
    leased: RefCell<[u128; 2]>,
    start: u8,
    end: u8,
}

impl LeaseIdManager {
    /// Creates a new `LeaserIdManager` instance.
    pub fn new(start: u8, end: u8) -> LeaseIdManager {
        LeaseIdManager {
            // Initialize the RefCell with a new, empty bitmask.
            leased: RefCell::new([0u128; 2]),
            start,
            end,
        }
    }

    /// Leases the first available identifier between 1 and 254.
    pub fn lease(&self) -> Option<u8> {
        // Get a mutable borrow of the internal mask. This will panic if
        // another mutable borrow already exists.
        let mut mask = self.leased.borrow_mut();

        // Iterate through each u128 in the mask.
        for (mask_index, &m) in mask.iter().enumerate() {
            // Find the index of the first available bit
            if m != u128::MAX {
                let bit_pos = m.trailing_ones();
                let id = ((mask_index * 128) as u32 + (bit_pos as u32) + 1) as u8;

                if id >= self.start && id <= self.end {
                    // Set the corresponding bit to indicate the ID is leased.
                    mask[mask_index] |= 1 << bit_pos;
                    return Some(id);
                }
            }
        }

        // If no available ID was found after checking all masks, return None.
        None
    }

    /// Forcefully lease an id
    pub fn lease_unchecked(&self, id: u8) -> Option<u8> {
        if id >= self.start && id <= self.end {
            // Get a mutable borrow of the internal mask.
            let mut mask = self.leased.borrow_mut();

            // Calculate the index of the u128 in the mask and the bit position
            // within that u128.
            let mask_index = ((id - 1) / 128) as usize;
            let bit_pos = (id - 1) % 128;
            mask[mask_index] |= 1 << bit_pos;
            return Some(id);
        }
        None
    }

    /// Check if an identifier is available
    pub fn available(&self, id: u8) -> bool {
        if id >= self.start && id <= self.end {
            // Get a mutable borrow of the internal mask.
            let mask = self.leased.borrow();

            // Calculate the index of the u128 in the mask and the bit position
            // within that u128.
            let mask_index = ((id - 1) / 128) as usize;
            let bit_pos = (id - 1) % 128;

            // Ensure the mask_index is within bounds.
            if mask_index >= mask.len() {
                return false;
            }

            // Check if the bit for this ID is currently set (leased).
            let is_leased = (mask[mask_index] & (1 << bit_pos)) != 0;
            return !is_leased;
        }
        false
    }

    /// Makes a previously leased identifier available for reuse.
    pub fn free(&self, id: u8) -> bool {
        if id >= self.start && id <= self.end {
            // Get a mutable borrow of the internal mask.
            let mut mask = self.leased.borrow_mut();

            // Calculate the index of the u128 in the mask and the bit position
            // within that u128.
            let mask_index = ((id - 1) / 128) as usize;
            let bit_pos = (id - 1) % 128;

            // Ensure the mask_index is within bounds.
            if mask_index >= mask.len() {
                return false;
            }

            // Check if the bit for this ID is currently set (leased).
            let is_leased = (mask[mask_index] & (1 << bit_pos)) != 0;

            if is_leased {
                // Clear the bit to make the ID available again.
                mask[mask_index] &= !(1 << bit_pos);
                return true;
            }
        }
        // The ID was invalid or not leased.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lease() {
        let leaser = LeaseIdManager::new(1, 239);

        let id1 = leaser.lease().unwrap();
        let id2 = leaser.lease().unwrap();

        let id3 = leaser.lease_unchecked(3).unwrap();
        assert_eq!(id3, 3);
        assert_eq!(leaser.available(3), false);

        assert_eq!(id1, 1);
        assert_eq!(leaser.available(id1), false);
        assert_eq!(id2, 2);
        assert_eq!(leaser.available(id2), false);
        assert_eq!(leaser.available(3), false);
        assert_eq!(leaser.available(4), true);
    }

    #[test]
    fn test_free() {
        let leaser = LeaseIdManager::new(1, 239);
        let id = leaser.lease().unwrap();

        assert_eq!(leaser.free(id), true);

        let next_id = leaser.lease().unwrap();
        assert_eq!(next_id, id);
    }

    #[test]
    fn test_exhaustion() {
        let leaser = LeaseIdManager::new(1, 254);

        for i in 1..=254 {
            let id = leaser.lease();
            assert!(id.is_some());
            assert_eq!(id.unwrap(), i as u8);
        }

        let id = leaser.lease();
        assert!(id.is_none());
    }
}
