//! Times at which crates were last updated.

use crate::index::CrateIndex;
use crate::store::Store;
use core::ops::Index;
use jiff::Timestamp;

/// The last update time of every crate, addressed by [`CrateIndex`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct UpdateDates(Store<Timestamp>);

impl UpdateDates {
    /// Creates an empty store.
    pub const fn new() -> Self {
        Self(Store::new())
    }

    /// Creates an empty store with room for `capacity` timestamps.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Store::with_capacity(capacity))
    }

    /// Appends `updated_at` and returns the index it was given.
    ///
    /// # Panics
    ///
    /// Panics if the store already holds [`u32::MAX`] timestamps.
    pub fn push(&mut self, updated_at: Timestamp) -> CrateIndex {
        self.0.push(updated_at)
    }

    /// Returns the timestamp at `index`, or `None` if the index is unoccupied.
    pub fn get(&self, index: CrateIndex) -> Option<Timestamp> {
        self.0.get(index).copied()
    }

    /// Returns the number of timestamps held.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the store holds no timestamps.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over every index and its timestamp, in index order.
    pub fn iter(&self) -> impl Iterator<Item = (CrateIndex, Timestamp)> + '_ {
        self.0
            .iter()
            .map(|(index, updated_at)| (index, *updated_at))
    }
}

impl Index<CrateIndex> for UpdateDates {
    type Output = Timestamp;

    /// # Panics
    ///
    /// Panics if the index is unoccupied.
    fn index(&self, index: CrateIndex) -> &Timestamp {
        self.0.get(index).expect("no timestamp at this index")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> Timestamp {
        text.parse().expect("timestamp parses")
    }

    fn update_dates_of(entries: &[&str]) -> UpdateDates {
        let mut update_dates = UpdateDates::new();
        for entry in entries {
            update_dates.push(at(entry));
        }
        update_dates
    }

    fn index(position: usize) -> CrateIndex {
        CrateIndex::from_index(position).expect("position fits in an index")
    }

    #[test]
    fn timestamps_are_retrieved_by_index() {
        let update_dates = update_dates_of(&["2020-01-01T00:00:00Z", "2021-06-05T08:00:00Z"]);
        assert_eq!(update_dates.get(index(1)), Some(at("2021-06-05T08:00:00Z")));
    }

    #[test]
    fn indexing_yields_the_pushed_timestamp() {
        let update_dates = update_dates_of(&["2020-01-01T00:00:00Z"]);
        assert_eq!(update_dates[index(0)], at("2020-01-01T00:00:00Z"));
    }

    #[test]
    fn unoccupied_indices_hold_nothing() {
        let update_dates = update_dates_of(&["2020-01-01T00:00:00Z"]);
        assert_eq!(update_dates.get(index(2)), None);
    }

    #[test]
    fn pushing_mints_consecutive_indices() {
        let mut update_dates = UpdateDates::new();
        assert_eq!(update_dates.push(at("2020-01-01T00:00:00Z")), index(0));
        assert_eq!(update_dates.push(at("2021-06-05T08:00:00Z")), index(1));
    }

    #[test]
    fn iteration_pairs_every_timestamp_with_its_index() {
        let update_dates = update_dates_of(&["2020-01-01T00:00:00Z", "2021-06-05T08:00:00Z"]);
        let pairs: Vec<_> = update_dates.iter().collect();
        assert_eq!(
            pairs,
            vec![
                (index(0), at("2020-01-01T00:00:00Z")),
                (index(1), at("2021-06-05T08:00:00Z")),
            ]
        );
    }

    #[test]
    fn a_new_store_is_empty() {
        let update_dates = UpdateDates::new();
        assert!(update_dates.is_empty());
        assert_eq!(update_dates.len(), 0);
    }
}
