use crate::error::Error;
use crate::versions::GreatestVersions;
use crate::versions::PublishedVersionId;
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use unlade_core::CrateId;
use unlade_parser::Column;
use unlade_parser::CsvParser;
use unlade_parser::Fields;
use unlade_parser::Record;

const VERSION_ID: &str = "version_id";
const CRATE_ID: &str = "crate_id";
const KIND: &str = "kind";

/// The value `dependencies.csv` writes for an ordinary dependency, as opposed
/// to a build or development one.
const NORMAL: &str = "0";

const BUILD: &str = "1";
const DEVELOPMENT: &str = "2";

/// Counts, for each crate, how many other crates depend on it.
///
/// Only ordinary dependencies of the greatest-precedence versions are counted,
/// and a dependent crate is counted once however many times it names the
/// dependency.
pub fn read_file(
    path: &Path,
    greatest: &GreatestVersions,
    count: impl FnMut(CrateId),
) -> Result<(), Error> {
    read_parser(CsvParser::open(path)?, greatest, count)
}

#[cfg(test)]
pub fn read(
    reader: impl Read,
    path: &Path,
    greatest: &GreatestVersions,
    count: impl FnMut(CrateId),
) -> Result<(), Error> {
    read_parser(CsvParser::new(reader, path)?, greatest, count)
}

fn read_parser(
    mut parser: CsvParser<impl Read, Error>,
    greatest: &GreatestVersions,
    mut count: impl FnMut(CrateId),
) -> Result<(), Error> {
    let columns = Columns::locate(&parser)?;

    let mut counted = HashSet::new();
    let mut record = Record::new();
    while parser.read_record(&mut record)? {
        let Some(row) = read_row(&parser.fields(&record), &columns)? else {
            continue;
        };
        if !greatest.contains(row.version_id) {
            continue;
        }
        let edge = DependencyEdge::new(row.version_id, row.crate_id);
        if counted.insert(edge) {
            count(row.crate_id);
        }
    }

    Ok(())
}

/// A dependency from one published version to one crate.
///
/// This is used as a set key so that repeated rows for different targets count
/// as one dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DependencyEdge {
    version_id: PublishedVersionId,
    crate_id: CrateId,
}

impl DependencyEdge {
    fn new(version_id: PublishedVersionId, crate_id: CrateId) -> Self {
        Self {
            version_id,
            crate_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row {
    version_id: PublishedVersionId,
    crate_id: CrateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Columns {
    version_id: Column,
    crate_id: Column,
    kind: Column,
}

impl Columns {
    fn locate(parser: &CsvParser<impl Read, Error>) -> Result<Self, Error> {
        Ok(Self {
            version_id: parser.column(VERSION_ID)?,
            crate_id: parser.column(CRATE_ID)?,
            kind: parser.column(KIND)?,
        })
    }
}

/// Reads a row, or nothing when it is not an ordinary dependency.
fn read_row(fields: &Fields<'_, Error>, columns: &Columns) -> Result<Option<Row>, Error> {
    if !is_normal(fields, columns.kind)? {
        return Ok(None);
    }

    Ok(Some(Row {
        version_id: PublishedVersionId::new(read_identifier(fields, columns.version_id)?),
        crate_id: CrateId::new(read_identifier(fields, columns.crate_id)?),
    }))
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

fn is_normal(fields: &Fields<'_, Error>, column: Column) -> Result<bool, Error> {
    match fields.text(column)? {
        NORMAL => Ok(true),
        BUILD | DEVELOPMENT => Ok(false),
        kind => Err(Error::UnknownDependencyKind {
            path: fields.path().to_path_buf(),
            line: fields.line(),
            kind: kind.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADERS: &str = "version_id,crate_id,kind\n";

    fn greatest_of(ids: &[u32]) -> GreatestVersions {
        use std::fmt::Write as _;

        let mut text = String::from("id,crate_id,num\n");
        for (position, id) in ids.iter().enumerate() {
            writeln!(text, "{id},{},1.0.0", position + 1).expect("writing to a string succeeds");
        }

        crate::versions::greatest(text.as_bytes(), Path::new("versions.csv"))
            .expect("versions parse")
    }

    fn counts_of(greatest: &[u32], rows: &str) -> Vec<u32> {
        let text = format!("{HEADERS}{rows}");
        let mut counted = Vec::new();
        read(
            text.as_bytes(),
            Path::new("dependencies.csv"),
            &greatest_of(greatest),
            |crate_id| counted.push(crate_id.get()),
        )
        .expect("rows parse");
        counted.sort_unstable();
        counted
    }

    fn error_of(rows: &str) -> Error {
        let text = format!("{HEADERS}{rows}");
        read(
            text.as_bytes(),
            Path::new("dependencies.csv"),
            &greatest_of(&[10]),
            |_| (),
        )
        .expect_err("rows are malformed")
    }

    #[test]
    fn a_dependency_of_a_greatest_precedence_version_is_counted() {
        assert_eq!(counts_of(&[10], "10,7,0\n"), vec![7]);
    }

    #[test]
    fn dependencies_of_older_versions_are_ignored() {
        assert!(counts_of(&[10], "9,7,0\n").is_empty());
    }

    #[test]
    fn build_and_development_dependencies_are_ignored() {
        assert!(counts_of(&[10], "10,7,1\n10,8,2\n").is_empty());
    }

    #[test]
    fn one_version_naming_a_crate_twice_counts_once() {
        assert_eq!(counts_of(&[10], "10,7,0\n10,7,0\n"), vec![7]);
    }

    #[test]
    fn separate_versions_naming_a_crate_each_count() {
        assert_eq!(counts_of(&[10, 11], "10,7,0\n11,7,0\n"), vec![7, 7]);
    }

    #[test]
    fn a_version_naming_several_crates_counts_each() {
        assert_eq!(counts_of(&[10], "10,7,0\n10,8,0\n"), vec![7, 8]);
    }

    #[test]
    fn extra_columns_are_ignored() {
        let text = "id,version_id,optional,crate_id,kind\n1,10,t,7,0\n";
        let mut counted = Vec::new();
        read(
            text.as_bytes(),
            Path::new("dependencies.csv"),
            &greatest_of(&[10]),
            |crate_id| counted.push(crate_id.get()),
        )
        .expect("parses");

        assert_eq!(counted, vec![7]);
    }

    #[test]
    fn a_missing_column_is_reported() {
        let error = read(
            "version_id,crate_id\n".as_bytes(),
            Path::new("dependencies.csv"),
            &greatest_of(&[10]),
            |_| (),
        )
        .expect_err("header is incomplete");

        assert!(matches!(error, Error::MissingColumn { column: "kind", .. }));
    }

    #[test]
    fn an_unknown_kind_is_reported_with_its_line() {
        let error = error_of("10,7,9\n");
        assert!(matches!(
            error,
            Error::UnknownDependencyKind { line: 2, .. }
        ));
    }

    #[test]
    fn a_malformed_identifier_is_reported_with_its_column() {
        let error = error_of("10,x,0\n");
        assert!(matches!(
            error,
            Error::InvalidIdentifier {
                column: "crate_id",
                ..
            }
        ));
    }
}
