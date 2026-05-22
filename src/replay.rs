use crate::admission::{AdmissionDecision, horizon_ms};
use crate::hash::stable_id;
use crate::holding::default_holding_policy;
use crate::model::{
    DEFAULT_COST_MODEL_VERSION, DEFAULT_VALIDATION_RECIPE_VERSION, IntelCandidateEvidenceBundle,
    MarketFeatureDelta, MarketRegimeContext, NATIVE_REPLAY_ADAPTER, REPLAY_RUN_SCHEMA_VERSION,
    ReplayResultSummary, ReplayRun, ReplayRunStatus, ResearchBias,
};
use std::collections::BTreeSet;

pub fn build_invalid_replay_run(
    bundle: &IntelCandidateEvidenceBundle,
    admission: &AdmissionDecision,
) -> ReplayRun {
    let symbol = first_symbol(bundle);
    build_replay_run(
        bundle,
        &symbol,
        "invalid_input",
        bundle.decision_available_at_ms,
        bundle.decision_available_at_ms,
        ReplayResultSummary {
            status: ReplayRunStatus::InvalidInput,
            bias: ResearchBias::RetestBias,
            reason_codes: admission.reason_codes.clone(),
            matched_market_delta_count: 0,
            raw_return_bps: None,
            btc_adjusted_return_bps: None,
            net_after_cost_bps: None,
            estimated_cost_bps: estimated_cost_bps(bundle),
            market_regime_labels: Vec::new(),
        },
    )
}

pub fn run_native_replay(
    bundle: &IntelCandidateEvidenceBundle,
    market_deltas: &[MarketFeatureDelta],
    regime_contexts: &[MarketRegimeContext],
) -> Vec<ReplayRun> {
    let mut runs = Vec::new();
    for symbol in &bundle.normalized_symbols {
        for horizon in &bundle.allowed_horizons {
            let Some(duration_ms) = horizon_ms(horizon) else {
                continue;
            };
            let window_start_ms = bundle.forbidden_lookahead_boundary_ms;
            let window_end_ms = window_start_ms + duration_ms;
            let summary = summarize_native_replay(
                bundle,
                symbol,
                horizon,
                window_start_ms,
                window_end_ms,
                market_deltas,
                regime_contexts,
            );
            runs.push(build_replay_run(
                bundle,
                symbol,
                horizon,
                window_start_ms,
                window_end_ms,
                summary,
            ));
        }
    }
    runs
}

fn summarize_native_replay(
    bundle: &IntelCandidateEvidenceBundle,
    symbol: &str,
    _horizon: &str,
    window_start_ms: i64,
    window_end_ms: i64,
    market_deltas: &[MarketFeatureDelta],
    regime_contexts: &[MarketRegimeContext],
) -> ReplayResultSummary {
    let matched = market_deltas
        .iter()
        .filter(|delta| {
            delta.symbol_canonical == symbol
                && delta.window_start_ms >= window_start_ms
                && delta.window_end_ms <= window_end_ms
                && !delta.quality_status.eq_ignore_ascii_case("invalid")
        })
        .collect::<Vec<_>>();
    let cost_bps = estimated_cost_bps(bundle);

    if matched.is_empty() {
        return ReplayResultSummary {
            status: ReplayRunStatus::MissingMarketReplayData,
            bias: ResearchBias::RetestBias,
            reason_codes: vec!["missing_native_replay_market_data".to_owned()],
            matched_market_delta_count: 0,
            raw_return_bps: None,
            btc_adjusted_return_bps: None,
            net_after_cost_bps: None,
            estimated_cost_bps: cost_bps,
            market_regime_labels: Vec::new(),
        };
    }

    let returns = matched
        .iter()
        .filter_map(|delta| return_pct(delta).map(|value| value * 100.0))
        .collect::<Vec<_>>();

    if returns.is_empty() {
        return ReplayResultSummary {
            status: ReplayRunStatus::InsufficientEvidence,
            bias: ResearchBias::RetestBias,
            reason_codes: vec!["native_replay_return_metric_missing".to_owned()],
            matched_market_delta_count: matched.len(),
            raw_return_bps: None,
            btc_adjusted_return_bps: None,
            net_after_cost_bps: None,
            estimated_cost_bps: cost_bps,
            market_regime_labels: matching_regime_labels(
                regime_contexts,
                window_start_ms,
                window_end_ms,
            ),
        };
    }

    let market_regime_labels =
        matching_regime_labels(regime_contexts, window_start_ms, window_end_ms);
    let raw_return_bps = average(&returns);
    let btc_adjustment_bps = regime_contexts
        .iter()
        .filter(|context| {
            context.window_start_ms >= window_start_ms && context.window_end_ms <= window_end_ms
        })
        .filter_map(|context| context.btc_return_same_window.map(|value| value * 100.0))
        .next();
    let btc_adjusted_return_bps = btc_adjustment_bps
        .map(|btc_return_bps| raw_return_bps - btc_return_bps)
        .unwrap_or(raw_return_bps);
    let net_after_cost_bps = btc_adjusted_return_bps - cost_bps;

    let (bias, mut reasons) = if net_after_cost_bps <= 0.0 {
        (
            ResearchBias::PruneBias,
            vec!["native_replay_net_edge_non_positive".to_owned()],
        )
    } else {
        (
            ResearchBias::RetestBias,
            vec![
                "native_replay_positive_but_survival_not_proven".to_owned(),
                "needs_unseen_window_validation".to_owned(),
            ],
        )
    };
    if btc_adjustment_bps.is_none() {
        reasons.push("market_regime_context_missing".to_owned());
    }
    if market_regime_labels.is_empty() {
        reasons.push("market_regime_label_missing".to_owned());
    }

    ReplayResultSummary {
        status: ReplayRunStatus::Completed,
        bias,
        reason_codes: reasons,
        matched_market_delta_count: matched.len(),
        raw_return_bps: Some(raw_return_bps),
        btc_adjusted_return_bps: Some(btc_adjusted_return_bps),
        net_after_cost_bps: Some(net_after_cost_bps),
        estimated_cost_bps: cost_bps,
        market_regime_labels,
    }
}

fn build_replay_run(
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

fn matching_regime_labels(
    regime_contexts: &[MarketRegimeContext],
    window_start_ms: i64,
    window_end_ms: i64,
) -> Vec<String> {
    regime_contexts
        .iter()
        .filter(|context| {
            context.window_start_ms >= window_start_ms
                && context.window_end_ms <= window_end_ms
                && !context.quality_status.eq_ignore_ascii_case("invalid")
        })
        .map(|context| context.volatility_regime.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn first_symbol(bundle: &IntelCandidateEvidenceBundle) -> String {
    bundle
        .normalized_symbols
        .first()
        .cloned()
        .unwrap_or_else(|| "UNKNOWN".to_owned())
}

fn return_pct(delta: &MarketFeatureDelta) -> Option<f64> {
    delta
        .price_change_same_window
        .or(delta.change_pct_15m)
        .or(delta.change_pct_1h)
}

fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn estimated_cost_bps(bundle: &IntelCandidateEvidenceBundle) -> f64 {
    let mut cost = 0.0;
    if bundle.validation_requirements.include_fee {
        cost += 10.0;
    }
    if bundle.validation_requirements.include_slippage {
        cost += 5.0;
    }
    if bundle.validation_requirements.include_latency_assumption {
        cost += 2.0;
    }
    cost
}
