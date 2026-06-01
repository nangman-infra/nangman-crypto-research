use chrono::{DateTime, Datelike, Timelike, Utc};

pub(in crate::alert) fn pipeline_alert_event_key(
    prefix: &str,
    created_at_ms: i64,
    app: &str,
    priority: &str,
    event_id: &str,
) -> Result<String, String> {
    let created_at = DateTime::<Utc>::from_timestamp_millis(created_at_ms)
        .ok_or_else(|| "created_at_ms is outside supported timestamp range".to_owned())?;
    Ok(format!(
        "{}/dt={:04}-{:02}-{:02}/hour={:02}/app={}/priority={}/{}.json",
        prefix.trim().trim_matches('/'),
        created_at.year(),
        created_at.month(),
        created_at.day(),
        created_at.hour(),
        s3_key_token(app),
        s3_key_token(priority),
        s3_key_token(event_id),
    ))
}

pub(in crate::alert) fn s3_key_token(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '=' => character,
            _ => '_',
        })
        .collect()
}
