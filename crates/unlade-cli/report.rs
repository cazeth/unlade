use crate::counts::Counts;
use jiff::Timestamp;
use jiff::tz::TimeZone;
use std::io;
use std::io::Write;
use unlade_core::CrateIndex;
use unlade_core::Names;
use unlade_core::UpdateDates;

/// Writes one line per crate, naming it and the day it was last updated.
///
/// Every count that is known follows, downloads before dependents, so the
/// columns of a listing are the same on every line.
///
/// # Errors
///
/// Returns an error if the writer does.
pub fn write(
    writer: &mut impl Write,
    names: &Names,
    update_dates: &UpdateDates,
    selected: &[CrateIndex],
    counts: &Counts,
) -> io::Result<()> {
    let width = widest(names, selected);

    for index in selected {
        let name = &names[*index];
        let date = day(update_dates[*index]);
        let counted = count_columns(counts, *index);
        writeln!(writer, "{name:width$}  {date}{counted}")?;
    }

    Ok(())
}

/// The known counts, each as a column of its own.
fn count_columns(counts: &Counts, index: CrateIndex) -> String {
    let downloads = counts.downloads(index).map(|count| count.to_string());
    let dependents = counts.dependents(index).map(|count| count.to_string());

    let mut columns = String::new();
    for count in downloads.into_iter().chain(dependents) {
        columns.push_str("  ");
        columns.push_str(&count);
    }
    columns
}

fn widest(names: &Names, selected: &[CrateIndex]) -> usize {
    selected
        .iter()
        .map(|index| names[*index].chars().count())
        .max()
        .unwrap_or_default()
}

fn day(updated_at: Timestamp) -> String {
    updated_at.to_zoned(TimeZone::UTC).date().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use unlade_core::Dependents;
    use unlade_core::Downloads;

    fn at(text: &str) -> Timestamp {
        text.parse().expect("timestamp parses")
    }

    fn components_of(entries: &[(&str, &str)]) -> (Names, UpdateDates) {
        let mut names = Names::new();
        let mut update_dates = UpdateDates::new();
        for (name, updated_at) in entries {
            names.push(*name);
            update_dates.push(at(updated_at));
        }
        (names, update_dates)
    }

    fn report_of(entries: &[(&str, &str)]) -> String {
        report_with(entries, &Counts::default())
    }

    fn report_with(entries: &[(&str, &str)], counts: &Counts) -> String {
        let (names, update_dates) = components_of(entries);
        let selected: Vec<_> = names.iter().map(|(index, _)| index).collect();

        let mut written = Vec::new();
        write(&mut written, &names, &update_dates, &selected, counts)
            .expect("writing to memory succeeds");
        String::from_utf8(written).expect("the report is text")
    }

    fn dependents_of(values: &[u32]) -> Counts {
        let mut dependents = Dependents::new();
        for value in values {
            dependents.push(*value);
        }
        Counts {
            dependents: Some(dependents),
            ..Counts::default()
        }
    }

    fn both_counts(downloads: &[u64], dependents: &[u32]) -> Counts {
        let mut counted = dependents_of(dependents);
        let mut store = Downloads::new();
        for value in downloads {
            store.push(*value);
        }
        counted.downloads = Some(store);
        counted
    }

    #[test]
    fn each_crate_takes_a_line() {
        let report = report_of(&[
            ("serde", "2024-01-01T00:00:00Z"),
            ("tokio", "2016-05-05T09:30:00Z"),
        ]);

        assert_eq!(report.lines().count(), 2);
    }

    #[test]
    fn a_line_names_the_crate_and_the_day_it_was_updated() {
        let report = report_of(&[("serde", "2024-01-01T13:45:00Z")]);
        assert_eq!(report, "serde  2024-01-01\n");
    }

    #[test]
    fn names_are_padded_to_the_widest_of_them() {
        let report = report_of(&[
            ("serde", "2024-01-01T00:00:00Z"),
            ("rustc-serialize", "2016-05-05T00:00:00Z"),
        ]);
        let dates: Vec<_> = report
            .lines()
            .map(|line| line.find("20").expect("a date is on every line"))
            .collect();

        assert_eq!(dates[0], dates[1]);
    }

    #[test]
    fn known_counts_end_the_line() {
        let counts = dependents_of(&[4_242]);
        let report = report_with(&[("serde", "2024-01-01T13:45:00Z")], &counts);

        assert_eq!(report, "serde  2024-01-01  4242\n");
    }

    #[test]
    fn downloads_come_before_dependents() {
        let counts = both_counts(&[9_000_000], &[4_242]);
        let report = report_with(&[("serde", "2024-01-01T13:45:00Z")], &counts);

        assert_eq!(report, "serde  2024-01-01  9000000  4242\n");
    }

    #[test]
    fn unknown_counts_leave_the_line_as_it_was() {
        let report = report_of(&[("serde", "2024-01-01T13:45:00Z")]);
        assert_eq!(report, "serde  2024-01-01\n");
    }

    #[test]
    fn nothing_selected_writes_nothing() {
        let (names, update_dates) = components_of(&[("serde", "2024-01-01T00:00:00Z")]);
        let mut written = Vec::new();
        write(&mut written, &names, &update_dates, &[], &Counts::default())
            .expect("writing to memory succeeds");

        assert!(written.is_empty());
    }
}
