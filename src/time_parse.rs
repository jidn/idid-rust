use crate::util_time::current_datetime;
use chrono::{DateTime, Duration, FixedOffset, NaiveTime};

/// Parse an adjustment to the current local time.
///
/// # Arguments
///
///    - `MINUTES`, ie "30" minutes in the past.
///    - "HH:MM" ie "7:30" or "14:00" in 24 hour time.
///    - "HH[:MM](am|pm)" ie "8am", "8:15am", "1:30pm", or "5pm".
///
pub fn time_adjustment(input: Option<&str>) -> Result<DateTime<FixedOffset>, String> {
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
                "time_adjustment minutes={}, current={}, computed={}, computed_timestamp={}",
                minutes,
                current_datetime(),
                offset_time,
                offset_time.timestamp() as f64
            );
            return Ok(offset_time);
        }
        return Err(format!("Invalid minutes {:?}", input_str));
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
    let mut _hour = 0;
    let mut minute = 0;
    let parts: Vec<&str> = time_str.split(':').collect();
    match parts.len() {
        1 => {
            // parse [HH]
            _hour = parts[0].parse::<u32>().expect("invalid HH(am|mm) format"); //ok()?;
        }
        2 => {
            // parse [HH, MM]
            _hour = parts[0].parse::<u32>().expect("invalid hours");
            minute = parts[1].parse::<u32>().expect("invalid minutes");
        }
        _ => return Err("invalid HH[:MM](am|mm) format".to_string()),
    }

    if minute > 59 {
        return Err("invalid minutes".to_string());
    }
    if _hour > 23 {
        return Err("invalid hours".to_string());
    }

    let pm = input_str.ends_with("pm");
    if _hour > 11 && (pm || input_str.ends_with("am")) {
        let am_pm_str = if pm { "pm" } else { "am" };
        return Err(format!("invalid hours with {:?}", am_pm_str));
    } else if pm {
        _hour += 12; // Convert to 24-hour format when using "pm"
    }

    Ok(local_timestamp(_hour, minute))
}

fn local_timestamp(hour: u32, minute: u32) -> DateTime<FixedOffset> {
    current_datetime()
        .with_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util_time::{current_datetime_reset, current_datetime_set};
    use assert_approx_eq::assert_approx_eq;
    use rstest::rstest;

    #[rstest]
    #[case("30", current_datetime() - Duration::minutes(30))]
    #[case("7:30", local_timestamp(7, 30))]
    #[case("7:30am", local_timestamp(7, 30))]
    #[case("7:30pm", local_timestamp(19, 30))]
    #[case("8am", local_timestamp(8, 0))]
    #[case("8pm", local_timestamp(20, 0))]
    fn test_time_adjustment(#[case] input: &str, #[case] expected: DateTime<FixedOffset>) {
        match time_adjustment(Some(input)) {
            Ok(actual) => {
                eprintln!(
                    "input={:?}, expected={:?}, actual={:?}",
                    input, expected, actual
                );
                assert_approx_eq!(expected.timestamp() as f64, actual.timestamp() as f64, 0.1);
            }
            Err(err) => {
                panic!(
                    "input={:?}, expected={:?}, erroe={:?}",
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
    fn test_time_adjustment_bad_input(#[case] input: &str, #[case] expected: &str) {
        match time_adjustment(Some(input)) {
            Ok(_) => {
                // Test fails if the result is Ok (unexpected)
                panic!("Expected Err but got Ok");
            }
            Err(err) => {
                // Test passes if the result is Err and the error message matches the expected value
                assert_eq!(
                    expected, err,
                    "input={:?} actual={:?} expected={:?}",
                    input, err, expected
                );
            }
        }
    }

    #[test]
    fn test_time_adjustment_none() {
        let set_time = DateTime::parse_from_rfc3339("2024-04-01T12:15:30+05:00").unwrap();
        current_datetime_set(set_time);

        // When I give none, I should give the current time.
        let found_datetime = time_adjustment(None).expect("Should be current time.");
        current_datetime_reset();

        assert_eq!(set_time, found_datetime);
    }
}
