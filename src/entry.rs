use crate::date_filter::DateFilter;
use chrono::{DateTime, FixedOffset};
use rev_lines::RevLines;
use std::fmt;
use std::fs;
use std::io;

// Create trait as creating variables with multiple traits causes errors
pub trait BufReadSeek: io::BufRead + io::Seek {}
impl<T: io::BufRead + io::Seek> BufReadSeek for T {}

pub const START_RECORDING: &str = "*~*~*--------------------";

/// An entry with a begin timestamp, cease timestamp, and associated text.
#[derive(Clone)]
pub struct Entry {
    pub begin: DateTime<FixedOffset>,
    pub cease: DateTime<FixedOffset>,
    pub text: String,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}",
            self.begin.to_rfc3339(),
            self.cease.to_rfc3339(),
            self.text
        )
    }
}

impl Entry {
    pub fn duration(&self) -> chrono::Duration {
        self.cease - self.begin
    }
    pub fn hh_mm(&self) -> String {
        hh_mm(&self.duration())
    }

    /// Serialize as json or TSV
    pub fn serialize(&self, in_seconds: &bool, json: bool) -> String {
        let (label, value) = match in_seconds {
            true => ("seconds", format!("{}", self.duration().num_seconds())),
            false => ("duration", format!("\"{}\"", self.hh_mm())),
        };
        if json {
            format!(
                "{{\"begin\":\"{}\",\"{}\":{},\"text\":\"{}\"}}",
                self.begin.to_rfc3339(),
                label,
                value,
                escape_for_json(&self.text)
            )
        } else {
            format!(
                "{}\t{}\t{}",
                self.begin.to_rfc3339(),
                value.trim_start_matches('"').trim_end_matches('"'),
                self.text
            )
        }
    }

    /// Get timestamp and text from a tab-separated value (TSV) line.
    ///
    /// # Arguments
    /// * `line` - A tab-separated string containing a rfc3339 timestamp and text.
    ///
    /// # Return
    /// timestamp and text
    ///
    /// # Examples
    ///
    /// use idid::entry;
    ///
    /// let line = "2024-04-01T12:00:00+00:00\tSample text";
    /// let entry = entry::Entry::from_tsv(line).unwrap();
    /// assert_eq!(entry.1, "Sample text");
    pub fn from_tsv(line: &str) -> Result<(chrono::DateTime<FixedOffset>, String), String> {
        let mut parts = line.splitn(2, '\t');
        let when_str = parts.next().expect("TSV line.");
        let when = DateTime::parse_from_rfc3339(when_str)
            .map_err(|e| format!("DateTime parser error '{when_str}': {e}"))?;
        let text = parts.next().unwrap_or_default().to_string();
        Ok((when, text))
    }
}

/// Reverse iterator over entries with filter.
pub struct EntryIterator<F, R>
where
    F: FnMut(&Entry) -> bool,
    R: BufReadSeek,
{
    // Search the file in reverse order
    lines: RevLines<R>,
    // The current number of lines from the end
    line_from_end: u32,

    // Usually DateFilter.contains(&Entry)->bool
    filter: F,

    // Usually DateFilter.oldest_date: Option(NaiveDate)
    oldest: Option<chrono::NaiveDate>,

    // rfc 3339\tText for what I did at this time
    last_line: Option<(DateTime<FixedOffset>, String)>,
}

