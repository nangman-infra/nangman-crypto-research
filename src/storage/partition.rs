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

pub(super) fn normalize_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }
    format!("{}/", trimmed.trim_end_matches('/'))
}

pub fn hourly_partitioned_prefix(prefix: &str, timestamp_ms: i64) -> AppResult<String> {
    let dt = partition(timestamp_ms)?;
    Ok(format!(
        "{}dt={}/hour={:02}/",
        normalize_prefix(prefix),
        dt.date,
        dt.hour
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hourly_partitioned_prefix_narrows_observer_restore_scan() {
        assert_eq!(
            hourly_partitioned_prefix(
                "paper-watch-live-mark/schema=paper_watch_live_mark_v1",
                1_779_935_219_010,
            )
            .expect("valid prefix"),
            "paper-watch-live-mark/schema=paper_watch_live_mark_v1/dt=2026-05-28/hour=02/"
        );
    }
}
