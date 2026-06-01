use super::super::*;
use std::path::{Path, PathBuf};

pub(super) struct ShadowPromotionInputPaths {
    input: PathBuf,
    delta: PathBuf,
    regime: PathBuf,
    output: PathBuf,
}

pub(super) fn write_shadow_promotion_inputs(root: &Path) -> ShadowPromotionInputPaths {
    let paths = ShadowPromotionInputPaths {
        input: root.join("bundles.json"),
        delta: root.join("delta.json"),
        regime: root.join("regime.json"),
        output: root.join("out"),
    };
    let mut bundles = Vec::new();
    let mut deltas = Vec::new();
    let mut regimes = Vec::new();

    for index in 0..31 {
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

    write_json(&paths.input, &Value::Array(bundles));
    write_json(&paths.delta, &Value::Array(deltas));
    write_json(&paths.regime, &Value::Array(regimes));

    paths
}

pub(super) fn shadow_promotion_args(paths: ShadowPromotionInputPaths) -> Args {
    Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(paths.input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(paths.delta),
        market_regime_context_file: Some(paths.regime),
        market_l1_s3_bucket: None,
        market_feature_delta_s3_keys: Vec::new(),
        market_regime_context_s3_keys: Vec::new(),
        historical_replay_run_files: Vec::new(),
        historical_replay_run_index_files: Vec::new(),
        oss_adapter_run_files: Vec::new(),
        shadow_validation_run_files: Vec::new(),
        oss_adapter_run_s3_bucket: None,
        oss_adapter_run_s3_keys: Vec::new(),
        shadow_validation_run_s3_bucket: None,
        shadow_validation_run_s3_keys: Vec::new(),
        historical_replay_run_s3_bucket: None,
        historical_replay_run_s3_keys: Vec::new(),
        historical_replay_run_index_s3_bucket: None,
        historical_replay_run_index_s3_keys: Vec::new(),
        output_dir: Some(paths.output),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(120_000_000),
        ..default_args()
    }
}
