use clap::Parser;
use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use std::path::PathBuf;

/// Lists crates from an extracted crates.io database dump.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(name = "unlade", version, about)]
pub struct Arguments {
    /// The directory holding the dump's CSV files.
    pub directory: PathBuf,

    /// Keep only crates last updated before this date.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_date)]
    pub updated_before: Option<Timestamp>,

    /// Keep only crates last updated on or after this date.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_date)]
    pub updated_after: Option<Timestamp>,

    /// Keep only crates whose name contains this text.
    #[arg(long, value_name = "TEXT")]
    pub name_contains: Option<String>,

    /// Keep only crates downloaded at least this many times.
    ///
    /// Reads `crate_downloads.csv`, which the other options do not need.
    #[arg(long, value_name = "COUNT")]
    pub min_downloads: Option<u64>,

    /// Keep only crates that this many other crates depend on.
    ///
    /// Reads `versions.csv` and `dependencies.csv`, which the other options do
    /// not need.
    #[arg(long, value_name = "COUNT")]
    pub min_dependents: Option<u32>,

    /// Show at most this many crates.
    #[arg(long, value_name = "COUNT")]
    pub limit: Option<usize>,
}

impl Arguments {
    /// The file holding every crate.
    pub fn crates_path(&self) -> PathBuf {
        self.directory.join("crates.csv")
    }

    /// The file holding the downloads of every crate.
    pub fn downloads_path(&self) -> PathBuf {
        self.directory.join("crate_downloads.csv")
    }

    /// The file holding every published version.
    pub fn versions_path(&self) -> PathBuf {
        self.directory.join("versions.csv")
    }

    /// The file holding the dependencies of every version.
    pub fn dependencies_path(&self) -> PathBuf {
        self.directory.join("dependencies.csv")
    }
}

/// Reads a calendar date as the first instant of that day in UTC.
fn parse_date(text: &str) -> Result<Timestamp, String> {
    let date: Date = text
        .parse()
        .map_err(|_| format!("`{text}` is not a date"))?;

    date.to_zoned(TimeZone::UTC)
        .map(|zoned| zoned.timestamp())
        .map_err(|_| format!("`{text}` has no start of day in UTC"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn parse(arguments: &[&str]) -> Result<Arguments, clap::Error> {
        let mut all = vec!["unlade"];
        all.extend_from_slice(arguments);
        Arguments::try_parse_from(all)
    }

    fn parse_ok(arguments: &[&str]) -> Arguments {
        parse(arguments).expect("arguments are valid")
    }

    fn at(text: &str) -> Timestamp {
        text.parse().expect("timestamp parses")
    }

    #[test]
    fn the_command_is_well_formed() {
        Arguments::command().debug_assert();
    }

    #[test]
    fn the_directory_is_required() {
        assert!(parse(&[]).is_err());
    }

    #[test]
    fn a_directory_alone_is_enough() {
        let arguments = parse_ok(&["dump/data"]);
        assert_eq!(arguments.directory, PathBuf::from("dump/data"));
        assert_eq!(arguments.limit, None);
        assert_eq!(arguments.name_contains, None);
    }

    #[test]
    fn dates_are_read_as_the_start_of_the_day_in_utc() {
        let arguments = parse_ok(&["dump", "--updated-before", "2020-01-01"]);
        assert_eq!(arguments.updated_before, Some(at("2020-01-01T00:00:00Z")));
    }

    #[test]
    fn every_filter_can_be_given_at_once() {
        let arguments = parse_ok(&[
            "dump",
            "--updated-before",
            "2020-01-01",
            "--updated-after",
            "2015-01-01",
            "--name-contains",
            "serde",
            "--limit",
            "10",
        ]);

        assert_eq!(arguments.updated_after, Some(at("2015-01-01T00:00:00Z")));
        assert_eq!(arguments.name_contains.as_deref(), Some("serde"));
        assert_eq!(arguments.limit, Some(10));
    }

    #[test]
    fn a_malformed_date_is_rejected() {
        assert!(parse(&["dump", "--updated-before", "yesterday"]).is_err());
        assert!(parse(&["dump", "--updated-before", "2020-02-30"]).is_err());
    }

    #[test]
    fn every_file_is_looked_for_in_the_directory() {
        let arguments = parse_ok(&["dump/data"]);
        assert_eq!(
            arguments.crates_path(),
            PathBuf::from("dump/data/crates.csv")
        );
        assert_eq!(
            arguments.versions_path(),
            PathBuf::from("dump/data/versions.csv"),
        );
        assert_eq!(
            arguments.dependencies_path(),
            PathBuf::from("dump/data/dependencies.csv"),
        );
    }

    #[test]
    fn a_download_count_is_read() {
        assert_eq!(
            parse_ok(&["dump", "--min-downloads", "1000000"]).min_downloads,
            Some(1_000_000),
        );
    }

    #[test]
    fn a_dependent_count_is_read() {
        assert_eq!(
            parse_ok(&["dump", "--min-dependents", "25"]).min_dependents,
            Some(25)
        );
    }

    #[test]
    fn a_malformed_limit_is_rejected() {
        assert!(parse(&["dump", "--limit", "many"]).is_err());
    }
}