/// # Arguments
/// lines: A reverse text file iterator.
/// filter: Does the entry match.
/// oldest: The oldest date allowed for early termination.
/// last_line: The timestamp and text from previous entry.
///
/// # Example
/// use idid::entry;
/// use rev_lines;
///
/// let tsv = rev_lines::RevLines::new(...);
/// let iterator = entry::EntryIterator {
///     lines: &tsv,
///     filter: |_| true,
///     oldest: None,
///     last_line: None,
/// };
///
/// You can also use a DateFilter.
///
/// use idid::date_filter;
/// use idid::entry;
/// use rev_lines;
///
/// let tsv = rev_lines::RevLines::new(...);
/// let filter = date_filter::DateFilter(&[], &[]);
/// let iterator = entry::EntryIterator::new(&tsv, &filter);
///
/// for entry in iterator {
///     println!(entry);
/// }
///
impl<F, R> EntryIterator<F, R>
where
    F: FnMut(&Entry) -> bool,
    R: BufReadSeek,
{
    /// Creates a new EntryIterator instance.
    ///
    /// Use the function pick for easier iterator creation.
    ///
    /// # Arguments
    ///
    /// * `source` - The source to read entries from (e.g., file, string).
    /// * `filter` - The filter function to apply to entries.
    /// * `oldest` - Optional oldest date for filtering entries.
    ///
    /// use idid::entry::{Entry, EntryIterator};
    ///
    /// let source = concat!(
    ///     "2024-04-01T08:00:00Z\t*~*~*--------------------\n",
    ///     "2024-04-01T12:00:00Z\tSample text\n",
    ///     "2024-04-02T12:15:00Z\tAnother entry");
    /// let oldest = chrono::NaiveDate::from_ymd_opt(2024, 4, 1);
    /// let filter = |entry: &Entry| entry.text.contains("Sample");
    ///
    /// let iter = EntryIterator::new(source.as_bytes(), filter, Some(oldest));
    ///
    /// for entry in iter {
    ///     println!("{:?}", entry);
    /// }
    pub fn new(source: R, filter: F, oldest: Option<chrono::NaiveDate>) -> Self {
        Self {
            filter,
            oldest,
            last_line: None,
            lines: RevLines::new(source),
            line_from_end: 0,
        }
    }
}

impl<F, R> Iterator for EntryIterator<F, R>
where
    F: FnMut(&Entry) -> bool,
    R: io::BufRead + io::Seek,
{
    type Item = Entry;
    /// Get the next matching entry.
    fn next(&mut self) -> Option<Self::Item> {
        // while let Some(line) = self.lines.next() {
        for line in self.lines.by_ref() {
            self.line_from_end += 1;
            let line = line.unwrap().trim().to_string();
            //println!("#{} {}", self.line_from_end, line);

            // Each TSV line is timestamp and description of something done
            let (when, text) = Entry::from_tsv(&line)
                .map_err(|e| format!("TSV #{} from end: {}", self.line_from_end, e))
                .unwrap_or_else(|err_msg| panic!("{}", err_msg));

            // Older than oldest? Nothing more to find.
            if self.oldest.is_some() && when.date_naive() < self.oldest? {
                // println!("finished: when={} >= oldest={:?}", when, self.oldest);
                return None;
            }

            // Create entry from last line and this line rfc3339 as beginning
            let some_entry = match self.last_line {
                Some((ref last_cease, ref last_text)) => Some(Entry {
                    begin: when,
                    cease: *last_cease,
                    text: last_text.clone(),
                }),
                None => None,
            };

            // Keep the current line data for the next line
            self.last_line = match text.starts_with(START_RECORDING) {
                true => None,
                false => Some((when, text)),
            };
            // println!("Entry={:?}, last_line={:?}", some_entry, self.last_line);

            // Is this an acceptible entry?
            if some_entry.is_some() {
                let entry = some_entry.unwrap();

                if (self.filter)(&entry) {
                    // println!("  acceptable: true");
                    return Some(entry);
                }
                // println!("  acceptable: false  {:?}", entry);
            }
        }
        // println!("  acceptable: False");
        None
    }
}

/// Picks entries from input using filter.
///
/// # Arguments
///
/// * `input` - Input source, which can be a file path or a string.
/// * `filter` - The entry filter predicate.
///
/// # Examples
///
/// use chrono::NaiveDate;
/// use date_filter::DateFilter;
/// use entry::pick;
///
/// let input = concat!(
///     "2024-04-01T08:00:00Z\t*~*~*--------------------\n",
///     "2024-04-01T12:00:00Z\tSample text\n",
///     "2024-04-02T12:15:00Z\tAnother entry");
/// let filter = DateFilter::new();
///
/// for entry in pick(input, &filter){
///     println!("{:?}", entry);
/// }
pub fn pick(
    input: impl Into<String>,
    filter: &DateFilter,
) -> EntryIterator<impl FnMut(&Entry) -> bool + '_, impl BufReadSeek> {
    let input_str = input.into();
    let buf_reader: Box<dyn BufReadSeek> = if input_str.contains('\t') {
        Box::new(io::Cursor::new(input_str))
    } else {
        let file = fs::File::open(&input_str).expect("Failed to open file");
        Box::new(io::BufReader::new(file))
    };

    let filter_func = move |entry: &Entry| filter.contains(&entry.begin.date_naive());

    EntryIterator::new(buf_reader, filter_func, filter.oldest_date)
}

