use crate::index::CrateIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Store<T> {
    values: Vec<T>,
}

impl<T> Store<T> {
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, value: T) -> CrateIndex {
        let index = CrateIndex::from_index(self.values.len())
            .expect("a store holds at most u32::MAX values");
        self.values.push(value);
        index
    }

    pub fn get(&self, index: CrateIndex) -> Option<&T> {
        self.values.get(index.index())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (CrateIndex, &T)> + '_ {
        self.values.iter().enumerate().map(|(position, value)| {
            let index = CrateIndex::from_index(position).expect("store position fits in an index");
            (index, value)
        })
    }
}

impl<T> Default for Store<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_of(values: &[u8]) -> Store<u8> {
        let mut store = Store::new();
        for value in values {
            store.push(*value);
        }
        store
    }

    fn index(position: usize) -> CrateIndex {
        CrateIndex::from_index(position).expect("position fits in an index")
    }

    #[test]
    fn pushing_mints_consecutive_indices() {
        let mut store = Store::new();
        assert_eq!(store.push('a'), index(0));
        assert_eq!(store.push('b'), index(1));
    }

    #[test]
    fn values_are_retrieved_by_index() {
        let store = store_of(&[7, 8, 9]);
        assert_eq!(store.get(index(2)), Some(&9));
    }

    #[test]
    fn indices_past_the_end_hold_nothing() {
        let store = store_of(&[7]);
        assert_eq!(store.get(index(1)), None);
    }

    #[test]
    fn iteration_pairs_every_value_with_its_index() {
        let store = store_of(&[7, 8]);
        let pairs: Vec<_> = store.iter().collect();
        assert_eq!(pairs, vec![(index(0), &7), (index(1), &8)]);
    }

    #[test]
    fn an_empty_store_reports_no_values() {
        let store = store_of(&[]);
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }
}
