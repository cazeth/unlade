use crate::dependencies;
use crate::error::Error;
use crate::versions;
use std::path::Path;
use unlade_core::CrateIdMap;
use unlade_core::CrateIndex;
use unlade_core::Dependents;

/// Counts how many crates depend on each crate in `crates`.
///
/// A crate is counted once for every other crate whose version has the greatest
/// semantic-version precedence and names it as an ordinary dependency. Build and
/// development dependencies do not count; optional ones do.
///
/// The returned counts are addressed by the same
/// [`CrateIndex`](unlade_core::CrateIndex) as `crates`.
///
/// # Errors
///
/// Returns an error if either file cannot be opened or read as CSV, if a header
/// lacks a column the parser needs, or if a row holds a malformed identifier,
/// version, or dependency kind.
pub fn count_dependents(
    versions_path: &Path,
    dependencies_path: &Path,
    ids: &CrateIdMap,
) -> Result<Dependents, Error> {
    let greatest = versions::greatest_file(versions_path)?;

    let mut tally = Tally::new(ids.len());
    dependencies::read_file(dependencies_path, &greatest, |id| {
        if let Some(index) = ids.get(id) {
            tally.count(index);
        }
    })?;

    Ok(tally.finish())
}

/// A running count for every crate, kept in the order the crates were read.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Tally {
    counts: Vec<u32>,
}

impl Tally {
    fn new(crate_count: usize) -> Self {
        Self {
            counts: vec![0; crate_count],
        }
    }

    /// Counts one crate depending on the crate at `index`.
    fn count(&mut self, index: CrateIndex) {
        let count = &mut self.counts[index.index()];
        *count = count.saturating_add(1);
    }

    fn finish(self) -> Dependents {
        let mut dependents = Dependents::with_capacity(self.counts.len());
        for count in self.counts {
            dependents.push(count);
        }
        dependents
    }
}
