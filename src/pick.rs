use crate::date_filter::DateFilter;
use chrono::{DateTime, FixedOffset};
use rev_lines::RevLines;
use std::fmt;
use std::fs;
use std::io;

pub const START_RECORDING: &'static str = "*~*~*--------------------";

#[derive(Clone)]
pub struct Entry {
    pub begin: DateTime<FixedOffset>,
    pub cease: DateTime<FixedOffset>,
    pub text: String,
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Entry {{ begin: {}, cease: {}, text: {:?} }}",
            self.begin.to_rfc3339(),
            self.cease.to_rfc3339(),
            self.text
        )
    }
}

impl Entry {
    /// Each TSV line is timestamp and description of something done.
    fn from_tsv(line: &str) -> Result<(chrono::DateTime<FixedOffset>, String), String> {
        let mut parts = line.splitn(2, '\t');
        let when_str = parts.next().expect("TSV line.");
        let when = DateTime::parse_from_rfc3339(when_str).expect("rfc3339");
        let text = parts.next().unwrap_or_default().to_string();
        Ok((when, text))
    }
    pub fn is_in(&self, filter: &DateFilter) -> bool {
        filter.contains(&self.begin.date_naive())
    }
}

//  -------- iterator for parsing TSV
struct EntryIterator<F, R>
where
    F: FnMut(&Entry) -> bool,
    R: io::BufRead + io::Seek,
{
    // Search the file in reverse order
    lines: RevLines<R>,

    // Usually DateFilter.contains(&Entry)->bool
    filter: F,

    // Usually DateFilter.oldest_date: Option(NaiveDate)
    oldest: Option<chrono::NaiveDate>,

    // rfc 3339\tText for what I did at this time
    prev_line: Option<(DateTime<FixedOffset>, String)>,
}

impl<F, R> Iterator for EntryIterator<F, R>
where
    F: FnMut(&Entry) -> bool,
    R: io::BufRead + io::Seek,
{
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = self.lines.next() {
            let line = line.unwrap().trim().to_string();
            println!("line={:?}", line);

            // Each TSV line is timestamp and description of something done
            let (when, text) = Entry::from_tsv(&line).unwrap();

            // Older than oldest? Nothing more to find.
            if self.oldest.is_some() && when.date_naive() >= self.oldest? {
                return None;
            }

            // Create entry from previous line
            let some_entry = match self.prev_line {
                Some((ref prev_cease, ref prev_text)) => Some(Entry {
                    begin: when,
                    cease: *prev_cease,
                    text: prev_text.clone(),
                }),
                None => None,
            };

            // Keep the current line for the next line
            self.prev_line = match text.starts_with(START_RECORDING) {
                true => None,
                false => Some((when, text)),
            };
            print!("Entry={:?}, prev_line={:?}", some_entry, self.prev_line);

            // Is this an acceptible entry?
            if some_entry.is_some() {
                let entry = some_entry.unwrap();

                if (self.filter)(&entry) {
                    return Some(entry);
                }
            }
        }
        None
    }
}

//  -------- Helpers to create EntryIterator
// use io::{BufRead, BufReader, Cursor, Seek};

// trait BufReadRev: BufRead + Seek {}
// impl<T: BufRead + Seek> BufReadRev for T {}
//
// fn pick(
//     input: impl Into<String>,
//     filter: &DateFilter,
// ) -> EntryIterator<impl FnMut(&Entry) -> bool, impl BufReadRev> {
//     let input_str = input.into();
//     let buf_reader: Box<dyn BufReadRev> = if input_str.contains('\t') {
//         Box::new(Cursor::new(input_str))
//     } else {
//         let file = fs::File::open(&input_str).expect("Failed to open file");
//         Box::new(BufReader::new(file))
//     };
//
//     let filter_func = |entry: &Entry| filter.contains(&entry.begin.date_naive());
//     // let filter_func = |entry: &Entry| entry.is_in(&filter);
//
//     EntryIterator {
//         lines: RevLines::new(buf_reader),
//         filter: filter_func,
//         oldest: filter.oldest_date,
//         prev_line: None,
//     }
// }

// enum EntryReader {
//     Stdin(io::BufReader<io::Stdio>),
//     File(io::BufReader<fs::File>),
//     String(io::BufReader<io::Cursor<Vec<u8>>>),
// }
//
// impl EntryReader {
//     fn from_string(text: &str) -> Self {
//         let mut bytes = Vec::new();
//         bytes.extend_from_slice(text.as_bytes());
//         EntryReader::String(io::BufReader::new(io::Cursor::new(bytes)))
//     }
// }

// pub fn entries_from<F>(
//     filename: &str,
//     filter: F,
//     oldest: Option<DateTime<FixedOffset>>,
//     // ) -> Result<impl Iterator<Item = Result<Entry, Box<dyn Error>>>, Box<dyn Error>>
// ) -> impl Iterator<Item = Entry>
// where
//     F: FnMut(&Entry) -> bool,
// {
//     // Create a variable has the right type of BufReader
//     let reader = match filename {
//         "-" => EntryReader::Stdin(BufReader::new(std::io::stdin())),
//         _ => {
//             if let Ok(metadata) = fs::metadata(filename) {
//                 if metadata.is_file() {
//                     EntryReader::File(BufReader::new(fs::File::open(filename)?));
//                 }
//             }
//             // Not '-' and not a valid file, treat as string content
//             EntryReader::from_string(&filename)
//         }
//     };
//     let lines = RevLines::new(&reader);
//     Ok(EntryIterator {
//         lines,
//         filter,
//         oldest,
//         prev_line: None,
//     })
// }

