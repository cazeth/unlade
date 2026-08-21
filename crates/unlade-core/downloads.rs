//! How many times each crate has been downloaded.

use crate::index::CrateIndex;
use crate::store::Store;
use core::ops::Index;

/// The download count of every crate, addressed by [`CrateIndex`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Downloads(Store<u64>);

impl Downloads {
    /// Creates an empty store.
    pub const fn new() -> Self {
        Self(Store::new())
    }

    /// Creates an empty store with room for `capacity` crates.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Store::with_capacity(capacity))
    }

    /// Appends `downloads` and returns the index it was given.
    ///
    /// # Panics
    ///
    /// Panics if the store already holds [`u32::MAX`] counts.
    pub fn push(&mut self, downloads: u64) -> CrateIndex {
        self.0.push(downloads)
    }

    /// Returns the count at `index`, or `None` if the index is unoccupied.
    pub fn get(&self, index: CrateIndex) -> Option<u64> {
        self.0.get(index).copied()
    }

    /// Returns the number of crates counted.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the store counts no crates.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over every index and its count, in index order.
    pub fn iter(&self) -> impl Iterator<Item = (CrateIndex, u64)> + '_ {
        self.0
            .iter()
            .map(|(index, dependents)| (index, *dependents))
    }
}

impl Index<CrateIndex> for Downloads {
    type Output = u64;

    /// # Panics
    ///
    /// Panics if the index is unoccupied.
    fn index(&self, index: CrateIndex) -> &u64 {
        self.0.get(index).expect("no download count at this index")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts_of(values: &[u64]) -> Downloads {
        let mut counts = Downloads::new();
        for value in values {
            counts.push(*value);
        }
        counts
    }

    fn index(position: usize) -> CrateIndex {
        CrateIndex::from_index(position).expect("position fits in an index")
    }

    #[test]
    fn counts_are_retrieved_by_index() {
        let counts = counts_of(&[0, 4_242]);
        assert_eq!(counts.get(index(1)), Some(4_242));
    }

    #[test]
    fn indexing_yields_the_pushed_count() {
        let counts = counts_of(&[7]);
        assert_eq!(counts[index(0)], 7);
    }

    #[test]
    fn unoccupied_indices_hold_nothing() {
        let counts = counts_of(&[7]);
        assert_eq!(counts.get(index(4)), None);
    }

    #[test]
    fn pushing_mints_consecutive_indices() {
        let mut counts = Downloads::new();
        assert_eq!(counts.push(1), index(0));
        assert_eq!(counts.push(2), index(1));
    }

    #[test]
    fn iteration_pairs_every_count_with_its_index() {
        let counts = counts_of(&[3, 5]);
        let pairs: Vec<_> = counts.iter().collect();
        assert_eq!(pairs, vec![(index(0), 3), (index(1), 5)]);
    }

    #[test]
    fn a_new_store_is_empty() {
        let counts = Downloads::new();
        assert!(counts.is_empty());
        assert_eq!(counts.len(), 0);
    }
}
