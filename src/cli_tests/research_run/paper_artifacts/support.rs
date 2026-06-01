use super::*;
use std::path::{Path, PathBuf};

pub(super) struct GateInputFiles {
    pub(super) input: PathBuf,
    pub(super) delta: PathBuf,
    pub(super) regime: PathBuf,
}

pub(super) fn write_gate_inputs(root: &Path, sample_count: usize) -> GateInputFiles {
    let input = root.join("bundles.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let mut bundles = Vec::new();
    let mut deltas = Vec::new();
    let mut regimes = Vec::new();

    for index in 0..sample_count {
        let decision_ms = 1_300 + (index as i64 * 3_600_000);
        let window_end_ms = decision_ms + 3_600_000;
        bundles.push(bundle_json_with_gate_inputs(index, decision_ms));
        deltas.push(market_delta_json(
            &format!("delta_{index:03}"),
            decision_ms,
            window_end_ms,
            0.5,
        ));
        regimes.push(market_regime_json(
            &format!("regime_{index:03}"),
            decision_ms,
            window_end_ms,
        ));
    }

    write_json(&input, &Value::Array(bundles));
    write_json(&delta, &Value::Array(deltas));
    write_json(&regime, &Value::Array(regimes));
    GateInputFiles {
        input,
        delta,
        regime,
    }
}

pub(super) fn paper_artifact_run_args(
    input: PathBuf,
    delta: Option<PathBuf>,
    regime: Option<PathBuf>,
    shadow_validation_run_files: Vec<PathBuf>,
    output_dir: PathBuf,
) -> Args {
    Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: delta,
        market_regime_context_file: regime,
        market_l1_s3_bucket: None,
        market_feature_delta_s3_keys: Vec::new(),
        market_regime_context_s3_keys: Vec::new(),
        historical_replay_run_files: Vec::new(),
        historical_replay_run_index_files: Vec::new(),
        oss_adapter_run_files: Vec::new(),
        shadow_validation_run_files,
        oss_adapter_run_s3_bucket: None,
        oss_adapter_run_s3_keys: Vec::new(),
        shadow_validation_run_s3_bucket: None,
        shadow_validation_run_s3_keys: Vec::new(),
        historical_replay_run_s3_bucket: None,
        historical_replay_run_s3_keys: Vec::new(),
        historical_replay_run_index_s3_bucket: None,
        historical_replay_run_index_s3_keys: Vec::new(),
        output_dir: Some(output_dir),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(120_000_000),
        ..default_args()
    }
}

pub(super) fn read_report(summary: &RunSummary) -> Value {
    serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
        .expect("report json parses")
}

pub(super) fn output_line_count(summary: &RunSummary, marker: &str) -> usize {
    let output_file = output_file_containing(summary, marker);
    fs::read_to_string(output_file)
        .expect("artifact output exists")
        .lines()
        .count()
}

pub(super) fn first_registry_record(summary: &RunSummary) -> Value {
    let registry_file = output_file_containing(summary, "/research-aggregate-registry/");
    let registry_text = fs::read_to_string(&registry_file).expect("registry output exists");
    serde_json::from_str(
        registry_text
            .lines()
            .next()
            .expect("registry output has one line"),
    )
    .expect("registry line parses")
}
