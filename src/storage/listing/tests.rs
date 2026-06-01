use super::limits::{PayloadListOptions, scan_limit_exceeded_error};
use super::selection::ListedPayloadObject;
use super::*;

#[test]
fn latest_payload_key_selection_prefers_recent_jsonl_parts() {
    let keys = select_latest_payload_keys(
        vec![
            ListedPayloadObject {
                key: "replay-run-index/schema=x/dt=2026-05-22/part-000001.jsonl".to_owned(),
                last_modified_ms: 100,
            },
            ListedPayloadObject {
                key: "replay-run-index/schema=x/dt=2026-05-23/part-000001.jsonl".to_owned(),
                last_modified_ms: 300,
            },
            ListedPayloadObject {
                key: "replay-run-index/schema=x/dt=2026-05-21/part-000001.jsonl".to_owned(),
                last_modified_ms: 200,
            },
        ],
        2,
    );

    assert_eq!(
        keys,
        vec![
            "replay-run-index/schema=x/dt=2026-05-23/part-000001.jsonl",
            "replay-run-index/schema=x/dt=2026-05-21/part-000001.jsonl",
        ]
    );
}

#[test]
fn scan_limit_error_uses_artifact_label() {
    let error = scan_limit_exceeded_error(
        "research-bucket",
        "paper-watch-live-mark/schema=x/",
        PayloadListOptions {
            file_suffix: "/part-000001.jsonl",
            scan_limit: 10,
            artifact_label: "paper-watch live mark",
        },
    );

    assert!(error.to_string().contains("paper-watch live mark"));
    assert!(
        !error
            .to_string()
            .contains("historical replay-run-index S3 scan limit exceeded")
    );
}
