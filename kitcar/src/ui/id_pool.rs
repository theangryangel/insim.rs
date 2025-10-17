//! Click ID Pool Manager - handles allocation and deallocation of unique IDs

use fixedbitset::FixedBitSet;
use insim::identifiers::ClickId;

/// Helper alias for a Btn ClickId Pool
pub type ClickIdPool = IdPool<1, 239>;

#[derive(Debug)]
/// ClickId Pool - handles allocation and deallocation of unique IDs for buttons
pub struct IdPool<const MIN: u8, const MAX: u8> {
    // Bit set where 1 = available, 0 = allocated
    available_ids: FixedBitSet,
}

impl<const MIN: u8, const MAX: u8> IdPool<MIN, MAX> {
    /// New!
    pub fn new() -> Self {
        let mut available_ids = FixedBitSet::with_capacity(MAX.saturating_add(1) as usize);
        // Set bits MIN-MAX to indicate they are available (skip 0)
        // sadly cannot use set_range since we want it inclusive
        (MIN..=MAX).into_iter().for_each(|i| {
            available_ids.set(i as usize, true);
        });

        Self { available_ids }
    }

    /// Lease/allocate a click ID by finding the first available slot.
    pub fn lease(&mut self) -> Option<ClickId> {
        // Find the first available ID by iterating from the beginning.
        if let Some(id) = self.available_ids.ones().rev().next() {
            let click_id = id as u8;
            self.available_ids.set(id, false);
            Some(click_id.into())
        } else {
            None // Pool exhausted
        }
    }

