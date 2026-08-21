//! Shared CSV parsing infrastructure for the unlade dump parsers.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]

pub use csv::Error as CsvError;
use csv::Position;
pub use csv::StringRecord as Record;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;

/// An error type that can represent structural CSV parsing failures.
///
/// Implementations remain in the public parser crates so those crates retain
/// their own error APIs and diagnostics.
pub trait ParseError: Sized {
    /// Creates an error for a file that could not be opened.
    fn open(path: PathBuf, source: io::Error) -> Self;

    /// Creates an error for a file that could not be read as CSV.
    fn read(path: PathBuf, source: CsvError) -> Self;

    /// Creates an error for a required column absent from the header.
    fn missing_column(path: PathBuf, column: &'static str) -> Self;

    /// Creates an error for a row that ends before a required field.
    fn missing_field(path: PathBuf, line: u64, column: &'static str) -> Self;
}

/// A located CSV column, coupling its name to its position.
///
/// Keeping these together prevents a schema parser from accidentally reporting
/// one column name while reading another column's position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    index: usize,
    name: &'static str,
}

impl Column {
    /// Returns the zero-based position of the column.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// Returns the column's header name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }
}

/// A streaming CSV parser with source and header context.
#[derive(Debug)]
pub struct CsvParser<R, E> {
    reader: csv::Reader<R>,
    path: PathBuf,
    headers: Record,
    error: PhantomData<fn() -> E>,
}

impl<E: ParseError> CsvParser<BufReader<File>, E> {
    /// Opens `path` and initializes a parser from its CSV header.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or its CSV header cannot
    /// be read.
    pub fn open(path: &Path) -> Result<Self, E> {
        let reader = File::open(path)
            .map(BufReader::new)
            .map_err(|source| E::open(path.to_path_buf(), source))?;
        Self::new(reader, path)
    }
}

impl<R: Read, E: ParseError> CsvParser<R, E> {
    /// Initializes a parser from `reader` and reads its CSV header.
    ///
    /// # Errors
    ///
    /// Returns an error if the CSV header cannot be read.
    pub fn new(reader: R, path: &Path) -> Result<Self, E> {
        let mut reader = csv::Reader::from_reader(reader);
        let headers = reader
            .headers()
            .cloned()
            .map_err(|source| E::read(path.to_path_buf(), source))?;

        Ok(Self {
            reader,
            path: path.to_path_buf(),
            headers,
            error: PhantomData,
        })
    }

    /// Locates a required column in the header.
    ///
    /// # Errors
    ///
    /// Returns an error if `name` is absent from the CSV header.
    pub fn column(&self, name: &'static str) -> Result<Column, E> {
        self.headers
            .iter()
            .position(|header| header == name)
            .map(|index| Column { index, name })
            .ok_or_else(|| E::missing_column(self.path.clone(), name))
    }

    /// Reads the next record, returning `false` at the end of the input.
    ///
    /// # Errors
    ///
    /// Returns an error if the next CSV record cannot be read.
    pub fn read_record(&mut self, record: &mut Record) -> Result<bool, E> {
        self.reader
            .read_record(record)
            .map_err(|source| E::read(self.path.clone(), source))
    }

    /// Adds source context to a record's fields.
    pub fn fields<'a>(&'a self, record: &'a Record) -> Fields<'a, E> {
        Fields::new(record, &self.path)
    }

    /// Returns the path being parsed.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// One CSV record with source context for field-level diagnostics.
#[derive(Debug)]
pub struct Fields<'a, E> {
    record: &'a Record,
    path: &'a Path,
    line: u64,
    error: PhantomData<fn() -> E>,
}

impl<'a, E: ParseError> Fields<'a, E> {
    fn new(record: &'a Record, path: &'a Path) -> Self {
        let line = record.position().map_or(0, Position::line);
        Self {
            record,
            path,
            line,
            error: PhantomData,
        }
    }

    /// Returns the field in `column`.
    ///
    /// # Errors
    ///
    /// Returns an error if the record ends before `column`.
    pub fn text(&self, column: Column) -> Result<&'a str, E> {
        self.record
            .get(column.index)
            .ok_or_else(|| E::missing_field(self.path.to_path_buf(), self.line, column.name))
    }

    /// Returns the line on which the record starts.
    #[must_use]
    pub const fn line(&self) -> u64 {
        self.line
    }

    /// Returns the path being parsed.
    #[must_use]
    pub const fn path(&self) -> &'a Path {
        self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Error {
        Open,
        Read,
        MissingColumn { column: &'static str },
        MissingField { line: u64, column: &'static str },
    }

    impl ParseError for Error {
        fn open(_: PathBuf, _: io::Error) -> Self {
            Self::Open
        }

        fn read(_: PathBuf, _: csv::Error) -> Self {
            Self::Read
        }

        fn missing_column(_: PathBuf, column: &'static str) -> Self {
            Self::MissingColumn { column }
        }

        fn missing_field(_: PathBuf, line: u64, column: &'static str) -> Self {
            Self::MissingField { line, column }
        }
    }

    #[test]
    fn columns_keep_their_name_and_position_together() {
        let parser = CsvParser::<_, Error>::new("name,id\n".as_bytes(), Path::new("data.csv"))
            .expect("header parses");
        let id = parser.column("id").expect("column exists");
        assert_eq!(id.index(), 1);
        assert_eq!(id.name(), "id");
    }

    #[test]
    fn a_missing_column_is_reported_by_name() {
        let parser = CsvParser::<_, Error>::new("name\n".as_bytes(), Path::new("data.csv"))
            .expect("header parses");
        assert_eq!(
            parser.column("id"),
            Err(Error::MissingColumn { column: "id" }),
        );
    }

    #[test]
    fn fields_report_their_line_and_column() {
        let parser = CsvParser::<_, Error>::new("id,name\n".as_bytes(), Path::new("data.csv"))
            .expect("header parses");
        let name = parser.column("name").expect("column exists");
        let record = Record::from(vec!["7"]);
        let error = parser
            .fields(&record)
            .text(name)
            .expect_err("field is absent");
        assert_eq!(
            error,
            Error::MissingField {
                line: 0,
                column: "name",
            },
        );
    }
}