// pub fn get_entries_from<F, R>(
//     filename: &str,
//     filter: F,
//     oldest: Option<DateTime<FixedOffset>>,
// ) -> Result<impl Iterator<Item = Result<Entry, Box<dyn Error>>>, Box<dyn Error>>
// where
//     F: FnMut(&Entry) -> bool,
//     R: io::BufRead,
// {
//     let reader = match filename {
//         "-" => EntryReader::Stdin(BufReader::new(std::io::stdin())),
//         _ => {
//             if let Ok(metadata) = fs::metadata(filename) {
//                 if metadata.is_file() {
//                     EntryReader::File(BufReader::new(fs::File::open(filename)?));
//                 }
//             }
//             // Not '-' and not a valid file, treat as string content
//             EntryReader::from_string(&filename)
//         }
//     };
//     let lines = RevLines::new(&reader);
//     Ok(EntryIterator {
//         lines,
//         filter,
//         oldest,
//         prev_line: None,
//     })
// }

// pub struct EntryIterator0 {
//     rev_lines: RevLines,
//     min_text_length: usize,
//     min_time_difference_seconds: i64,
//     current_entry: Option<Entry>,
// }
//
// impl EntryIterator0 {
//     pub fn new(file_name: &str, min_text_length: usize, min_time_difference_seconds: i64) -> Self {
//         let rev_lines = RevLines::new(file_name).expect("Failed to open file");
//         EntryIterator0 {
//             rev_lines,
//             min_text_length,
//             min_time_difference_seconds,
//             current_entry: None,
//         }
//     }
// }
//
// impl Iterator for EntryIterator0 {
//     type Item = Entry;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some(line) = self.rev_lines.next() {
//             let entry = Entry { ... };
//             return Some(entry);
//         }
//         None
//     }
// }
///////////////
// last_entry = None
// ## Go line by line in reverse order
// for line in source:
//     entry = Entry.from_tsv(line)
//
//     # The last entry began when this one ceased
//     if last_entry is not None:
//         last_entry.begin = entry.cease
//
//     if (
//         last_entry is not None
//         and not is_start(last_entry)
//         and any(last_entry.begin in r for r in ranges)
//         and all(_(last_entry.text) for _ in filters)
//     ):
//         matching.append(last_entry)
//
//     # Stop when dates are before any date of interest
//     if entry.cease.date() < ranges[0].begin:
//         break
//
//     # A start entry can not be used in the last entry
//     last_entry = None if is_start(entry) else entry
//
// # Matched in reverse order, return in chronological order
// matching.sort()
// return matching

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_pick_empty() {
        let mut it = EntryIterator {
            lines: RevLines::new(Cursor::new(Vec::new())),
            filter: |_| true,
            oldest: None, // NaiveDate::from_ymd_opt(2024, 04, 1),
            prev_line: None,
        };
        assert!(it.next().is_none());
    }

    fn sample_day() -> String {
        concat!(
            "2024-04-01T08:00:00Z\t*~*~*--------------------\n",
            "2024-04-01T08:10:00Z\t+messages\n",
            "2024-04-01T08:15:00Z\tday planning and prep\n",
            "2024-04-01T09:00:00Z\treviewed material for meeting\n",
            "2024-04-01T10:05:30Z\t+meeting about issues with Acme Corp\n",
            "2024-04-01T10:20:00Z\t+messages Acme Corp\n",
            "2024-04-01T12:05:00Z\t+issue 22A wip\n",
            "2024-04-01T12:55:00Z\t*~*~*--------------------\n",
            "2024-04-01T13:30:00Z\t+issue 22A resolved\n",
            "2024-04-01T14:00:00Z\t+messages\n",
            "2024-04-01T15:50:00Z\t+issue 27 resolved\n",
            "2024-04-01T16:55:00Z\t+issue 24 wip\n",
        )
        .to_string()
    }
    fn sample_week() -> String {
        concat!(
            "2024-03-25T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-25T17:00:00Z\t\n",
            "2024-03-26T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-26T17:00:00Z\t\n",
            "2024-03-27T08:00:00Z\t*~*~*--------------------\n",
            // Nothing recorded here
            "2024-03-27T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-27T17:00:00Z\t\n",
            "2024-03-28T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-28T17:00:00Z\t\n",
            "2024-03-29T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-29T17:00:00Z\t\n",
        )
        .to_string()
    }

    #[test]
    fn test_pick_all() {
        let src: String = sample_week();
        let iterator = EntryIterator {
            lines: string_reader(&src),
            filter: |_| true,
            oldest: None,
            prev_line: None,
        };
        assert_eq!(5, iterator.count());
    }

    #[test]
    fn test_pick_filter_by_text() {
        let src = sample_day();
        let iterator = EntryIterator {
            lines: string_reader(&src),
            filter: |e| e.text.starts_with("+issue"),
            oldest: None,
            prev_line: None,
        };
        // Get all entries for
        assert_eq!(4, iterator.count());
    }

    /// Create an in-memory buffer
    ///
    /// # Example
    /// let long_text = concat!(" ... ",
    ///     " ... ",
    ///     " ... ");
    /// let buffer = string_reader(long_text);
    fn string_reader(text: &str) -> RevLines<BufReader<Cursor<Vec<u8>>>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(text.as_bytes());
        RevLines::new(BufReader::new(Cursor::new(bytes)))
    }
}
