//! Bidirectional mapping between crates.io identifiers and dense indices.

use crate::CrateId;
use crate::CrateIndex;
use std::collections::HashMap;

/// Maps each sparse crates.io [`CrateId`] to its dense [`CrateIndex`].
///
/// Calling [`get_or_insert`](Self::get_or_insert) with a known identifier
/// returns its existing index. A new identifier receives the next index, so
/// indices remain dense from zero.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrateIdMap {
    ids: Vec<CrateId>,
    indices: HashMap<CrateId, CrateIndex>,
}

impl CrateIdMap {
    /// Creates an empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the existing index for `id`, or assigns the next dense index.
    ///
    /// # Panics
    ///
    /// Panics if the map already contains `u32::MAX + 1` identifiers.
    pub fn get_or_insert(&mut self, id: CrateId) -> CrateIndex {
        if let Some(index) = self.get(id) {
            return index;
        }

        let index = CrateIndex::from_index(self.ids.len())
            .expect("a crate ID map holds at most u32::MAX identifiers");
        self.ids.push(id);
        self.indices.insert(id, index);
        index
    }

    /// Returns the index assigned to `id`, if the identifier is known.
    pub fn get(&self, id: CrateId) -> Option<CrateIndex> {
        self.indices.get(&id).copied()
    }

    /// Returns the identifier assigned to `index`, if the index is occupied.
    pub fn id(&self, index: CrateIndex) -> Option<CrateId> {
        self.ids.get(index.index()).copied()
    }

    /// Returns the number of identities held.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Returns whether the map holds no identities.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Iterates over every index and identifier in index order.
    ///
    /// # Panics
    ///
    /// Panics if the map contains more identifiers than [`CrateIndex`] can
    /// represent.
    pub fn iter(&self) -> impl Iterator<Item = (CrateIndex, CrateId)> + '_ {
        self.ids.iter().copied().enumerate().map(|(position, id)| {
            let index = CrateIndex::from_index(position).expect("ID position fits in an index");
            (index, id)
        })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for CrateIdMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&self.ids, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CrateIdMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ids = <Vec<CrateId> as serde::Deserialize>::deserialize(deserializer)?;
        let mut indices = HashMap::with_capacity(ids.len());

        for (position, id) in ids.iter().copied().enumerate() {
            let index = CrateIndex::from_index(position).ok_or_else(|| {
                <D::Error as serde::de::Error>::custom(
                    "a crate ID map cannot hold more than 2^32 identifiers",
                )
            })?;

            if indices.insert(id, index).is_some() {
                return Err(<D::Error as serde::de::Error>::custom(format_args!(
                    "duplicate crate ID {}",
                    id.get(),
                )));
            }
        }

        Ok(Self { ids, indices })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserting_an_id_twice_returns_its_existing_index() {
        let mut ids = CrateIdMap::new();
        let first = ids.get_or_insert(CrateId::new(42));
        let other = ids.get_or_insert(CrateId::new(7));
        let again = ids.get_or_insert(CrateId::new(42));

        assert_eq!(again, first);
        assert_ne!(other, first);
        assert_eq!(ids.id(first), Some(CrateId::new(42)));
        assert_eq!(ids.len(), 2);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialization_preserves_both_directions() {
        let mut ids = CrateIdMap::new();
        let forty_two = ids.get_or_insert(CrateId::new(42));
        let seven = ids.get_or_insert(CrateId::new(7));

        let json = serde_json::to_string(&ids).expect("map serializes");
        let restored: CrateIdMap = serde_json::from_str(&json).expect("map deserializes");

        assert_eq!(json, "[42,7]");
        assert_eq!(restored.id(forty_two), Some(CrateId::new(42)));
        assert_eq!(restored.get(CrateId::new(7)), Some(seven));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialization_rejects_duplicate_ids() {
        let error = serde_json::from_str::<CrateIdMap>("[42,42]")
            .expect_err("duplicate IDs violate the map invariant");

        assert!(error.to_string().contains("duplicate crate ID 42"));
    }
}
