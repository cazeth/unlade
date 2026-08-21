use crate::arguments::Arguments;
use crate::counts::Counts;
use jiff::Timestamp;
use unlade_core::CrateIndex;
use unlade_core::Names;
use unlade_core::UpdateDates;

/// The conditions a crate has to meet to be listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    updated_before: Option<Timestamp>,
    updated_after: Option<Timestamp>,
    name_contains: Option<String>,
    min_downloads: Option<u64>,
    min_dependents: Option<u32>,
    limit: Option<usize>,
}

impl Filter {
    pub fn new(arguments: &Arguments) -> Self {
        Self {
            updated_before: arguments.updated_before,
            updated_after: arguments.updated_after,
            name_contains: arguments.name_contains.clone(),
            min_downloads: arguments.min_downloads,
            min_dependents: arguments.min_dependents,
            limit: arguments.limit,
        }
    }

    /// Whether download counts have to be known to apply this filter.
    pub fn needs_downloads(&self) -> bool {
        self.min_downloads.is_some()
    }

    /// Whether the counts of depending crates have to be known to apply this
    /// filter.
    pub fn needs_dependents(&self) -> bool {
        self.min_dependents.is_some()
    }

    /// Returns the crates that meet every condition, in the order they were
    /// read.
    pub fn select(
        &self,
        names: &Names,
        update_dates: &UpdateDates,
        counts: &Counts,
    ) -> Vec<CrateIndex> {
        let limit = self.limit.unwrap_or(usize::MAX);

        names
            .iter()
            .filter(|(index, name)| self.matches(name, update_dates[*index], counts, *index))
            .map(|(index, _)| index)
            .take(limit)
            .collect()
    }

    fn matches(
        &self,
        name: &str,
        updated_at: Timestamp,
        counts: &Counts,
        index: CrateIndex,
    ) -> bool {
        self.is_named(name)
            && self.is_updated_within(updated_at)
            && self.is_downloaded(counts.downloads(index))
            && self.is_depended_on(counts.dependents(index))
    }

    /// A crate with an unknown count never meets a condition on that count.
    fn is_downloaded(&self, downloads: Option<u64>) -> bool {
        let Some(minimum) = self.min_downloads else {
            return true;
        };

        downloads.is_some_and(|count| count >= minimum)
    }

    /// A crate with an unknown count never meets a condition on that count.
    fn is_depended_on(&self, dependents: Option<u32>) -> bool {
        let Some(minimum) = self.min_dependents else {
            return true;
        };

        dependents.is_some_and(|count| count >= minimum)
    }

    fn is_named(&self, name: &str) -> bool {
        self.name_contains
            .as_ref()
            .is_none_or(|text| name.contains(text.as_str()))
    }

