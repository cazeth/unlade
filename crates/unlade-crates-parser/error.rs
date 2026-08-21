use crate::date_time::InvalidDateTime;
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;

/// A failure while reading a `crates.csv` dump.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The file could not be opened.
    #[error("could not open `{path}`")]
    Open {
        /// The file that was being opened.
        path: PathBuf,
        /// The underlying failure.
        #[source]
        source: io::Error,
    },

    /// The file could not be read as CSV.
    #[error("could not read `{path}` as CSV")]
    Read {
        /// The file that was being read.
        path: PathBuf,
        /// The underlying failure.
        #[source]
        source: unlade_parser::CsvError,
    },

    /// A column the parser needs is absent from the header.
    #[error("`{path}` has no `{column}` column")]
    MissingColumn {
        /// The file that was being read.
        path: PathBuf,
        /// The column that was expected.
        column: &'static str,
    },

    /// A row ended before a column the parser needs.
    #[error("`{path}` line {line} has no `{column}` field")]
    MissingField {
        /// The file that was being read.
        path: PathBuf,
        /// The line the row started on.
        line: u64,
        /// The column that was expected.
        column: &'static str,
    },

    /// A row holds an identifier that is not a number.
    #[error("`{path}` line {line} has a malformed `id`")]
    InvalidCrateId {
        /// The file that was being read.
        path: PathBuf,
        /// The line the row started on.
        line: u64,
        /// The underlying failure.
        #[source]
        source: ParseIntError,
    },

    /// A row holds a download count that is not a number.
    #[error("`{path}` line {line} has a malformed `downloads`")]
    InvalidDownloads {
        /// The file that was being read.
        path: PathBuf,
        /// The line the row started on.
        line: u64,
        /// The underlying failure.
        #[source]
        source: ParseIntError,
    },

    /// A row holds a datetime the parser does not recognise.
    #[error("`{path}` line {line} has a malformed `updated_at`")]
    InvalidUpdateDate {
        /// The file that was being read.
        path: PathBuf,
        /// The line the row started on.
        line: u64,
        /// The underlying failure.
        #[source]
        source: InvalidDateTime,
    },
}

impl unlade_parser::ParseError for Error {
    fn open(path: PathBuf, source: io::Error) -> Self {
        Self::Open { path, source }
    }

    fn read(path: PathBuf, source: unlade_parser::CsvError) -> Self {
        Self::Read { path, source }
    }

    fn missing_column(path: PathBuf, column: &'static str) -> Self {
        Self::MissingColumn { path, column }
    }

    fn missing_field(path: PathBuf, line: u64, column: &'static str) -> Self {
        Self::MissingField { path, line, column }
    }
}
