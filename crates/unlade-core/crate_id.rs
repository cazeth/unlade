//! Identifiers assigned by crates.io.
/// The identifier crates.io assigns to a crate.
///
/// Dump files use these identifiers to refer to crates. [`CrateIdMap`](crate::CrateIdMap)
/// translates them into the [`CrateIndex`](crate::CrateIndex) values used by
/// component stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct CrateId(u32);

impl CrateId {
    /// Returns the identifier for `value`.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the underlying value.
    pub const fn get(self) -> u32 {
        self.0
    }
}
