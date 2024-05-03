use crate::util_time::current_datetime;
use chrono::{Datelike, Duration, NaiveDate};

/// Create a vector of NaiveDate from strings
///
/// See date_from_str for format details.
pub fn strings_to_dates(dates: &Option<Vec<String>>) -> Result<Vec<NaiveDate>, String> {
    // Process dates and ranges using str_to_date
    let mut parsed: Vec<NaiveDate> = Vec::new();
    if let Some(vec) = dates {
        for date_str in vec {
            match date_from_str(&date_str) {
                Ok(date) => parsed.push(date),
                Err(err) => return Err(format!("Invalid {}: {}", date_str, err)),
            }
        }
    }
    Ok(parsed)
}

/// Create a date relative to the current date and time.
///
/// # Arguments
///
/// * `format` - A string slice representing a date format.
///    - Days before today. 0 is today and 1 is yesterday; max 999.
///    - Literal "today" or "yesterday".
///    - YYYYMMDD (dashes optinal).
///    - YYMMDD (leading 0, dashes optional) starting in the year 2000.
///    - Last MMDD (leading 0, dashes optinal); within 364ish days.
///    - Last week day with optional numeric suffix to add weeks.
///      "mon" is last Monday and "mon1" goes back an additional week.
///
/// # Panics
///
/// This function does not panic under normal circumstances.
/// However, if the input string represents a date outside the valid range of
/// `NaiveDate`, it may panic when trying to create the date.
pub fn date_from_str(format: &str) -> Result<NaiveDate, String> {
    let now = current_datetime().date_naive();

    #[cfg(debug_assertions)]
    println!("format={:?}, now={}", format, now);

    // All days before today and YYYY-MM-DD variants
    if format.chars().all(|c| c.is_digit(10) || c == '-') {
        let value = numeric_to_date(format, Some(now));
        #[cfg(debug_assertions)]
        println!("numeric_to_date: {:?}", value.as_ref().expect("date"));
        return value;
    }

    // Instead of forcing the full "yesterday", allow anything that starts
    // with "yester", like "yesternight".  This one is for you Cameron.
    let lower_case = format.to_lowercase();
    if lower_case.starts_with("yester") {
        #[cfg(debug_assertions)]
        println!("Working on 'yester'");
        return now
            .checked_sub_signed(chrono::Duration::days(1))
            .ok_or_else(|| format!("unable to get {} ", lower_case));
    }

    match format {
        "today" => Ok(now),
        _ => last_dow(&lower_case, Some(now)),
    }
}

/// Parse a string to a date
///
/// # Arguments
///
/// * `input` - A string slice representing a date in one of the following formats.
///  -  Days before today. 0 is today and 1 is yesterday; max 999.
///  -  YYYYMMDD (dashes optinal).
///  -  YYMMDD (leading 0, dashes optional) starting in the year 2000.
///  -  Last MMDD (leading 0, dashes optinal); within 364ish days.
///
/// * `reference_date` - An optional `chrono::NaiveDate` representing a reference date.
///
/// # Returns
///
/// Returns a `Result<chrono::NaiveDate, String>` with the expected NaiveDate
/// or the reason why it couldn't parse the input.
///
/// # Panics
///
/// When input string represents a date outside the valid range of `NaiveDate`.
pub fn numeric_to_date(
    input: &str,
    reference_date: Option<NaiveDate>,
) -> Result<NaiveDate, String> {
    let today = reference_date.unwrap_or_else(|| local_naive_date());

    // remove any dashes
    let dashless = input.replace("-", "");

    match dashless.len() {
        1 | 2 | 3 => {
            // Interpret input as number of days before the reference date
            let days_before = dashless
                .parse::<i64>()
                .map_err(|_| format!("invalid number of days past: {}", dashless))?;
            today
                .checked_sub_signed(Duration::days(days_before))
                .ok_or_else(|| format!("Unable to get {} previous days.", days_before))
        }
        4 => {
            // Parse "MM-DD" or "MMDD"
            let year = today.year();
            let from_input = MonthDay::from_str(&dashless)?;

            let computed = NaiveDate::from_ymd_opt(year, from_input.month, from_input.day)
                .ok_or_else(|| format!("invalid date: {}", &input))?;
            if computed < today {
                Ok(computed)
            } else {
                Ok(NaiveDate::from_ymd_opt(year - 1, from_input.month, from_input.day).unwrap())
            }
        }
        6 | 8 => {
            // Parse input of either YY-MM-DD or YYYY-MM-DD
            let (year_str, month_day_str) = match dashless.len() {
                6 => (format!("20{}", &dashless[0..2]), &dashless[2..6]),
                8 => (dashless[0..4].to_string(), &dashless[4..8]),
                _ => return Err("Invalid input length.".to_string()),
            };
            let year = year_str.parse::<i32>().map_err(|e| e.to_string())?;
            if year < 1 {
                return Err(format!("invalid year: {}", year_str));
            }
            let from_input = MonthDay::from_str(&month_day_str)?;

            NaiveDate::from_ymd_opt(year, from_input.month, from_input.day)
                .ok_or_else(|| format!("invalid date: {}", &input))
        }
        _ => Err(format!("invalid date: {}", input)),
    }
}

