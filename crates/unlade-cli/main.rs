//! Lists crates from an extracted crates.io database dump.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod arguments;
mod counts;
mod filter;
mod report;

use crate::arguments::Arguments;
use crate::counts::Counts;
use crate::filter::Filter;
use clap::Parser;
use miette::IntoDiagnostic;
use miette::Result;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use unlade_core::CrateIdMap;
use unlade_crates_parser::parse_crates;
use unlade_crates_parser::parse_downloads;
use unlade_dependencies_parser::count_dependents;

fn main() -> Result<()> {
    let arguments = Arguments::parse();
    let filter = Filter::new(&arguments);

    let crates = parse_crates(&arguments.crates_path()).into_diagnostic()?;
    let counts = read_counts(&arguments, &filter, &crates.ids)?;
    let selected = filter.select(&crates.names, &crates.update_dates, &counts);

    let mut writer = BufWriter::new(io::stdout().lock());
    report::write(
        &mut writer,
        &crates.names,
        &crates.update_dates,
        &selected,
        &counts,
    )
    .into_diagnostic()?;
    writer.flush().into_diagnostic()
}

/// Reads the counts the listing asks for, each of which takes a file the
/// listing would not otherwise open.
fn read_counts(arguments: &Arguments, filter: &Filter, ids: &CrateIdMap) -> Result<Counts> {
    let mut counts = Counts::default();

    if filter.needs_downloads() {
        let downloads = parse_downloads(&arguments.downloads_path(), ids).into_diagnostic()?;
        counts.downloads = Some(downloads);
    }

    if filter.needs_dependents() {
        let dependents = count_dependents(
            &arguments.versions_path(),
            &arguments.dependencies_path(),
            ids,
        )
        .into_diagnostic()?;
        counts.dependents = Some(dependents);
    }

    Ok(counts)
}
