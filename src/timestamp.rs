// class Entry:
//     """A recorded accomplishment entry."""
//
//     begin: datetime
//     cease: datetime
//     text: str
//     def duration(self) -> timedelta:
//         """Duration from begin to cease."""
//         return self.cease - self.begin

use chrono::{Datelike, Local, NaiveDate};

pub fn get_current_timestamp() -> String {
    // Get the current local time
    let local_time = Local::now();

    // Format the local time in ISO 8601 format with timezone offset
    local_time.format("%Y-%m-%d %H:%M:%S%:z").to_string()
}

/// Parse a date string
///
/// # Arguments
/// * `input` - A string slice representing a date in one of the following formats.
///    - `DAYS` Number of days in the past; zero is today and one is yesterday.
///    - "today" or "yesterday" string literal.
///    - "YYYY-MM-DD" or "MM-DD' within the last 364 days; dash optional.
///    - The last day-of-the-week. A suffix adds weeks.  
///      "mon" and "mon0" mean the same and "mon2" adds two additional weeks.
///
/// * `date` - An optional `chrono::NaiveDate` representing a reference date.
///
/// # Returns
///
/// Returns an `Option<chrono::NaiveDate>` representing the parsed date if successful,
/// or `None` if the input string is not in one of the supported formats.
///
/// # Panics
///
/// This function does not panic under normal circumstances.
/// However, if the input string represents a date outside the valid range of `NaiveDate`,
/// it may panic when trying to create the date.
pub fn parse_date(input: &str, reference_date: Option<NaiveDate>) -> Option<NaiveDate> {
    let today = Some(reference_date.unwrap_or_else(|| local_naive_date()));

    if input.chars().all(|c| c.is_digit(10) || c == '-') {
        return parse_numeric_date(input, today);
    }

    let lower_case = input.to_lowercase();
    if lower_case.starts_with("yester") {
        return today?.checked_sub_signed(chrono::Duration::days(1));
    }

    match input {
        "today" => today,
        _ => parse_last_dow(&lower_case, today),
    }
}

/// Get the last day of the week.
///
/// The input is expected to be lowercase.
fn parse_last_dow(input: &str, reference_date: Option<NaiveDate>) -> Option<NaiveDate> {
    // let lowercase_input = input.to_lowercase();
    let day_of_week: &str = &input[..3];

    // Calculate the target day of the week
    let target_day = match day_of_week {
        "mon" => 0,
        "tue" => 1,
        "wed" => 2,
        "thu" => 3,
        "fri" => 4,
        "sat" => 5,
        "sun" => 6,
        _ => return None, // Invalid day of the week abbreviation
    };

    let mut weeks_ago: u32 = if input.len() > 3 {
        input[3..].parse().unwrap_or(0) // Default to 0 if parsing fails
    } else {
        0
    };

    // Calculate the day of the week based on today's date
    let today = reference_date.unwrap_or(local_naive_date());
    // 0 - Monday, 1 - Tuesday, ..., 6 - Sunday
    let days_from_monday = today.weekday().num_days_from_monday();
    if target_day == days_from_monday {
        // if today is WED and parsing "wed" then we want to back one week
        weeks_ago += 1;
    }

    // Calculate the difference in days to the target day of the week
    let days_ago = (days_from_monday + 7 - target_day) % 7 + weeks_ago * 7;

    // Calculate the target date
    Some(today - chrono::Duration::days(days_ago.into()))
}

