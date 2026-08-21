use crate::error::Error;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use unlade_core::CrateId;
use unlade_core::SemanticVersion;
use unlade_parser::Column;
use unlade_parser::CsvParser;
use unlade_parser::Fields;
use unlade_parser::Record;

const ID: &str = "id";
const CRATE_ID: &str = "crate_id";
const NUM: &str = "num";

/// The database identifier of a row in `versions.csv`.
///
/// This identifies a publication so it can be joined to `dependencies.csv`.
/// It is not the publication's semantic version; `PublishedVersion` keeps the
/// identifier and parsed [`SemanticVersion`] together while versions are
/// compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublishedVersionId(u32);

impl PublishedVersionId {
    /// Creates an identifier from the value stored in the dump.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the value stored in the dump.
    #[cfg(test)]
    pub fn get(self) -> u32 {
        self.0
    }
}

/// The greatest-precedence published version selected for each crate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GreatestVersions {
    by_crate: HashMap<CrateId, PublishedVersion>,
    ids: HashSet<PublishedVersionId>,
}

impl GreatestVersions {
    fn new() -> Self {
        Self::default()
    }

    /// Considers `candidate` for selection as its crate's greatest version.
    ///
    /// If the crate has no selection yet, `candidate` becomes its selection. If
    /// it has greater semantic-version precedence than the current selection, it replaces
    /// that selection. Equal- or lower-precedence candidates leave the current
    /// selection unchanged.
    fn consider(&mut self, candidate: PublishedVersion) {
        let selected = self.by_crate.get(&candidate.crate_id);
        if selected.is_none_or(|selected| candidate.outranks(selected)) {
            if let Some(selected) = selected {
                self.ids.remove(&selected.id);
            }
            self.ids.insert(candidate.id);
            self.by_crate.insert(candidate.crate_id, candidate);
        }
    }

    /// Whether `id` identifies a selected greatest-precedence publication.
    pub fn contains(&self, id: PublishedVersionId) -> bool {
        self.ids.contains(&id)
    }

    #[cfg(test)]
    fn version_ids(&self) -> impl Iterator<Item = PublishedVersionId> + '_ {
        self.ids.iter().copied()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.by_crate.len()
    }
}

/// Finds the version of every crate with the greatest semantic-version precedence.
///
/// Semantic-version build metadata does not affect precedence. If several publications
/// of one crate have equal precedence, the first one in the file is kept.
pub fn greatest_file(path: &Path) -> Result<GreatestVersions, Error> {
    greatest_parser(CsvParser::open(path)?)
}

#[cfg(test)]
pub fn greatest(reader: impl Read, path: &Path) -> Result<GreatestVersions, Error> {
    greatest_parser(CsvParser::new(reader, path)?)
}

