use crate::error::Error;
use std::io::Read;
use std::path::Path;
use unlade_core::CrateIdMap;
use unlade_core::Downloads;
use unlade_parser::Column;
use unlade_parser::CsvParser;
use unlade_parser::Fields;
use unlade_parser::Record;

const CRATE_ID: &str = "crate_id";
const DOWNLOADS: &str = "downloads";

/// Reads a `crate_downloads.csv` dump.
///
/// The returned counts are addressed by the same
/// [`CrateIndex`](unlade_core::CrateIndex) as `crates`, and a crate the file
/// holds no row for counts as never downloaded.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read as CSV, if the header
/// lacks a column the parser needs, or if a row holds a malformed identifier or
/// count.
pub fn parse_downloads(path: &Path, ids: &CrateIdMap) -> Result<Downloads, Error> {
    read_downloads_parser(CsvParser::open(path)?, ids)
}

#[cfg(test)]
fn read_downloads(reader: impl Read, path: &Path, ids: &CrateIdMap) -> Result<Downloads, Error> {
    read_downloads_parser(CsvParser::new(reader, path)?, ids)
}

fn read_downloads_parser(
    mut parser: CsvParser<impl Read, Error>,
    ids: &CrateIdMap,
) -> Result<Downloads, Error> {
    let columns = Columns::locate(&parser)?;

    let mut counts = vec![0_u64; ids.len()];
    let mut record = Record::new();
    while parser.read_record(&mut record)? {
        let row = read_row(&parser.fields(&record), &columns)?;
        if let Some(index) = ids.get(row.id) {
            counts[index.index()] = row.downloads;
        }
    }

    Ok(collect(counts))
}

fn collect(counts: Vec<u64>) -> Downloads {
    let mut downloads = Downloads::with_capacity(counts.len());
    for count in counts {
        downloads.push(count);
    }
    downloads
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row {
    id: unlade_core::CrateId,
    downloads: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Columns {
    crate_id: Column,
    downloads: Column,
}

impl Columns {
    fn locate(parser: &CsvParser<impl Read, Error>) -> Result<Self, Error> {
        Ok(Self {
            crate_id: parser.column(CRATE_ID)?,
            downloads: parser.column(DOWNLOADS)?,
        })
    }
}

fn read_row(fields: &Fields<'_, Error>, columns: &Columns) -> Result<Row, Error> {
    Ok(Row {
        id: read_crate_id(fields, columns.crate_id)?,
        downloads: read_count(fields, columns.downloads)?,
    })
}

fn read_crate_id(
    fields: &Fields<'_, Error>,
    column: Column,
) -> Result<unlade_core::CrateId, Error> {
    fields
        .text(column)?
        .parse()
        .map(unlade_core::CrateId::new)
        .map_err(|source| Error::InvalidCrateId {
            path: fields.path().to_path_buf(),
            line: fields.line(),
            source,
        })
}

fn read_count(fields: &Fields<'_, Error>, column: Column) -> Result<u64, Error> {
    fields
        .text(column)?
        .parse()
        .map_err(|source| Error::InvalidDownloads {
            path: fields.path().to_path_buf(),
            line: fields.line(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use unlade_core::CrateId;
    use unlade_core::CrateIndex;

    const HEADERS: &str = "crate_id,downloads\n";

    fn crate_ids(ids: &[u32]) -> CrateIdMap {
        let mut map = CrateIdMap::new();
        for id in ids {
            map.get_or_insert(CrateId::new(*id));
        }
        map
    }

    fn index(position: usize) -> CrateIndex {
        CrateIndex::from_index(position).expect("position fits in an index")
    }

    fn downloads_of(ids: &[u32], rows: &str) -> Vec<u64> {
        let text = format!("{HEADERS}{rows}");
        read_downloads(
            text.as_bytes(),
            Path::new("crate_downloads.csv"),
            &crate_ids(ids),
        )
        .expect("rows parse")
        .iter()
        .map(|(_, downloads)| downloads)
        .collect()
    }

    fn error_of(rows: &str) -> Error {
        let text = format!("{HEADERS}{rows}");
        read_downloads(
            text.as_bytes(),
            Path::new("crate_downloads.csv"),
            &crate_ids(&[1]),
        )
        .expect_err("rows are malformed")
    }

    #[test]
    fn counts_are_placed_at_the_index_of_their_crate() {
        assert_eq!(downloads_of(&[1, 2], "2,900\n1,500\n"), vec![500, 900]);
    }

    #[test]
    fn a_crate_without_a_row_has_never_been_downloaded() {
        assert_eq!(downloads_of(&[1, 2], "1,500\n"), vec![500, 0]);
    }

    #[test]
    fn rows_for_unknown_crates_are_ignored() {
        assert_eq!(downloads_of(&[1], "1,500\n99,700\n"), vec![500]);
    }

    #[test]
    fn counts_beyond_a_smaller_integer_are_read() {
        assert_eq!(downloads_of(&[1], "1,5000000000\n"), vec![5_000_000_000]);
    }

    #[test]
    fn extra_columns_are_ignored() {
        let text = "downloads,id,crate_id\n500,7,1\n";
        let counts = read_downloads(
            text.as_bytes(),
            Path::new("crate_downloads.csv"),
            &crate_ids(&[1]),
        )
        .expect("parses");

        assert_eq!(counts.get(index(0)), Some(500));
    }

    #[test]
    fn a_missing_column_is_reported() {
        let error = read_downloads(
            "crate_id\n".as_bytes(),
            Path::new("crate_downloads.csv"),
            &crate_ids(&[1]),
        )
        .expect_err("header is incomplete");

        assert!(matches!(
            error,
            Error::MissingColumn {
                column: "downloads",
                ..
            }
        ));
    }

    #[test]
    fn a_malformed_count_is_reported_with_its_line() {
        let error = error_of("1,many\n");
        assert!(matches!(error, Error::InvalidDownloads { line: 2, .. }));
    }

    #[test]
    fn a_malformed_identifier_is_reported() {
        let error = error_of("x,500\n");
        assert!(matches!(error, Error::InvalidCrateId { .. }));
    }
}
