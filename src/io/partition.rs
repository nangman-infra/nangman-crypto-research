use crate::error::{AppError, AppResult};
use chrono::{DateTime, Datelike, Timelike, Utc};

pub(super) struct Partition {
    pub(super) date: String,
    pub(super) hour: u32,
}

pub(super) fn partition(timestamp_ms: i64) -> AppResult<Partition> {
    let dt = DateTime::<Utc>::from_timestamp_millis(timestamp_ms)
        .ok_or_else(|| AppError::validation(format!("invalid timestamp_ms: {timestamp_ms}")))?;
    Ok(Partition {
        date: format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day()),
        hour: dt.hour(),
    })
}
