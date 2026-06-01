use super::*;

#[test]
fn paper_watch_live_cycle_rejects_conflicting_candidate_inputs() {
    let root = test_root("paper-watch-live-conflicting-candidate-inputs");
    let err = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--paper-watch-candidate-file",
            root.join("paper-watch-candidates.json").to_str().unwrap(),
            "--paper-watch-candidate-s3-bucket",
            "research-bucket",
            "--paper-watch-candidate-s3-key",
            "paper-watch-candidate/example.jsonl",
            "--market-live-tick-file",
            root.join("market-live-ticks.json").to_str().unwrap(),
            "--output-dir",
            root.join("out").to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("conflicting candidate inputs are rejected");

    assert!(
        err.to_string()
            .contains("use either --paper-watch-candidate-file")
    );
}

#[test]
fn paper_watch_live_cycle_rejects_bad_market_live_inputs() {
    let root = test_root("paper-watch-live-bad-market-live-inputs");
    let err = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--paper-watch-candidate-file",
            root.join("paper-watch-candidates.json").to_str().unwrap(),
            "--market-live-tick-file",
            root.join("market-live-ticks.json").to_str().unwrap(),
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-dir",
            root.join("out").to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("conflicting market live inputs are rejected");

    assert!(
        err.to_string()
            .contains("use either --market-live-tick-file")
    );
}

#[test]
fn paper_watch_live_cycle_rejects_relative_and_non_nats_inputs() {
    let relative_candidate = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--paper-watch-candidate-file",
            "paper-watch-candidates.json",
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-dir",
            test_root("paper-watch-live-relative-candidate")
                .join("out")
                .to_str()
                .unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("relative candidate file is rejected");
    assert!(
        relative_candidate
            .to_string()
            .contains("--paper-watch-candidate-file requires an absolute path")
    );

    let bad_url = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--paper-watch-candidate-s3-bucket",
            "research-bucket",
            "--paper-watch-candidate-s3-key",
            "paper-watch-candidate/example.jsonl",
            "--market-live-nats-url",
            "http://127.0.0.1:4222",
            "--output-dir",
            test_root("paper-watch-live-bad-nats-url")
                .join("out")
                .to_str()
                .unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("non-nats url is rejected");
    assert!(
        bad_url
            .to_string()
            .contains("--market-live-nats-url must start with nats://")
    );
}

#[test]
fn paper_watch_live_cycle_rejects_observer_mode_combo() {
    let root = test_root("paper-watch-live-observer-mode-combo");
    let err = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--run-paper-watch-observer",
            "--paper-watch-candidate-file",
            root.join("paper-watch-candidates.json").to_str().unwrap(),
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-dir",
            root.join("out").to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("paper watch modes must be isolated");

    assert!(
        err.to_string()
            .contains("use --run-paper-watch-live-cycle separately")
    );
}
