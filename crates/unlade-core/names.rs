//! Crate names.

use crate::index::CrateIndex;
use crate::store::Store;
use core::ops::Index;

/// The name of every crate, addressed by [`CrateIndex`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Names(Store<String>);

impl Names {
    /// Creates an empty store.
    pub const fn new() -> Self {
        Self(Store::new())
    }

    /// Creates an empty store with room for `capacity` names.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Store::with_capacity(capacity))
    }

    /// Appends `name` and returns the index it was given.
    ///
    /// # Panics
    ///
    /// Panics if the store already holds [`u32::MAX`] names.
    pub fn push(&mut self, name: impl Into<String>) -> CrateIndex {
        self.0.push(name.into())
    }

    /// Returns the name at `index`, or `None` if the index is unoccupied.
    pub fn get(&self, index: CrateIndex) -> Option<&str> {
        self.0.get(index).map(String::as_str)
    }

    /// Returns the number of names held.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the store holds no names.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over every index and its name, in index order.
    pub fn iter(&self) -> impl Iterator<Item = (CrateIndex, &str)> + '_ {
        self.0.iter().map(|(index, name)| (index, name.as_str()))
    }
}

impl Index<CrateIndex> for Names {
    type Output = str;

    /// # Panics
    ///
    /// Panics if the index is unoccupied.
    fn index(&self, index: CrateIndex) -> &str {
        self.get(index).expect("no name at this index")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names_of(entries: &[&str]) -> Names {
        let mut names = Names::new();
        for entry in entries {
            names.push(*entry);
        }
        names
    }

    fn index(position: usize) -> CrateIndex {
        CrateIndex::from_index(position).expect("position fits in an index")
    }

    #[test]
    fn names_are_retrieved_by_index() {
        let names = names_of(&["serde", "tokio"]);
        assert_eq!(names.get(index(1)), Some("tokio"));
    }

    #[test]
    fn indexing_yields_the_pushed_name() {
        let names = names_of(&["serde", "tokio"]);
        assert_eq!(&names[index(0)], "serde");
    }

    #[test]
    fn unoccupied_indices_hold_nothing() {
        let names = names_of(&["serde"]);
        assert_eq!(names.get(index(3)), None);
    }

    #[test]
    fn pushing_mints_consecutive_indices() {
        let mut names = Names::new();
        assert_eq!(names.push("serde"), index(0));
        assert_eq!(names.push("tokio"), index(1));
    }

    #[test]
    fn iteration_pairs_every_name_with_its_index() {
        let names = names_of(&["serde", "tokio"]);
        let pairs: Vec<_> = names.iter().collect();
        assert_eq!(pairs, vec![(index(0), "serde"), (index(1), "tokio")]);
    }

    #[test]
    fn a_new_store_is_empty() {
        let names = Names::new();
        assert!(names.is_empty());
        assert_eq!(names.len(), 0);
    }
}
