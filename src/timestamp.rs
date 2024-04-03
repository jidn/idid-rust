// class Entry:
//     """A recorded accomplishment entry."""
//
//     begin: datetime
//     cease: datetime
//     text: str
//     def duration(self) -> timedelta:
//         """Duration from begin to cease."""
//         return self.cease - self.begin

use crate::util_time::current_datetime;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveTime};

pub fn get_current_timestamp() -> String {
    // Get the current local time
    let local_time = current_datetime();

    // Format the local time in ISO 8601 format with timezone offset
    local_time.format("%Y-%m-%d %H:%M:%S%:z").to_string()
}

/// Parse an adjustment to the current local time.
///
/// # Arguments
///
/// * `input` - A optional string slice representing a time. Formats include:
///    - `MINUTES`, ie "30" minutes in the past.
///    - "HH:MM" ie "7:30" or "14:00" in 24 hour time.
///    - "HH[:MM](am|pm)" ie "8am", "8:15am", "1:30pm", or "5pm".
///
pub fn parse_time_adjustment(input: Option<&str>) -> Result<DateTime<FixedOffset>, String> {
    if input.is_none() {
        return Ok(current_datetime());
    }

    let input_str = input.unwrap();
    // Try parsing input as minutes in the past
    if let Ok(minutes) = input_str.parse::<i32>() {
        if minutes > 0 && minutes <= 1440 {
            let offset_time = current_datetime()
                .checked_sub_signed(Duration::minutes(minutes as i64))
                .unwrap();
            #[cfg(debug_assertions)]
            println!(
                "parse_time_adjutment minutes={}, current={}, computed={}, computed_timestamp={}",
                minutes,
                current_datetime(),
                offset_time,
                offset_time.timestamp() as f64
            );
            return Ok(offset_time);
        }
        return Err(format!("Invalid minutes \"{}\"", input_str));
    }

    // Parse as a given time, "HH:MM", "HH[:MM](am|pm)"
    let time_str = input_str
        .trim_end_matches("am")
        .trim_end_matches("pm")
        .trim();

    let digits_and_colon = time_str.chars().all(|c| c.is_digit(10) || c == ':');
    if !digits_and_colon {
        return Err("invalid HH[:MM](am|mm) format".to_string());
    }

    // Parse the HH[:MM]
    let mut hour = 0;
    let mut minute = 0;
    let parts: Vec<&str> = time_str.split(':').collect();
    match parts.len() {
        1 => {
            // parse [HH]
            hour = parts[0].parse::<u32>().expect("invalid HH(am|mm) format"); //ok()?;
        }
        2 => {
            // parse [HH, MM]
            hour = parts[0].parse::<u32>().expect("invalid hours");
            minute = parts[1].parse::<u32>().expect("invalid minutes");
        }
        _ => return Err("invalid HH[:MM](am|mm) format".to_string()),
    }

    if minute > 59 {
        return Err("invalid minutes".to_string());
    }
    if hour > 23 {
        return Err("invalid hours".to_string());
    }

    let pm = input_str.ends_with("pm");
    if hour > 11 && (pm || input_str.ends_with("am")) {
        let am_pm_str = if pm { "pm" } else { "am" };
        return Err(format!("invalid hours with \"{}\"", am_pm_str));
    } else if pm {
        hour += 12; // Convert to 24-hour format when using "pm"
    }

    Ok(local_timestamp(hour, minute))
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
pub fn parse_date(format: &str) -> Result<NaiveDate, String> {
    let now = current_datetime().date_naive();

    #[cfg(debug_assertions)]
    println!("parse_date(\"{}\", {})", format, now);

    // All days before today and YYYY-MM-DD variants
    if format.chars().all(|c| c.is_digit(10) || c == '-') {
        #[cfg(debug_assertions)]
        println!("parse_numeric_to_date");
        return parse_numeric_to_date(format, Some(now));
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
        _ => {
            #[cfg(debug_assertions)]
            println!("parse_last_dow(\"{}\",...)", &lower_case);
            parse_last_dow(&lower_case, Some(now))
        }
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
pub fn parse_numeric_to_date(
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
            let from_input = MonthDay::parse_from_str(&dashless)?;

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
            let from_input = MonthDay::parse_from_str(&month_day_str)?;

            NaiveDate::from_ymd_opt(year, from_input.month, from_input.day)
                .ok_or_else(|| format!("invalid date: {}", &input))
        }
        _ => Err(format!("invalid date: {}", input)),
    }
}

/// Get the last day of the week.
///
/// The input is expected to be lowercase.
fn parse_last_dow(input: &str, reference_date: Option<NaiveDate>) -> Result<NaiveDate, String> {
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
fn local_timestamp(hour: u32, minute: u32) -> DateTime<FixedOffset> {
    current_datetime()
        .with_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
        .unwrap()
}
pub struct MonthDay {
    pub month: u32,
    pub day: u32,
}
impl MonthDay {
    pub fn parse_from_str(input: &str) -> Result<Self, String> {
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
    use assert_approx_eq::assert_approx_eq;
    use rstest::rstest;

    fn set_current_datetime_to_april_1_2024() {
        current_datetime_set(DateTime::parse_from_rfc3339("2024-04-01T12:15:30+05:00").unwrap());
        println!("Set current_datetime to {}", current_datetime());
    }

    #[rstest]
    #[case("30", current_datetime() - Duration::minutes(30))]
    #[case("7:30", local_timestamp(7, 30))]
    #[case("7:30am", local_timestamp(7, 30))]
    #[case("7:30pm", local_timestamp(19, 30))]
    #[case("8am", local_timestamp(8, 0))]
    #[case("8pm", local_timestamp(20, 0))]
    fn test_parse_time_adjustment(#[case] input: &str, #[case] expected: DateTime<FixedOffset>) {
        match parse_time_adjustment(Some(input)) {
            Ok(actual) => {
                eprintln!(
                    "input=\"{}\", expected={}, actual={}",
                    input, expected, actual
                );
                assert_approx_eq!(expected.timestamp() as f64, actual.timestamp() as f64, 0.1);
            }
            Err(err) => {
                panic!(
                    "input=\"{}\", expected=\"{}\", error: {}",
                    input, expected, err
                );
            }
        }
    }

    #[rstest]
    #[case("0", "Invalid minutes \"0\"")]
    #[case("1441", "Invalid minutes \"1441\"")]
    #[case("invalid", "invalid HH[:MM](am|mm) format")]
    #[case("1:60", "invalid minutes")]
    #[case("24pm", "invalid hours")]
    #[case("13pm", "invalid hours with \"pm\"")]
    #[case("1jk", "invalid HH[:MM](am|mm) format")]
    #[case("1:30jk", "invalid HH[:MM](am|mm) format")]
    fn test_parse_time_adjustment_bad_input(#[case] input: &str, #[case] expected: &str) {
        match parse_time_adjustment(Some(input)) {
            Ok(_) => {
                // Test fails if the result is Ok (unexpected)
                panic!("Expected Err but got Ok");
            }
            Err(err) => {
                // Test passes if the result is Err and the error message matches the expected value
                assert_eq!(
                    expected, err,
                    "input=\"{}\" actual=\"{}\" expected=\"{}\"",
                    input, err, expected
                );
            }
        }
    }

    #[rstest] // parse_date relative to 2024-04-01
    #[case("0", 2024, 4, 1)] // today
    #[case("1", 2024, 3, 31)] // yesterday
    #[case("today", 2024, 4, 1)]
    #[case("yester", 2024, 3, 31)]
    #[case("2024-03-01", 2024, 3, 1)]
    #[case("fri", 2024, 3, 29)]
    fn test_parse_date(
        #[case] input: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
        let expected = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        set_current_datetime_to_april_1_2024();
        let parsed_date = parse_date(input);
        current_datetime_reset();
        match parsed_date {
            Ok(actual) => {
                println!(
                    "input=\"{}\", expected={}, actual={}",
                    input, expected, actual
                );
                assert_eq!(expected, actual);
            }
            Err(err) => {
                panic!(
                    "input=\"{}\", expected=\"{}\", error: {}",
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
    fn test_parse_numeric_date(
        #[case] input: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
        let reference_date = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let expected = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        match parse_numeric_to_date(input, Some(reference_date)) {
            Ok(actual) => {
                println!(
                    "input=\"{}\", expected={}, actual={}",
                    input, expected, actual
                );
                assert_eq!(expected, actual);
            }
            Err(err) => {
                panic!(
                    "input=\"{}\", expected=\"{}\", error: {}",
                    input, expected, err
                );
            }
        }
    }
    #[rstest] // parse_numeric_to_date with invalid input
    #[case("ABC", "invalid number of days past: ABC")]
    #[case("1000", "invalid day: 00")] // from MonthDay::parse_from_str()
    #[case("0432", "invalid day: 32")] // from MonthDate::parse_from_str()
    #[case("0030", "invalid month: 00")] // from MonthDay::parse_from_str()
    #[case("1330", "invalid month: 13")] // from MonthDay::parse_from_str()
    #[case("0230", "invalid date: 0230")] // errors in NativeTime::from_ymd_opt()
    #[case("0000-04-06", "invalid year: 0000")]
    #[case("12345", "invalid date: 12345")] // dashless length not handled
    fn test_parse_numeric_date_bad_input(#[case] input: &str, #[case] expected_error: &str) {
        match parse_numeric_to_date(input, None) {
            Ok(_) => {
                // Test fails if the result is Ok (unexpected)
                panic!("Expected Err but got Ok");
            }
            Err(err) => {
                // Test passes if the result is Err and the error message matches the expected value
                assert_eq!(
                    expected_error, err,
                    "input=\"{}\" actual=\"{}\" expected=\"{}\"",
                    input, err, expected_error
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
    fn test_parse_dow(#[case] input: &str, #[case] dom: u32) {
        set_current_datetime_to_april_1_2024(); // monday
        let actual = parse_date(input).unwrap();
        current_datetime_reset();
        let expected = NaiveDate::from_ymd_opt(2024, 3, dom).unwrap();
        assert_eq!(
            expected, actual,
            "input=\"{}\", expected=\"{}\", actual=\"{}\"",
            input, expected, actual
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_dow_bad_input() {
        let input = "xyz";
        match parse_date(input) {
            Ok(_) => {
                // Test fails if the result is Ok (unexpected)
                panic!("Expected Err but got Ok");
            }
            Err(err) => {
                // Test passes if the result is Err and the error message matches the expected value
                let expected =
                    "invalid day of the week abbreviation; use: mon, tue, wed, thu, fri, sat, sun";
                assert_eq!(
                    expected, err,
                    "input=\"{}\", expected=\"{}\", actual=\"{}\" ",
                    input, expected, err
                );
            }
        }
    }

    #[test]
    fn test_parse_dow_no_reference_date() {
        // Parse Tuesday without providing reference_date parameter
        let tuesday = parse_date("tue");
        // Expect Some date
        assert_eq!(tuesday.unwrap().weekday().num_days_from_monday(), 1);
    }
}