    /// Release/deallocate one or more ClickIds
    pub fn release<'a, I>(&mut self, click_ids: &'a I)
    where
        &'a I: IntoIterator<Item = &'a ClickId>,
    {
        for click_id in click_ids.into_iter() {
            if (MIN..=MAX).contains(&click_id) {
                self.available_ids.set(click_id.0 as usize, true);
            }
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let total_capacity = MAX as usize - MIN as usize + 1;
        let available = self.available_ids.count_ones(..);
        let allocated = total_capacity - available;
        (total_capacity, available, allocated)
    }

    /// Get available count
    pub fn available_count(&self) -> usize {
        self.available_ids.count_ones(..)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool_creation() {
        let pool: IdPool<1, 10> = IdPool::new();
        let (total, available, allocated) = pool.stats();

        assert_eq!(total, 10); // 1-10 inclusive
        assert_eq!(available, 10);
        assert_eq!(allocated, 0);
    }

    #[test]
    fn test_new_pool_with_single_id() {
        let pool: IdPool<5, 5> = IdPool::new();
        let (total, available, allocated) = pool.stats();

        assert_eq!(total, 1);
        assert_eq!(available, 1);
        assert_eq!(allocated, 0);
    }

    #[test]
    fn test_new_pool_with_zero_min() {
        let pool: IdPool<0, 3> = IdPool::new();
        let (total, available, allocated) = pool.stats();

        assert_eq!(total, 4); // 0-3 inclusive
        assert_eq!(available, 4);
        assert_eq!(allocated, 0);
    }

    #[test]
    fn test_lease_single_id() {
        let mut pool: IdPool<1, 10> = IdPool::new();

        let id = pool.lease();
        assert!(id.is_some());
        assert_eq!(id.unwrap().0, 1); // Should get the first available (min)

        let (total, available, allocated) = pool.stats();
        assert_eq!(total, 10);
        assert_eq!(available, 9);
        assert_eq!(allocated, 1);
    }

    #[test]
    fn test_lease_multiple_ids_sequential() {
        let mut pool: IdPool<5, 8> = IdPool::new();

        let id1 = pool.lease().unwrap();
        let id2 = pool.lease().unwrap();
        let id3 = pool.lease().unwrap();

        assert_eq!(id1.0, 5);
        assert_eq!(id2.0, 6);
        assert_eq!(id3.0, 7);

        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 1);
        assert_eq!(allocated, 3);
    }

    #[test]
    fn test_lease_until_exhausted() {
        let mut pool: IdPool<1, 3> = IdPool::new();

        // Lease all available IDs
        let id1 = pool.lease();
        let id2 = pool.lease();
        let id3 = pool.lease();

        assert_eq!(id1, Some(ClickId(1)));
        assert_eq!(id2, Some(ClickId(2)));
        assert_eq!(id3, Some(ClickId(3)));

        // Pool should be exhausted
        let id4 = pool.lease();
        assert_eq!(id4, None);

        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 0);
        assert_eq!(allocated, 3);
    }

    #[test]
    fn test_release_single_id() {
        let mut pool: IdPool<1, 5> = IdPool::new();

        let id = pool.lease().unwrap();
        assert_eq!(id.0, 1);

        pool.release(&vec![id]);

        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 5);
        assert_eq!(allocated, 0);

        // Should be able to lease the same ID again
        let new_id = pool.lease().unwrap();
        assert_eq!(new_id.0, 1);
    }

    #[test]
    fn test_release_multiple_ids() {
        let mut pool: IdPool<1, 5> = IdPool::new();

        let id1 = pool.lease().unwrap(); // 1
        let _id2 = pool.lease().unwrap(); // 2
        let id3 = pool.lease().unwrap(); // 3

        pool.release(&vec![id1, id3]); // Release 1 and 3, keep 2

        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 4);
        assert_eq!(allocated, 1);

        // Next lease should get ID 1 (first available)
        let next_id = pool.lease().unwrap();
        assert_eq!(next_id.0, 1);
    }

    #[test]
    fn test_release_out_of_range_ids() {
        let mut pool: IdPool<5, 10> = IdPool::new();

        let id = pool.lease().unwrap(); // Should be 5
        assert_eq!(id.0, 5);

        // Try to release IDs outside the valid range
        pool.release(&vec![ClickId(0), ClickId(4), ClickId(11), ClickId(255)]);

        // Stats shouldn't change since none were in range
        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 5); // 6, 7, 8, 9, 10
        assert_eq!(allocated, 1); // 5

        // Release the valid ID
        pool.release(&[id]);
        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 6);
        assert_eq!(allocated, 0);
    }

    #[test]
    fn test_release_already_available_ids() {
        let mut pool: IdPool<1, 3> = IdPool::new();

        let _ = pool.lease().unwrap(); // 1

        // Release ID 2 which was never allocated
        pool.release(&[ClickId(2)]);

        // Stats shouldn't change since ID 2 was already available
        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 2); // 2, 3
        assert_eq!(allocated, 1); // 1
    }

    #[test]
    fn test_release_with_different_iterables() {
        let mut pool: IdPool<1, 5> = IdPool::new();

        let id1 = pool.lease().unwrap();
        let id2 = pool.lease().unwrap();
        let id3 = pool.lease().unwrap();

        // Release using Vec
        pool.release(&vec![id1]);

        // Release using array
        pool.release(&[id2]);

        // Release using slice
        let ids = [id3];
        pool.release(&ids);

        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 5);
        assert_eq!(allocated, 0);
    }

    #[test]
    fn test_stats_consistency() {
        let mut pool: IdPool<10, 20> = IdPool::new();

        let (total, available, allocated) = pool.stats();
        assert_eq!(total, available + allocated);
        assert_eq!(total, 11); // 10-20 inclusive

        // Lease some IDs
        let _id1 = pool.lease();
        let _id2 = pool.lease();
        let _id3 = pool.lease();

        let (total2, available2, allocated2) = pool.stats();
        assert_eq!(total2, available2 + allocated2);
        assert_eq!(total2, total); // Total should remain constant
        assert_eq!(allocated2, 3);
        assert_eq!(available2, 8);
    }

    #[test]
    fn test_available_count() {
        let mut pool: IdPool<1, 5> = IdPool::new();

        assert_eq!(pool.available_count(), 5);

        let _id1 = pool.lease();
        assert_eq!(pool.available_count(), 4);

        let _id2 = pool.lease();
        assert_eq!(pool.available_count(), 3);

        pool.release(&vec![_id1.unwrap()]);
        assert_eq!(pool.available_count(), 4);
    }

    #[test]
    fn test_lease_after_partial_release() {
        let mut pool: IdPool<1, 4> = IdPool::new();

        // Lease all IDs
        let _id1 = pool.lease().unwrap(); // 1
        let id2 = pool.lease().unwrap(); // 2
        let id3 = pool.lease().unwrap(); // 3
        let _id4 = pool.lease().unwrap(); // 4

        // Pool exhausted
        assert_eq!(pool.lease(), None);

        // Release middle IDs
        pool.release(&vec![id2, id3]);

        // Should be able to lease again, getting the lowest available
        let new_id = pool.lease().unwrap();
        assert_eq!(new_id.0, 2);

        let another_id = pool.lease().unwrap();
        assert_eq!(another_id.0, 3);

        // Now exhausted again
        assert_eq!(pool.lease(), None);
    }

    #[test]
    fn test_empty_release() {
        let mut pool: IdPool<1, 3> = IdPool::new();
        let _id = pool.lease().unwrap();

        // Release empty collection
        let empty_vec: Vec<ClickId> = vec![];
        pool.release(&empty_vec);

        // Stats should be unchanged
        let (_, available, allocated) = pool.stats();
        assert_eq!(available, 2);
        assert_eq!(allocated, 1);
    }
}
