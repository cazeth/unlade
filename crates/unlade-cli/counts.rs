use unlade_core::CrateIndex;
use unlade_core::Dependents;
use unlade_core::Downloads;

/// The counts a listing was asked for, each read from a file of its own and so
/// each absent unless an option needed it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    pub downloads: Option<Downloads>,
    pub dependents: Option<Dependents>,
}

impl Counts {
    /// How many times the crate at `index` has been downloaded.
    pub fn downloads(&self, index: CrateIndex) -> Option<u64> {
        self.downloads
            .as_ref()
            .and_then(|downloads| downloads.get(index))
    }

    /// How many crates depend on the crate at `index`.
    pub fn dependents(&self, index: CrateIndex) -> Option<u32> {
        self.dependents
            .as_ref()
            .and_then(|dependents| dependents.get(index))
    }
}
