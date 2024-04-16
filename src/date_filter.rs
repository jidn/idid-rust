use chrono::NaiveDate;

#[derive(Debug)]
pub struct DateFilter {
    date_ranges: Vec<(NaiveDate, NaiveDate)>,
    dates: Vec<NaiveDate>,

    // The oldest and newest date from dates or date_ranges
    pub oldest_date: Option<NaiveDate>,
    pub newest_date: Option<NaiveDate>,
}

impl DateFilter {
    /// New DateFilter
    ///
    /// # Arguments
    /// * date_ranges - range in pairs, but pair ordering is not important
    /// * individual_dates - individual days
    ///
    /// # Returns
    /// A new DateFilter
    /// # Example
    /// ```
    /// let range = [];
    /// let dates = [];
    /// let filter = DateFilter::new(&range, &dates);
    ///
    /// ```
    pub fn new(date_ranges: &[NaiveDate], individual_dates: &[NaiveDate]) -> Self {
        let mut oldest_date: Option<NaiveDate> = None;
        let mut newest_date: Option<NaiveDate> = None;

        // Process individual dates
        let sorted_individual_dates = {
            let mut sorting = individual_dates.to_vec();
            sorting.sort();
            sorting
        };
        // Get the oldest and newest so far
        if !sorted_individual_dates.is_empty() {
            oldest_date = sorted_individual_dates.first().cloned();
            newest_date = sorted_individual_dates.last().cloned();
        }

        let mut processed_ranges = Vec::new();
        if date_ranges.len() > 0 {
            for pair in date_ranges.chunks_exact(2) {
                let (start, end) = match pair {
                    [start, end] => (*start, *end),
                    _ => panic!("Invalid pair of dates"),
                };
                // Swap start and end if they are in the wrong order
                let (start, end) = if start > end {
                    (end, start)
                } else {
                    (start, end)
                };

                processed_ranges.push((start, end));
                newest_date = newest_date.map_or(Some(end), |newest| Some(newest.max(end)));
            }

            // Sort by the begin of each range
            processed_ranges.sort_by(|a, b| a.0.cmp(&b.0));

            // Once sorted the processed_ranges oldest is obvious
            if let Some((first_start, _)) = processed_ranges.first() {
                oldest_date = match oldest_date {
                    Some(current_oldest) if *first_start < current_oldest => Some(*first_start),
                    None => Some(*first_start),
                    _ => oldest_date,
                };
            }
        }
        Self {
            date_ranges: processed_ranges,
            dates: sorted_individual_dates,
            oldest_date,
            newest_date,
        }
    }

    /// Does the given date match any of the dates or date_ranges
    ///
    /// # Arguments
    /// * date - for comparison
    ///
    /// # Returns
    /// `bool` if the given date matches either one of the struct dates
    /// or is in any of the inclusive date ranges.
    pub fn contains(&self, date: &NaiveDate) -> bool {
        if let (Some(oldest), Some(newest)) = (self.oldest_date, self.newest_date) {
            if date < &oldest || &newest < date {
                return false;
            }
        }
        self.dates.contains(date)
            || self
                .date_ranges
                .iter()
                .any(|(begin, cease)| date >= begin && date <= cease)
    }
}

#[cfg(test)]
pub fn ymd(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_empty() {
        let range = [];
        let dates = [];

        let filter = DateFilter::new(&range, &dates);
        println!("Filter: {:#?}", filter);
        assert!(filter.date_ranges.is_empty());
        assert!(filter.dates.is_empty());
        assert_eq!(filter.oldest_date, None);
        assert_eq!(filter.newest_date, None);
        assert_eq!(
            filter.contains(&NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
            false
        );
    }

    #[test]
    fn test_filter_only_dates() {
        let individual_dates = vec![ymd(2024, 3, 1), ymd(2024, 2, 1), ymd(2024, 4, 1)];
        let filter = DateFilter::new(&[], &individual_dates);
        assert_eq!(filter.contains(&ymd(2024, 3, 1)), true);
        assert_eq!(filter.contains(&ymd(2024, 1, 1)), false);
        assert_eq!(filter.oldest_date, filter.dates.first().cloned());
        assert_eq!(filter.newest_date, filter.dates.last().cloned());
    }

    #[test]
    fn test_filter_only_ranges() {
        let date_ranges = vec![
            ymd(2024, 3, 1),
            ymd(2024, 3, 10),
            ymd(2024, 1, 10),
            ymd(2024, 1, 1),
        ];
        let filter = DateFilter::new(&date_ranges, &[]);
        assert!(!filter.date_ranges.is_empty());

        let expected_oldest = filter.date_ranges.first().map(|(start, _)| *start);
        assert_eq!(filter.oldest_date, expected_oldest);

        let expected_newest = filter.date_ranges.last().map(|(_, end)| *end);
        assert_eq!(filter.newest_date, expected_newest);

        assert_eq!(filter.contains(&ymd(2024, 1, 1)), true);
        assert_eq!(filter.contains(&ymd(2024, 1, 10)), true);
        assert_eq!(filter.contains(&ymd(2024, 1, 5)), true);
        assert_eq!(filter.contains(&ymd(2024, 2, 1)), false);
    }

    #[test]
    fn test_filter_both_ranges_and_dates() {
        let date_ranges = vec![
            ymd(2024, 1, 1),
            ymd(2024, 1, 10),
            ymd(2024, 3, 1),
            ymd(2024, 3, 10),
        ];
        let individual_dates = vec![ymd(2024, 4, 1), ymd(2024, 2, 1), ymd(2024, 3, 15)];
        let filter = DateFilter::new(&date_ranges, &individual_dates);

        assert_eq!(filter.oldest_date, Some(ymd(2024, 1, 1)));
        assert_eq!(filter.newest_date, Some(ymd(2024, 4, 1)));

        assert_eq!(filter.contains(&ymd(2024, 1, 5)), true);
        assert_eq!(filter.contains(&ymd(2024, 1, 1)), true);
        assert_eq!(filter.contains(&ymd(2024, 1, 10)), true);
        assert_eq!(filter.contains(&ymd(2024, 6, 1)), false);
    }
}