    fn is_updated_within(&self, updated_at: Timestamp) -> bool {
        let before = self.updated_before.is_none_or(|limit| updated_at < limit);
        let after = self.updated_after.is_none_or(|limit| updated_at >= limit);
        before && after
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unlade_core::Dependents;
    use unlade_core::Downloads;

    fn at(text: &str) -> Timestamp {
        text.parse().expect("timestamp parses")
    }

    fn sample() -> (Names, UpdateDates) {
        let mut names = Names::new();
        let mut update_dates = UpdateDates::new();
        let entries = [
            ("serde", "2024-01-01T00:00:00Z"),
            ("serde_json", "2016-05-05T00:00:00Z"),
            ("tokio", "2024-06-05T00:00:00Z"),
            ("rustc-serialize", "2016-01-01T00:00:00Z"),
        ];
        for (name, updated_at) in entries {
            names.push(name);
            update_dates.push(at(updated_at));
        }
        (names, update_dates)
    }

    fn filter_of(arguments: &[&str]) -> Filter {
        let mut all = vec!["unlade", "dump"];
        all.extend_from_slice(arguments);
        let arguments = <Arguments as clap::Parser>::try_parse_from(all).expect("arguments parse");
        Filter::new(&arguments)
    }

    fn names_of(arguments: &[&str]) -> Vec<String> {
        names_with(arguments, &Counts::default())
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

    fn downloads_of(values: &[u64]) -> Counts {
        let mut downloads = Downloads::new();
        for value in values {
            downloads.push(*value);
        }
        Counts {
            downloads: Some(downloads),
            ..Counts::default()
        }
    }

    fn names_with(arguments: &[&str], counts: &Counts) -> Vec<String> {
        let (names, update_dates) = sample();
        filter_of(arguments)
            .select(&names, &update_dates, counts)
            .into_iter()
            .map(|index| names[index].to_owned())
            .collect()
    }

    #[test]
    fn without_conditions_every_crate_is_listed() {
        assert_eq!(names_of(&[]).len(), 4);
    }

    #[test]
    fn crates_updated_before_a_date_are_kept() {
        assert_eq!(
            names_of(&["--updated-before", "2020-01-01"]),
            vec!["serde_json", "rustc-serialize"],
        );
    }

    #[test]
    fn crates_updated_after_a_date_are_kept() {
        assert_eq!(
            names_of(&["--updated-after", "2020-01-01"]),
            vec!["serde", "tokio"],
        );
    }

    #[test]
    fn the_bounds_can_be_combined_into_a_range() {
        assert_eq!(
            names_of(&[
                "--updated-after",
                "2016-02-01",
                "--updated-before",
                "2020-01-01"
            ]),
            vec!["serde_json"],
        );
    }

    #[test]
    fn a_date_bound_excludes_its_own_instant_at_the_upper_end() {
        assert_eq!(
            names_of(&["--updated-before", "2016-05-05"]),
            vec!["rustc-serialize"],
        );
        assert_eq!(
            names_of(&["--updated-after", "2016-05-05"]),
            vec!["serde", "serde_json", "tokio"],
        );
    }

    #[test]
    fn names_are_matched_anywhere_in_the_name() {
        assert_eq!(
            names_of(&["--name-contains", "serde"]),
            vec!["serde", "serde_json"],
        );
    }

    #[test]
    fn conditions_apply_together() {
        assert_eq!(
            names_of(&["--name-contains", "serde", "--updated-before", "2020-01-01"]),
            vec!["serde_json"],
        );
    }

    #[test]
    fn crates_with_enough_dependents_are_kept() {
        let counts = dependents_of(&[500, 3, 900, 0]);
        assert_eq!(
            names_with(&["--min-dependents", "100"], &counts),
            vec!["serde", "tokio"],
        );
    }

    #[test]
    fn crates_with_enough_downloads_are_kept() {
        let counts = downloads_of(&[9_000_000_000, 12, 400, 7]);
        assert_eq!(
            names_with(&["--min-downloads", "400"], &counts),
            vec!["serde", "tokio"],
        );
    }

    #[test]
    fn the_dependent_count_combines_with_the_other_conditions() {
        let counts = dependents_of(&[500, 3, 900, 0]);
        assert_eq!(
            names_with(
                &["--min-dependents", "100", "--updated-after", "2024-03-01"],
                &counts,
            ),
            vec!["tokio"],
        );
    }

    #[test]
    fn without_counts_nothing_meets_a_count_condition() {
        assert!(names_of(&["--min-dependents", "1"]).is_empty());
        assert!(names_of(&["--min-downloads", "1"]).is_empty());
    }

    #[test]
    fn only_a_count_condition_needs_the_counts() {
        assert!(!filter_of(&[]).needs_dependents());
        assert!(!filter_of(&[]).needs_downloads());
        assert!(filter_of(&["--min-dependents", "1"]).needs_dependents());
        assert!(filter_of(&["--min-downloads", "1"]).needs_downloads());
    }

    #[test]
    fn the_limit_caps_the_listing() {
        assert_eq!(names_of(&["--limit", "2"]), vec!["serde", "serde_json"]);
        assert!(names_of(&["--limit", "0"]).is_empty());
    }

    #[test]
    fn the_limit_counts_crates_that_met_the_conditions() {
        assert_eq!(
            names_of(&["--name-contains", "serde", "--limit", "1"]),
            vec!["serde"],
        );
    }
}