/// Parse a date string
///
/// # Arguments
/// * `input` - A string slice representing a date in one of the following formats.
///    - `DAYS` Number of days in the past; zero is today and one is yesterday.
///    - "YYYY-MM-DD" or "MM-DD' within the last 364 days; dash optional.
/// * `date` - An optional `chrono::NaiveDate` representing a reference date.
///
/// # Returns
///
/// Returns an `Option<chrono::NaiveDate>` representing the parsed date if successful,
/// or `None` if the input string is not in one of the supported formats.
///
/// # Panics
///
/// When input string represents a date outside the valid range of `NaiveDate`.
pub fn parse_numeric_date(input: &str, reference_date: Option<NaiveDate>) -> Option<NaiveDate> {
    let today = reference_date.unwrap_or_else(|| local_naive_date());

    // remove any dashes
    let dashless = input.replace("-", "");

    match dashless.len() {
        1 | 2 | 3 => {
            println!("DAYS {dashless}");
            // Interpret input as number of days before the reference date
            let days_before = dashless.parse::<i64>().ok()?;
            today.checked_sub_signed(chrono::Duration::days(days_before))
        }
        4 => {
            // Parse "MM-DD" or "MMDD"
            println!("MM-DD {dashless}");
            let year = today.year();
            let month = dashless[0..2].parse::<u32>().ok()?;
            let day = dashless[2..4].parse::<u32>().ok()?;

            if month < 1 || month > 12 || day < 1 || day > 31 {
                return None;
            }

            let computed = NaiveDate::from_ymd_opt(year, month, day);
            match computed {
                None => None,
                Some(date) if date < today => computed, // The current year
                _ => NaiveDate::from_ymd_opt(year - 1, month, day),
            }
        }
        6 => {
            // Parse "YY-MM-DD" or "YYMMDD"
            println!("YY-MM-DD {dashless}");
            let year = 2000 + dashless[0..2].parse::<i32>().ok()?;
            let month = dashless[2..4].parse::<u32>().ok()?;
            let day = dashless[4..6].parse::<u32>().ok()?;

            if year < 1 || month < 1 || month > 12 || day < 1 || day > 31 {
                return None;
            }
            NaiveDate::from_ymd_opt(year, month, day)
        }
        8 => {
            // Parse "YYYY-MM-DD" or "YYYYMMDD"
            println!("YYYY-MM-DD {dashless}");
            let year = dashless[0..4].parse::<i32>().ok()?;
            let month = dashless[4..6].parse::<u32>().ok()?;
            let day = dashless[6..8].parse::<u32>().ok()?;

            if year < 1 || month < 1 || month > 12 || day < 1 || day > 31 {
                return None;
            }
            NaiveDate::from_ymd_opt(year, month, day)
        }
        _ => None,
    }
}

fn local_naive_date() -> NaiveDate {
    Local::now().date_naive()
}

pub fn write_new_entry(text: &str) {
    println!("{}\t{}", get_current_timestamp(), text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_parse_yy_mm_dd() {
        assert_eq!(
            parse_date("24-01-15", None),
            NaiveDate::from_ymd_opt(2024, 1, 15)
        );
    }

    #[test]
    fn test_parse_yymmdd() {
        assert_eq!(
            parse_date("240115", None),
            NaiveDate::from_ymd_opt(2024, 1, 15)
        );
    }

    #[test]
    fn test_parse_mm_dd() {
        let reference_date = NaiveDate::from_ymd_opt(2024, 4, 1);
        assert_eq!(
            parse_date("02-01", reference_date),
            NaiveDate::from_ymd_opt(2024, 02, 01)
        );
        assert_eq!(
            parse_date("06-01", reference_date),
            NaiveDate::from_ymd_opt(2023, 06, 01)
        );
    }

    #[test]
    fn test_parse_mmdd() {
        let reference_date = NaiveDate::from_ymd_opt(2024, 4, 1);
        assert_eq!(
            parse_date("02-01", reference_date),
            NaiveDate::from_ymd_opt(2024, 02, 01)
        );
        assert_eq!(
            parse_date("1101", reference_date),
            NaiveDate::from_ymd_opt(2023, 11, 01)
        );
    }

    #[test]
    fn test_parse_days_before() {
        let reference_date = NaiveDate::from_ymd_opt(2023, 1, 1);
        assert_eq!(
            parse_date("30", reference_date),
            NaiveDate::from_ymd_opt(2022, 12, 2)
        );
    }

    #[test]
    fn test_invalid_date_format() {
        let reference_date = NaiveDate::from_ymd_opt(2023, 1, 1);
        assert_eq!(parse_date("22-13-01", reference_date), None);
        assert_eq!(parse_date("02-30", reference_date), None);
    }

    #[rstest]
    #[case("sun0", 31)]
    #[case("sun1", 24)]
    #[case("mon0", 25)]
    #[case("mon1", 18)]
    #[case("mon2", 11)]
    #[case("tue0", 26)]
    #[case("tue1", 19)]
    #[case("tue2", 12)]
    fn test_parse_dow_weeks_ago(#[case] input: &str, #[case] dom: u32) {
        // Given today is Monday, April 1, 2024
        let today = NaiveDate::from_ymd_opt(2024, 4, 1);

        let computed = parse_date(input, today);
        assert_eq!(computed, NaiveDate::from_ymd_opt(2024, 3, dom));
    }

    #[test]
    fn test_parse_dow_invalid_input() {
        let invalid_input = parse_date("xyz", None);
        // Expect None
        assert_eq!(invalid_input, None);
    }

    #[test]
    fn test_parse_dow_no_reference_parameter() {
        // Parse Tuesday without providing reference_date parameter
        let tuesday = parse_date("tue", None);
        // Expect Some date
        assert_eq!(tuesday.unwrap().weekday().num_days_from_monday(), 1);
    }
}
