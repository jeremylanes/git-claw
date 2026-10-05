//! Deterministic Slot ID allocation.

use std::collections::BTreeSet;

/// Allocates the lowest available positive integer Slot ID (1..N),
/// recycling any freed slots/gaps.
pub fn allocate_lowest_slot<I>(occupied_slots: I) -> u32
where
    I: IntoIterator<Item = u32>,
{
    let set: BTreeSet<u32> = occupied_slots.into_iter().collect();
    let mut candidate: u32 = 1;
    while set.contains(&candidate) {
        candidate = candidate.checked_add(1).expect("slot ID overflow");
    }
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_empty() {
        assert_eq!(allocate_lowest_slot(vec![]), 1);
    }

    #[test]
    fn test_allocate_contiguous() {
        assert_eq!(allocate_lowest_slot(vec![1, 2, 3]), 4);
    }

    #[test]
    fn test_allocate_gap_recycling() {
        assert_eq!(allocate_lowest_slot(vec![1, 3, 4]), 2);
    }

    #[test]
    fn test_allocate_unsorted_with_duplicates() {
        assert_eq!(allocate_lowest_slot(vec![3, 1, 1, 2, 5]), 4);
    }
}
