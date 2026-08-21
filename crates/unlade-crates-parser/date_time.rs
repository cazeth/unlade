use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;
use thiserror::Error;

/// A datetime field the dump could not have produced.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InvalidDateTime {
    /// The field does not hold a datetime.
    #[error("`{text}` is not a datetime of the form `YYYY-MM-DD HH:MM:SS`")]
    Malformed {
        /// The text that failed to parse.
        text: String,
        /// The underlying failure.
        #[source]
        source: jiff::Error,
    },

    /// The field stops before the time of day.
    #[error("`{text}` has no time of day")]
    Incomplete {
        /// The text that failed to parse.
        text: String,
    },

    /// The field carries an offset other than UTC.
    #[error("`{text}` is not in UTC")]
    NotUtc {
        /// The text that failed to parse.
        text: String,
    },
}

impl InvalidDateTime {
    /// Returns the text that failed to parse.
    pub fn text(&self) -> &str {
        match self {
            Self::Malformed { text, .. } | Self::Incomplete { text } | Self::NotUtc { text } => {
                text
            }
        }
    }
}

/// The shortest datetime the dump writes, `YYYY-MM-DD HH:MM:SS`.
const CIVIL_LENGTH: usize = 19;

const UTC_OFFSETS: [&str; 4] = ["Z", "z", "+00", "+00:00"];

/// Parses a datetime as written in the crates.io dump.
///
/// Values are UTC, may carry a fractional part, and may carry a trailing UTC
/// offset.
pub fn parse(text: &str) -> Result<Timestamp, InvalidDateTime> {
    let civil = civil_part(text)?;

    parse_utc(civil).map_err(|source| InvalidDateTime::Malformed {
        text: text.to_owned(),
        source,
    })
}

/// The datetime without its offset, once the offset is known to be UTC and the
/// datetime is known to reach the seconds field.
fn civil_part(text: &str) -> Result<&str, InvalidDateTime> {
    let (civil, offset) = split_offset(text);

    if !offset.is_empty() && !UTC_OFFSETS.contains(&offset) {
        return Err(InvalidDateTime::NotUtc {
            text: text.to_owned(),
        });
    }

    if civil.len() < CIVIL_LENGTH {
        return Err(InvalidDateTime::Incomplete {
            text: text.to_owned(),
        });
    }

    Ok(civil)
}

fn parse_utc(civil: &str) -> Result<Timestamp, jiff::Error> {
    let date_time: DateTime = civil.parse()?;
    date_time
        .to_zoned(TimeZone::UTC)
        .map(|zoned| zoned.timestamp())
}

/// Splits a datetime from a trailing offset.
///
/// The offset is located past the civil part, where the only `-` separators of
/// the date cannot be mistaken for one.
fn split_offset(text: &str) -> (&str, &str) {
    let Some(tail) = text.get(CIVIL_LENGTH..) else {
        return (text, "");
    };

    match tail.find(['+', '-', 'Z', 'z']) {
        Some(position) => text.split_at(CIVIL_LENGTH + position),
        None => (text, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seconds_of(text: &str) -> Option<i64> {
        parse(text).ok().map(Timestamp::as_second)
    }

    fn is_rejected(text: &str) -> bool {
        parse(text).is_err()
    }

    #[test]
    fn the_epoch_is_zero_seconds() {
        assert_eq!(seconds_of("1970-01-01 00:00:00"), Some(0));
    }

    #[test]
    fn datetimes_convert_to_known_epoch_values() {
        assert_eq!(seconds_of("2020-01-01 12:11:10"), Some(1_577_880_670));
        assert_eq!(seconds_of("2014-11-10 21:37:00"), Some(1_415_655_420));
        assert_eq!(seconds_of("2026-05-04 00:00:01"), Some(1_777_852_801));
    }

    #[test]
    fn leap_days_are_counted() {
        assert_eq!(seconds_of("2000-02-29 00:00:00"), Some(951_782_400));
    }

    #[test]
    fn fractional_seconds_are_kept() {
        let value = parse("2020-01-01 12:11:10.999999").expect("parses");
        assert_eq!(value.subsec_nanosecond(), 999_999_000);
    }

    #[test]
    fn a_trailing_utc_offset_is_accepted() {
        assert_eq!(
            seconds_of("2020-01-01 12:11:10.999999+00"),
            seconds_of("2020-01-01 12:11:10"),
        );
        assert_eq!(
            seconds_of("2020-01-01 12:11:10+00:00"),
            seconds_of("2020-01-01 12:11:10"),
        );
        assert_eq!(
            seconds_of("2020-01-01T12:11:10Z"),
            seconds_of("2020-01-01 12:11:10"),
        );
    }

    #[test]
    fn offsets_other_than_utc_are_rejected() {
        assert!(matches!(
            parse("2020-01-01 12:11:10+05"),
            Err(InvalidDateTime::NotUtc { .. }),
        ));
        assert!(is_rejected("2020-01-01 12:11:10-08:00"));
    }

    #[test]
    fn dates_before_the_epoch_are_read() {
        assert_eq!(seconds_of("1969-12-31 23:59:59"), Some(-1));
    }

    #[test]
    fn days_outside_the_month_are_rejected() {
        assert!(is_rejected("2021-02-29 00:00:00"));
        assert!(is_rejected("2021-04-31 00:00:00"));
        assert!(is_rejected("2021-01-00 00:00:00"));
        assert!(is_rejected("2021-13-01 00:00:00"));
    }

    #[test]
    fn times_outside_the_day_are_rejected() {
        assert!(is_rejected("2021-01-01 24:00:00"));
        assert!(is_rejected("2021-01-01 00:60:00"));
    }

    #[test]
    fn a_leap_second_is_read_as_the_last_second_of_the_minute() {
        assert_eq!(
            seconds_of("2016-12-31 23:59:60"),
            seconds_of("2016-12-31 23:59:59"),
        );
    }

    #[test]
    fn a_date_without_a_time_is_rejected() {
        assert!(matches!(
            parse("2021-01-01"),
            Err(InvalidDateTime::Incomplete { .. }),
        ));
        assert!(is_rejected("2021-01-01 00:00"));
    }

    #[test]
    fn malformed_fields_are_rejected() {
        assert!(is_rejected(""));
        assert!(is_rejected("2021-1-1 00:00:00"));
        assert!(is_rejected("2021-01-01 00:00:00."));
        assert!(is_rejected("2021-01-01 00:00:00.abc"));
        assert!(is_rejected("not a date at all"));
    }

    #[test]
    fn the_reported_text_is_the_offending_field() {
        let error = parse("not a date at all").unwrap_err();
        assert_eq!(error.text(), "not a date at all");
    }
}