pub fn hh_mm(duration: &chrono::Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60; // Get remaining minutes after subtracting full hours

    format!("{:02}:{:02}", hours, minutes)
}

fn escape_for_json(text: &str) -> String {
    let mut escaped_string = String::new();
    for c in text.chars() {
        match c {
            '"' => escaped_string.push_str("\\\""),
            '\\' => escaped_string.push_str("\\\\"),
            '\n' => escaped_string.push_str("\\n"),
            '\r' => escaped_string.push_str("\\r"),
            '\t' => escaped_string.push_str("\\t"),
            _ => escaped_string.push(c),
        }
    }
    escaped_string
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date_filter::ymd;

    #[test]
    fn test_entry_from_tsv() {
        let line = "2024-04-01T12:00:00+00:00\tSample text";
        let entry = Entry::from_tsv(line).unwrap();
        assert_eq!(entry.1, "Sample text");
    }

    #[test]
    fn test_pick_iterator_empty() {
        let mut iterator = EntryIterator {
            filter: |_: &Entry| true,
            oldest: None,
            last_line: None,
            lines: RevLines::new(io::Cursor::new(Vec::new())),
            line_from_end: 0,
        };
        let next_value = iterator.next();
        // println!("next={:?}", next_value);
        assert!(next_value.is_none());
    }

    #[test]
    fn test_entry_iterator_new() {
        let source: String = sample_simple();
        let reader = string_reader(&source);

        // Only get one of the two possible entries
        let filter_func = |entry: &Entry| entry.text.contains("Sample");
        let iterator = EntryIterator::new(reader, filter_func, None);
        let entries: Vec<_> = iterator.collect();

        assert_eq!(1, entries.len());
        assert_eq!(entries[0].text, "Sample text");
    }

    #[test]
    fn test_pick_with_datefilter_all() {
        let source: String = sample_week();
        let date_ranges = vec![ymd(2024, 4, 1), ymd(2024, 3, 1)];
        let filter = DateFilter::new(&date_ranges, &[]);

        let actual = pick(source, &filter).count();
        assert_eq!(5, actual);
    }

    // Two entries
    fn sample_simple() -> String {
        concat!(
            "2024-04-01T08:00:00Z\t*~*~*--------------------\n",
            "2024-04-01T12:00:00Z\tSample text\n",
            "2024-04-01T12:15:00Z\tAnother entry",
        )
        .to_string()
    }

    /// A day with 4 issue, 3 messages, and total of 10 entries
    // fn sample_day() -> String {
    //     concat!(
    //         "2024-04-01T08:00:00Z\t*~*~*--------------------\n",
    //         "2024-04-01T08:10:00Z\t+messages\n",
    //         "2024-04-01T08:15:00Z\tday planning and prep\n",
    //         "2024-04-01T09:00:00Z\treviewed material for meeting\n",
    //         "2024-04-01T10:05:30Z\t+meeting about issues with Acme Corp\n",
    //         "2024-04-01T10:20:00Z\t+messages Acme Corp\n",
    //         "2024-04-01T12:05:00Z\t+issue 22A wip\n",
    //         "2024-04-01T12:55:00Z\t*~*~*--------------------\n",
    //         "2024-04-01T13:30:00Z\t+issue 22A resolved\n",
    //         "2024-04-01T14:00:00Z\t+messages\n",
    //         "2024-04-01T15:50:00Z\t+issue 27 resolved\n",
    //         "2024-04-01T16:55:00Z\t+issue 24 wip\n",
    //     )
    //     .to_string()
    // }

    /// One entry for each day of the work week
    fn sample_week() -> String {
        concat!(
            "2024-03-25T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-25T17:00:00Z\tMonday\n",
            "2024-03-26T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-26T17:00:00Z\tTuesday\n",
            "2024-03-27T08:00:00Z\t*~*~*--------------------\n",
            // Nothing recorded here; multiple, sequential starts
            "2024-03-27T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-27T17:00:00Z\tWednesday\n",
            "2024-03-28T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-28T17:00:00Z\tThursday\n",
            "2024-03-29T09:00:00Z\t*~*~*--------------------\n",
            "2024-03-29T17:00:00Z\tFriday\n",
        )
        .to_string()
    }

    /// Create an in-memory buffer
    fn string_reader(text: &str) -> io::BufReader<io::Cursor<Vec<u8>>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(text.as_bytes());
        io::BufReader::new(io::Cursor::new(bytes))
    }
}
