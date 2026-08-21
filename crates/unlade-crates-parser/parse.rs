use crate::columns::Columns;
use crate::error::Error;
use crate::row::read_row;
use std::io::Read;
use std::path::Path;
use unlade_core::CrateIdMap;
use unlade_core::Names;
use unlade_core::UpdateDates;
use unlade_parser::CsvParser;
use unlade_parser::Record;

/// Components read from `crates.csv`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedCrates {
    /// The crates.io identifiers and dense indices assigned to them.
    pub ids: CrateIdMap,
    /// Crate names, addressed by the indices in `ids`.
    pub names: Names,
    /// Last-update times, addressed by the indices in `ids`.
    pub update_dates: UpdateDates,
}

/// Reads a `crates.csv` dump.
///
/// Crates are stored in the order they appear in the file.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read as CSV, if the header
/// lacks a column the parser needs, or if a row holds a malformed identifier or
/// datetime.
pub fn parse_crates(path: &Path) -> Result<ParsedCrates, Error> {
    read_parser(CsvParser::open(path)?)
}

#[cfg(test)]
fn read(reader: impl Read, path: &Path) -> Result<ParsedCrates, Error> {
    read_parser(CsvParser::new(reader, path)?)
}

fn read_parser(mut parser: CsvParser<impl Read, Error>) -> Result<ParsedCrates, Error> {
    let columns = Columns::locate(&parser)?;

    let mut ids = CrateIdMap::new();
    let mut names = Names::new();
    let mut update_dates = UpdateDates::new();
    let mut record = Record::new();
    while parser.read_record(&mut record)? {
        let row = read_row(&parser.fields(&record), &columns)?;
        if ids.get(row.id).is_some() {
            continue;
        }

        let index = ids.get_or_insert(row.id);
        debug_assert_eq!(names.push(row.name), index);
        debug_assert_eq!(update_dates.push(row.updated_at), index);
    }

    Ok(ParsedCrates {
        ids,
        names,
        update_dates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::Timestamp;
    use unlade_core::CrateId;
    use unlade_core::CrateIndex;

    const HEADERS: &str = "id,name,updated_at\n";

    fn parse(text: &str) -> Result<ParsedCrates, Error> {
        read(text.as_bytes(), Path::new("crates.csv"))
    }

    fn parse_rows(rows: &str) -> ParsedCrates {
        parse(&format!("{HEADERS}{rows}")).expect("rows parse")
    }

    fn at(text: &str) -> Timestamp {
        text.parse().expect("timestamp parses")
    }

    fn index(position: usize) -> CrateIndex {
        CrateIndex::from_index(position).expect("position fits in an index")
    }

    fn names_of(crates: &ParsedCrates) -> Vec<&str> {
        crates.names.iter().map(|(_, name)| name).collect()
    }

    #[test]
    fn rows_become_crates() {
        let crates = parse_rows("1,serde,2020-01-01 12:11:10\n2,tokio,2021-06-05 08:00:00\n");

        assert_eq!(crates.ids.len(), 2);
        assert_eq!(names_of(&crates), vec!["serde", "tokio"]);
        assert_eq!(crates.ids.id(index(1)), Some(CrateId::new(2)));
        assert_eq!(
            crates.update_dates.get(index(0)),
            Some(at("2020-01-01T12:11:10Z")),
        );
    }

    #[test]
    fn every_store_addresses_the_same_crate() {
        let crates = parse_rows("7,serde,2020-01-01 12:11:10\n9,tokio,2021-06-05 08:00:00\n");
        let tokio = index(1);

        assert_eq!(crates.names.get(tokio), Some("tokio"));
        assert_eq!(crates.ids.id(tokio), Some(CrateId::new(9)));
        assert!(crates.update_dates.get(tokio).is_some());
    }

    #[test]
    fn columns_beyond_those_needed_are_ignored() {
        let text = "name,description,id,updated_at\nserde,a serializer,1,2020-01-01 12:11:10\n";
        let crates = parse(text).expect("rows parse");

        assert_eq!(names_of(&crates), vec!["serde"]);
        assert_eq!(crates.ids.id(index(0)), Some(CrateId::new(1)));
    }

    #[test]
    fn quoted_fields_keep_their_separators() {
        let crates = parse_rows("1,\"odd,name\",2020-01-01 12:11:10\n");
        assert_eq!(names_of(&crates), vec!["odd,name"]);
    }

    #[test]
    fn carriage_returns_are_not_part_of_a_field() {
        let crates =
            parse("id,name,updated_at\r\n1,serde,2020-01-01 12:11:10\r\n").expect("parses");
        assert_eq!(names_of(&crates), vec!["serde"]);
    }

    #[test]
    fn a_header_without_rows_yields_no_crates() {
        let crates = parse(HEADERS).expect("header parses");
        assert!(crates.ids.is_empty());
    }

    #[test]
    fn an_empty_file_reports_the_first_missing_column() {
        let error = parse("").expect_err("empty input fails");
        assert!(matches!(error, Error::MissingColumn { .. }));
    }

    #[test]
    fn a_missing_column_is_reported() {
        let error = parse("id,name\n1,serde\n").expect_err("header is incomplete");
        assert!(matches!(
            error,
            Error::MissingColumn {
                column: "updated_at",
                ..
            }
        ));
    }

    #[test]
    fn a_malformed_identifier_is_reported_with_its_line() {
        let error = error_of("1,serde,2020-01-01 12:11:10\nx,tokio,2020-01-01 12:11:10\n");
        assert!(matches!(error, Error::InvalidCrateId { line: 3, .. }));
    }

    #[test]
    fn a_malformed_update_date_is_reported_with_its_line() {
        let error = error_of("1,serde,yesterday\n");
        assert!(matches!(error, Error::InvalidUpdateDate { line: 2, .. }));
    }

    #[test]
    fn a_short_row_is_reported() {
        let error = error_of("1,serde\n");
        assert!(matches!(error, Error::Read { .. }));
    }

    fn error_of(rows: &str) -> Error {
        parse(&format!("{HEADERS}{rows}")).expect_err("rows are malformed")
    }
}
