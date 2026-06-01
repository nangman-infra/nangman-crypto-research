use crate::model::{
    CandidateClass, DataQualitySummaryRef, HoldingPolicy, IntelCandidateEvidenceBundle,
    LiquidityFilterStatus, LiquidityFilterSummary, ReplayResultSummary, ReplayRun, ReplayRunStatus,
    ResearchBias, SourceIndependenceSummary, ValidationRequirements,
};
use std::collections::BTreeMap;

pub(super) fn replay_run(
    replay_run_id: &str,
    candidate_id: &str,
    window_start_ms: i64,
) -> ReplayRun {
    ReplayRun {
        replay_run_id: replay_run_id.to_owned(),
        source_candidate_id: candidate_id.to_owned(),
        source_candidate_lifecycle_key: "candidate-life".to_owned(),
        research_partition_key: "partition".to_owned(),
        research_aggregate_key: "aggregate".to_owned(),
        symbol_canonical: "SUI".to_owned(),
        decision_available_at_ms: 1_000,
        symbol_universe_snapshot_id: "universe".to_owned(),
        universe_as_of_ms: 900,
        approved_universe_symbol: true,
        hypothesis_type: "risk_incident_watch".to_owned(),
        validation_adapter: "event_reaction_smoke".to_owned(),
        strategy_id_or_family: "strategy".to_owned(),
        window_start_ms,
        window_end_ms: window_start_ms + 900_000,
        forbidden_lookahead_boundary_ms: 1_000,
        holding_policy: HoldingPolicy::default(),
        data_quality_summary_ref: data_quality_summary(),
        source_independence_summary: source_independence(),
        symbol_resolution_trace_ref: Vec::new(),
        parameter_variant_id: "base".to_owned(),
        cost_model_version: "cost_v1".to_owned(),
        validation_recipe_version: "recipe_v1".to_owned(),
        result_summary: ReplayResultSummary {
            status: ReplayRunStatus::Completed,
            bias: ResearchBias::PromoteToShadowBias,
            reason_codes: Vec::new(),
            matched_market_delta_count: 1,
            raw_return_bps: Some(12.0),
            btc_adjusted_return_bps: Some(10.0),
            net_after_cost_bps: Some(8.0),
            estimated_cost_bps: 1.0,
            market_regime_labels: vec!["medium_volatility".to_owned()],
            liquidity_filter_summary: Some(LiquidityFilterSummary {
                status: LiquidityFilterStatus::Passed,
                reason_codes: vec!["liquidity_filter_positive_volume_observed".to_owned()],
                observed_metric_count: 1,
                positive_volume_metric_count: 1,
            }),
        },
        schema_version: "replay_run_v1".to_owned(),
    }
}

pub(super) fn evidence_bundle(
    candidate_id: &str,
    include_liquidity_filter: bool,
) -> IntelCandidateEvidenceBundle {
    IntelCandidateEvidenceBundle {
        candidate_id: candidate_id.to_owned(),
        candidate_lifecycle_key: "candidate-life".to_owned(),
        bundle_key: "bundle-key".to_owned(),
        producer_app: "test".to_owned(),
        producer_run_id: "run".to_owned(),
        created_at_ms: 1_000,
        event_time_ms: 1_000,
        published_at_ms: None,
        fetched_at_ms: 1_000,
        structured_at_ms: 1_000,
        candidate_created_at_ms: 1_000,
        decision_available_at_ms: 1_000,
        forbidden_lookahead_boundary_ms: 1_000,
        schema_version: "intel_candidate_evidence_bundle_v1".to_owned(),
        scoring_policy_version: "policy".to_owned(),
        normalized_symbols: vec!["SUI".to_owned()],
        symbol_universe_snapshot_id: "universe".to_owned(),
        universe_as_of_ms: 900,
        approved_universe_symbol: true,
        event_types: vec!["incident".to_owned()],
        hypothesis_type: "risk_incident_watch".to_owned(),
        allowed_horizons: vec!["24h".to_owned()],
        source_story_cluster_ids: vec!["cluster".to_owned()],
        source_structured_packet_ids: vec!["packet".to_owned()],
        source_context_flag_packet_ids: Vec::new(),
        evidence_refs: Vec::new(),
        metric_evidence: Vec::new(),
        data_quality_summary: data_quality_summary(),
        selected_market_artifacts: Vec::new(),
        candidate_class: CandidateClass::ResearchCandidate,
        candidate_score: 80,
        research_priority: "p0_event_risk".to_owned(),
        research_eligible: true,
        validation_requirements: ValidationRequirements {
            required_adapters: Vec::new(),
            optional_adapters: Vec::new(),
            min_unseen_windows: 0,
            include_fee: true,
            include_slippage: true,
            include_latency_assumption: true,
            include_liquidity_filter,
            required_train_validation_split: false,
            max_adapter_runtime_minutes: 10,
        },
        source_independence: source_independence(),
        symbol_resolution_trace: Vec::new(),
        confidence_summary: BTreeMap::new(),
        observe_or_reject_reasons: Vec::new(),
        parent_artifact_ids: Vec::new(),
        storage_uri: "bundle-key".to_owned(),
        checksum: String::new(),
        idempotency_key: "idem".to_owned(),
    }
}

fn data_quality_summary() -> DataQualitySummaryRef {
    DataQualitySummaryRef {
        market_data_quality_summary_key: None,
        status: "present".to_owned(),
    }
}

fn source_independence() -> SourceIndependenceSummary {
    SourceIndependenceSummary {
        source_event_count: 1,
        independent_source_count: 1,
        official_source_present: true,
        duplicate_content_hashes: Vec::new(),
        syndicated_from: None,
        original_source_ids: vec!["official".to_owned()],
    }
}
