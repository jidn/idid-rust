use chrono::{DateTime, FixedOffset, NaiveDate};
use rev_lines::RevLines;
use std::fs::File;

const START_RECORDING: &'static str = "*~*~*--------------------";

pub struct Entry {
    pub begin: Option<DateTime<FixedOffset>>,
    pub cease: DateTime<FixedOffset>,
    pub text: String,
}

/// Convert a TSV line to an entry.
///
/// # Arguments
/// * line - A string with ISO 3339 date time and some text separated by a tab.
///
/// # Returns
/// An Entry with `cease` and `text`
///
impl Entry {
    pub fn parse(input: &str) -> Result<Entry, String> {
        // Split the input string by tab character
        let mut parts = input.split('\t');

        // Parse the timestamp part as ISO 3339
        let cease_str = parts.next().ok_or("Invalid TSV column 1")?;
        let cease = DateTime::parse_from_rfc3339(cease_str)
            .map_err(|e| format!("rfc3339 parse fail: {}", e))?
            .fixed_offset();

        // Get the text part and trim any newline characters
        let text = parts
            .next()
            .ok_or("Invalid TSV column 2")?
            .trim_end_matches('\n')
            .to_string();

        Ok(Self {
            begin: None,
            cease,
            text,
        })
    }
}

// class Entry:
//     """A recorded accomplishment entry."""
//
//     begin: datetime
//     cease: datetime
//     text: str
//     def duration(self) -> timedelta:
//         """Duration from begin to cease."""
//         return self.cease - self.begin

// fn pick(tsv_path: &str) {
//     let file = File::open(tsv_path).unwrap();
//     let rev_lines = RevLines::new(file);
//
//     let last_line
//     for line in rev_lines {
//         println!("{:?}", line);
//
//         if last_line.is_some() {
//             // process this line where begin is the last_line value
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util_time::{current_datetime_reset, current_datetime_set};
    use assert_approx_eq::assert_approx_eq;
    use rstest::rstest;

    #[test]
    fn test_entry_parse() {
        let input_str = "2024-04-01 12:15:30+05:00\tSome text with newline\n";
        match Entry::parse(input_str) {
            Ok(parsed) => {
                println!("Parsed cease={:?}, text={:?}", parsed.cease, parsed.text);
            }
            Err(err) => {
                println!("Parsing error: {:?}", err);
            }
        }
    }
}