/// Get the last day of the week.
///
/// The input is expected to be lowercase.
fn last_dow(input: &str, reference_date: Option<NaiveDate>) -> Result<NaiveDate, String> {
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
        _ => {
            return Err(
                "invalid day of the week abbreviation; use: mon, tue, wed, thu, fri, sat, sun"
                    .to_string(),
            )
        } // Return an error for invalid day abbreviation
    };

    // Get the number of additional weeks or 0 if not given
    let mut weeks_ago: u32 = if input.len() > 3 {
        input[3..]
            .parse()
            .map_err(|e| format!("failed to parse weeks: {}", e))? // Propagate parsing error as a string
    } else {
        0
    };

    // Calculate the day of the week based on today's date
    let today = reference_date.unwrap_or_else(|| local_naive_date());
    // 0 - Monday, 1 - Tuesday, ..., 6 - Sunday
    let days_from_monday = today.weekday().num_days_from_monday();
    if target_day == days_from_monday {
        // if today is WED and parsing "wed" then we want to go back one week
        weeks_ago += 1;
    }

    // Calculate the difference in days to the target day of the week
    let days_ago = (days_from_monday + 7 - target_day) % 7 + weeks_ago * 7;

    // Calculate the target date
    Ok(today - Duration::days(days_ago.into()))
}

fn local_naive_date() -> NaiveDate {
    current_datetime().date_naive()
}

pub struct MonthDay {
    pub month: u32,
    pub day: u32,
}
impl MonthDay {
    pub fn from_str(input: &str) -> Result<Self, String> {
        let month = input[0..2]
            .parse::<u32>()
            .map_err(|e| format!("Invalid month: {}", e))?;
        let day = input[2..4]
            .parse::<u32>()
            .map_err(|e| format!("invalid day: {}", e))?;

        if month < 1 || month > 12 {
            return Err(format!("invalid month: {}", &input[0..2]));
        } else if day < 1 || day > 31 {
            return Err(format!("invalid day: {}", &input[2..4]));
        }

        Ok(MonthDay { month, day })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util_time::{current_datetime_reset, current_datetime_set};
    use chrono::DateTime;
    use rstest::rstest;

    fn set_current_datetime_to_april_1_2024() {
        current_datetime_set(DateTime::parse_from_rfc3339("2024-04-01T12:15:30+05:00").unwrap());
        println!("Set current_datetime to {}", current_datetime());
    }

    #[rstest] // parse date relative to 2024-04-01
    #[case("0", 2024, 4, 1)] // today
    #[case("1", 2024, 3, 31)] // yesterday
    #[case("today", 2024, 4, 1)]
    #[case("yester", 2024, 3, 31)]
    #[case("2024-03-01", 2024, 3, 1)]
    #[case("fri", 2024, 3, 29)]
    fn test_date_parse(
        #[case] input: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
        let expected = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        set_current_datetime_to_april_1_2024();
        let parsed_date = date_from_str(input);
        current_datetime_reset();
        match parsed_date {
            Ok(actual) => {
                println!(
                    "input={:?}, expected={:?}, actual={:?}",
                    input, expected, actual
                );
                assert_eq!(expected, actual);
            }
            Err(err) => {
                panic!(
                    "input={:?}, expected={:?}, error={:?}",
                    input, expected, err
                );
            }
        }
    }
    #[rstest] // parse_numeric_to_date relative to 2024-04-01
    #[case("2", 2024, 3, 30)] // two days ago
    #[case("999", 2021, 7, 7)] // 999 days ago
    #[case("0402", 2023, 4, 2)] // last April 2
    #[case("04-02", 2023, 4, 2)] // same with dashes
    #[case("240401", 2024, 4, 1)] // 2024-04-01
    #[case("2024-04-01", 2024, 4, 1)] // 2024-04-01
    fn test_date_parse_numeric(
        #[case] input: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
        let reference_date = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let expected = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        match numeric_to_date(input, Some(reference_date)) {
            Ok(actual) => {
                println!(
                    "input={:?}, expected={:?}, actual={:?}",
                    input, expected, actual
                );
                assert_eq!(expected, actual);
            }
            Err(err) => {
                panic!(
                    "input={:?}, expected={:?}, error={:?}",
                    input, expected, err
                );
            }
        }
    }

    #[rstest] // numeric_to_date with invalid input
    #[case("ABC", "invalid number of days past: ABC")]
    #[case("1000", "invalid day: 00")]
    #[case("0432", "invalid day: 32")]
    #[case("0030", "invalid month: 00")]
    #[case("1330", "invalid month: 13")]
    #[case("0230", "invalid date: 0230")]
    #[case("0000-04-06", "invalid year: 0000")]
    #[case("12345", "invalid date: 12345")]
    fn test_date_parse_numeric_bad_input(#[case] input: &str, #[case] expected_error: &str) {
        match numeric_to_date(input, None) {
            Ok(_) => {
                // Test fails if the result is Ok (unexpected)
                panic!("Expected Err but got Ok");
            }
            Err(err) => {
                // Test passes if the result is Err and the error message matches the expected value
                assert_eq!(
                    expected_error, err,
                    "input={:?}, expected={:?}, actual={:?}",
                    input, expected_error, err
                );
            }
        }
    }

    #[rstest]
    #[case("sun", 31)]
    #[case("sun1", 24)]
    #[case("mon", 25)]
    #[case("mon0", 25)]
    #[case("mon1", 18)]
    #[case("tue0", 26)]
    #[case("tue1", 19)]
    fn test_date_parse_dow(#[case] input: &str, #[case] dom: u32) {
        set_current_datetime_to_april_1_2024(); // monday
        let actual = date_from_str(input).unwrap();
        current_datetime_reset();
        let expected = NaiveDate::from_ymd_opt(2024, 3, dom).unwrap();
        assert_eq!(
            expected, actual,
            "input={:?}, expected={:?}, actual={:?}",
            input, expected, actual
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_date_parse_dow_bad_input() {
        assert_eq!(
            date_from_str("xyz"),
            Err(
                "invalid day of the week abbreviation; use: mon, tue, wed, thu, fri, sat, sun"
                    .to_string()
            )
        );
    }
}
