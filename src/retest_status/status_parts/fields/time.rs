use chrono::{DateTime, Utc};

pub(in crate::retest_status::status_parts) fn horizon_rank(horizon: &str) -> usize {
    match horizon {
        "1h" => 1,
        "4h" => 2,
        "24h" | "1d" => 3,
        "72h" => 4,
        "7d" => 5,
        _ => 99,
    }
}

pub(in crate::retest_status) fn iso8601_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}
