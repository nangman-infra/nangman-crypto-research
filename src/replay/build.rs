use crate::hash::stable_id;
use crate::holding::default_holding_policy;
use crate::model::{
    DEFAULT_COST_MODEL_VERSION, DEFAULT_VALIDATION_RECIPE_VERSION, IntelCandidateEvidenceBundle,
    NATIVE_REPLAY_ADAPTER, REPLAY_RUN_SCHEMA_VERSION, ReplayResultSummary, ReplayRun,
};

pub(super) fn build_replay_run(
    bundle: &IntelCandidateEvidenceBundle,
    symbol: &str,
    horizon: &str,
    window_start_ms: i64,
    window_end_ms: i64,
    result_summary: ReplayResultSummary,
) -> ReplayRun {
    let strategy_id_or_family = "candidate_event_reaction_v0";
    let parameter_variant_id = "default_event_reaction_v0";
    let window_id = format!("{window_start_ms}-{window_end_ms}");
    let partition_key = format!(
        "{}:{}:{}:{}:{}:{}:{}",
        symbol,
        bundle.hypothesis_type,
        strategy_id_or_family,
        horizon,
        window_id,
        parameter_variant_id,
        NATIVE_REPLAY_ADAPTER
    );
    let aggregate_key = format!(
        "{}:{}:{}:{}:{}:{}",
        symbol,
        bundle.hypothesis_type,
        strategy_id_or_family,
        horizon,
        parameter_variant_id,
        NATIVE_REPLAY_ADAPTER
    );
    let replay_run_id = stable_id(
        "replay",
        &[
            &bundle.candidate_id,
            symbol,
            &window_start_ms.to_string(),
            &window_end_ms.to_string(),
            NATIVE_REPLAY_ADAPTER,
        ],
    );
    ReplayRun {
        replay_run_id,
        source_candidate_id: bundle.candidate_id.clone(),
        source_candidate_lifecycle_key: bundle.candidate_lifecycle_key.clone(),
        research_partition_key: partition_key,
        research_aggregate_key: aggregate_key,
        symbol_canonical: symbol.to_owned(),
        decision_available_at_ms: bundle.decision_available_at_ms,
        symbol_universe_snapshot_id: bundle.symbol_universe_snapshot_id.clone(),
        universe_as_of_ms: bundle.universe_as_of_ms,
        approved_universe_symbol: bundle.approved_universe_symbol,
        hypothesis_type: bundle.hypothesis_type.clone(),
        validation_adapter: NATIVE_REPLAY_ADAPTER.to_owned(),
        strategy_id_or_family: strategy_id_or_family.to_owned(),
        window_start_ms,
        window_end_ms,
        holding_policy: default_holding_policy(bundle.decision_available_at_ms),
        forbidden_lookahead_boundary_ms: bundle.forbidden_lookahead_boundary_ms,
        data_quality_summary_ref: bundle.data_quality_summary.clone(),
        source_independence_summary: bundle.source_independence.clone(),
        symbol_resolution_trace_ref: bundle.symbol_resolution_trace.clone(),
        parameter_variant_id: parameter_variant_id.to_owned(),
        cost_model_version: DEFAULT_COST_MODEL_VERSION.to_owned(),
        validation_recipe_version: DEFAULT_VALIDATION_RECIPE_VERSION.to_owned(),
        result_summary,
        schema_version: REPLAY_RUN_SCHEMA_VERSION.to_owned(),
    }
}

pub(super) fn first_symbol(bundle: &IntelCandidateEvidenceBundle) -> String {
    bundle
        .normalized_symbols
        .first()
        .cloned()
        .unwrap_or_else(|| "UNKNOWN".to_owned())
}
