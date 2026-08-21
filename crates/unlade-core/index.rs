//! Dense addressing of crates.

/// Position of a crate within the component stores.
///
/// Indices are minted by the stores and are contiguous from zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct CrateIndex(u32);

impl CrateIndex {
    /// Returns the index at `position`, or `None` if it exceeds [`u32::MAX`].
    pub fn from_index(position: usize) -> Option<Self> {
        u32::try_from(position).ok().map(Self)
    }

    /// Returns the position this index refers to.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(position: usize) -> Option<usize> {
        CrateIndex::from_index(position).map(CrateIndex::index)
    }

    #[test]
    fn positions_survive_a_round_trip() {
        assert_eq!(round_trip(0), Some(0));
        assert_eq!(round_trip(4_096), Some(4_096));
    }

    #[test]
    fn positions_beyond_the_representable_range_are_rejected() {
        assert_eq!(round_trip(u32::MAX as usize + 1), None);
    }

    #[test]
    fn indices_order_by_position() {
        assert!(CrateIndex::from_index(1) < CrateIndex::from_index(2));
    }
}