fn greatest_parser(mut parser: CsvParser<impl Read, Error>) -> Result<GreatestVersions, Error> {
    let columns = Columns::locate(&parser)?;

    let mut greatest = GreatestVersions::new();
    let mut record = Record::new();
    while parser.read_record(&mut record)? {
        let published = read_version(&parser.fields(&record), &columns)?;
        greatest.consider(published);
    }

    Ok(greatest)
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishedVersion {
    id: PublishedVersionId,
    crate_id: CrateId,
    version: SemanticVersion,
}

impl PublishedVersion {
    /// Whether this publication has greater semantic-version precedence than `other`.
    fn outranks(&self, other: &Self) -> bool {
        self.version.outranks(&other.version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Columns {
    id: Column,
    crate_id: Column,
    num: Column,
}

impl Columns {
    fn locate(parser: &CsvParser<impl Read, Error>) -> Result<Self, Error> {
        Ok(Self {
            id: parser.column(ID)?,
            crate_id: parser.column(CRATE_ID)?,
            num: parser.column(NUM)?,
        })
    }
}

fn read_version(fields: &Fields<'_, Error>, columns: &Columns) -> Result<PublishedVersion, Error> {
    Ok(PublishedVersion {
        id: PublishedVersionId::new(read_identifier(fields, columns.id)?),
        crate_id: CrateId::new(read_identifier(fields, columns.crate_id)?),
        version: read_number(fields, columns.num)?,
    })
}

fn read_identifier(fields: &Fields<'_, Error>, column: Column) -> Result<u32, Error> {
    fields
        .text(column)?
        .parse()
        .map_err(|source| Error::InvalidIdentifier {
            path: fields.path().to_path_buf(),
            line: fields.line(),
            column: column.name(),
            source,
        })
}

fn read_number(fields: &Fields<'_, Error>, column: Column) -> Result<SemanticVersion, Error> {
    fields
        .text(column)?
        .parse()
        .map_err(|source| Error::InvalidVersion {
            path: fields.path().to_path_buf(),
            line: fields.line(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADERS: &str = "id,crate_id,num\n";

    fn greatest_of(rows: &str) -> HashSet<u32> {
        let text = format!("{HEADERS}{rows}");
        greatest(text.as_bytes(), Path::new("versions.csv"))
            .expect("rows parse")
            .version_ids()
            .map(PublishedVersionId::get)
            .collect()
    }

    fn error_of(rows: &str) -> Error {
        let text = format!("{HEADERS}{rows}");
        greatest(text.as_bytes(), Path::new("versions.csv")).expect_err("rows are malformed")
    }

    #[test]
    fn the_greatest_version_of_each_crate_is_kept() {
        let greatest = greatest_of("10,1,1.0.0\n11,1,2.0.0\n12,2,0.1.0\n");
        assert_eq!(greatest, HashSet::from([11, 12]));
    }

    #[test]
    fn versions_are_ordered_by_semver_not_by_text() {
        let greatest = greatest_of("10,1,0.9.0\n11,1,0.10.0\n");
        assert_eq!(greatest, HashSet::from([11]));
    }

    #[test]
    fn the_row_order_does_not_matter() {
        let greatest = greatest_of("10,1,2.0.0\n11,1,1.0.0\n");
        assert_eq!(greatest, HashSet::from([10]));
    }

    #[test]
    fn a_prerelease_ranks_below_its_release() {
        let greatest = greatest_of("10,1,2.0.0-alpha.1\n11,1,2.0.0\n");
        assert_eq!(greatest, HashSet::from([11]));
    }

    #[test]
    fn build_metadata_does_not_affect_version_precedence() {
        let greatest = greatest_of("10,1,1.0.0+z\n11,1,1.0.0+a\n");
        assert_eq!(greatest, HashSet::from([10]));
    }

    #[test]
    fn a_crate_with_one_version_keeps_it() {
        let greatest = greatest_of("10,1,0.1.0\n");
        assert_eq!(greatest, HashSet::from([10]));
    }

    #[test]
    fn a_header_without_rows_yields_nothing() {
        assert!(greatest_of("").is_empty());
    }

    #[test]
    fn extra_columns_are_ignored() {
        let text = "yanked,id,num,crate_id\nf,10,1.0.0,1\n";
        let greatest = greatest(text.as_bytes(), Path::new("versions.csv")).expect("parses");
        assert_eq!(greatest.len(), 1);
    }

    #[test]
    fn a_missing_column_is_reported() {
        let error = greatest("id,num\n".as_bytes(), Path::new("versions.csv"))
            .expect_err("header is incomplete");
        assert!(matches!(
            error,
            Error::MissingColumn {
                column: "crate_id",
                ..
            }
        ));
    }

    #[test]
    fn a_malformed_version_is_reported_with_its_line() {
        let error = error_of("10,1,1.0.0\n11,1,not-semver\n");
        assert!(matches!(error, Error::InvalidVersion { line: 3, .. }));
    }

    #[test]
    fn a_malformed_identifier_is_reported_with_its_column() {
        let error = error_of("x,1,1.0.0\n");
        assert!(matches!(
            error,
            Error::InvalidIdentifier {
                line: 2,
                column: "id",
                ..
            }
        ));
    }
}
