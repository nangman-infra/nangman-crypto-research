use super::*;

#[test]
fn parse_s3_uri_accepts_valid_artifact_location() {
    assert_eq!(
        parse_s3_uri("s3://nangman-crypto-dev-research-123/replay-run/schema=v1/part.jsonl"),
        Some((
            "nangman-crypto-dev-research-123".to_owned(),
            "replay-run/schema=v1/part.jsonl".to_owned()
        ))
    );
}

#[test]
fn parse_s3_uri_rejects_invalid_bucket_name() {
    assert_eq!(parse_s3_uri("s3://Bad_Bucket/replay-run/part.jsonl"), None);
    assert_eq!(parse_s3_uri("s3://192.168.5.4/replay-run/part.jsonl"), None);
}

#[test]
fn parse_s3_uri_rejects_period_only_key_segments() {
    assert_eq!(
        parse_s3_uri("s3://valid-bucket/replay-run/../part.jsonl"),
        None
    );
    assert_eq!(parse_s3_uri("s3://valid-bucket/./part.jsonl"), None);
}
