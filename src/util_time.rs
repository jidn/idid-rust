use chrono::{DateTime, FixedOffset, Local};
use std::cell::RefCell;

thread_local! {
    static FIXED_TIME: RefCell<Option<DateTime<FixedOffset>>> = RefCell::new(None);
}

pub(crate) fn current_datetime() -> DateTime<FixedOffset> {
    FIXED_TIME.with(|time_cell| {
        if let Some(datetime) = *time_cell.borrow() {
            #[cfg(test)]
            println!("current_datetime {}", datetime);
            datetime
        } else {
            #[cfg(test)]
            println!("current_datetime Local::now()");
            Local::now().fixed_offset()
        }
    })
}

/// # Example
/// current_datetime_set(DateTime::parse_from_rfc3339("2024-04-01T12:15:30+05:00").unwrap();
#[cfg(test)]
pub fn current_datetime_set(time: DateTime<FixedOffset>) {
    FIXED_TIME.with(|time_cell| {
        *time_cell.borrow_mut() = Some(time);
    });
}

#[cfg(test)]
pub fn current_datetime_reset() {
    FIXED_TIME.with(|time_cell| {
        *time_cell.borrow_mut() = None;
    });
}
