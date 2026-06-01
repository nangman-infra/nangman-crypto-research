use super::validate_bundle_admission;
use crate::model::{
    CANDIDATE_BUNDLE_SCHEMA_VERSION, CandidateClass, ConfidenceBand, DataQualitySummaryRef,
    IntelCandidateEvidenceBundle, SourceIndependenceSummary, SymbolResolutionTrace,
    ValidationRequirements,
};
use std::collections::BTreeMap;

#[test]
fn validation_reason_order_matches_admission_contract() {
    let mut bundle = valid_bundle();
    bundle.schema_version = "wrong_schema".to_owned();
    bundle.research_eligible = false;
    bundle.candidate_class = CandidateClass::Reject;
    bundle.forbidden_lookahead_boundary_ms = 1_299;
    bundle.fetched_at_ms = 1_400;
    bundle.symbol_universe_snapshot_id = " ".to_owned();
    bundle.universe_as_of_ms = 1_400;
    bundle.approved_universe_symbol = false;
    bundle.normalized_symbols.clear();
    bundle.allowed_horizons = vec!["7d".to_owned()];
    bundle.data_quality_summary.status.clear();
    bundle.data_quality_summary.market_data_quality_summary_key = None;
    bundle.source_independence.independent_source_count = 0;
    bundle.symbol_resolution_trace.clear();
    bundle.validation_requirements.required_adapters.clear();
    bundle.source_story_cluster_ids.clear();
    bundle.source_structured_packet_ids.clear();
    bundle.evidence_refs.clear();
    bundle.parent_artifact_ids.clear();

    let decision = validate_bundle_admission(&bundle);

    assert!(!decision.admitted);
    assert_eq!(
        decision.reason_codes,
        [
            "invalid_candidate_bundle_schema",
            "candidate_not_research_eligible",
            "candidate_class_not_research_eligible",
            "lookahead_boundary_mismatch",
            "decision_available_at_before_source_time",
            "missing_symbol_universe_snapshot_id",
            "universe_as_of_after_decision_available",
            "not_admitted_universe",
            "missing_symbol_set",
            "holding_horizon_contract_violation",
            "missing_data_quality_summary_status",
            "missing_data_quality_summary",
            "not_admitted_source_independence",
            "not_admitted_symbol_resolution",
            "native_replay_not_required",
            "missing_source_artifact_lineage",
        ]
    );
}

fn valid_bundle() -> IntelCandidateEvidenceBundle {
    IntelCandidateEvidenceBundle {
        candidate_id: "candidate".to_owned(),
        candidate_lifecycle_key: "candidate:v1".to_owned(),
        bundle_key: "candidate-evidence-bundle/part-000001.jsonl".to_owned(),
        producer_app: "test".to_owned(),
        producer_run_id: "run".to_owned(),
        created_at_ms: 1_300,
        event_time_ms: 1_000,
        published_at_ms: Some(1_050),
        fetched_at_ms: 1_100,
        structured_at_ms: 1_200,
        candidate_created_at_ms: 1_300,
        decision_available_at_ms: 1_300,
        forbidden_lookahead_boundary_ms: 1_300,
        schema_version: CANDIDATE_BUNDLE_SCHEMA_VERSION.to_owned(),
        scoring_policy_version: "policy".to_owned(),
        normalized_symbols: vec!["SUI".to_owned()],
        symbol_universe_snapshot_id: "universe".to_owned(),
        universe_as_of_ms: 1_200,
        approved_universe_symbol: true,
        event_types: vec!["project_notice".to_owned()],
        hypothesis_type: "event_reaction".to_owned(),
        allowed_horizons: vec!["1h".to_owned()],
        source_story_cluster_ids: vec!["cluster".to_owned()],
        source_structured_packet_ids: vec!["packet".to_owned()],
        source_context_flag_packet_ids: Vec::new(),
        evidence_refs: vec!["packet".to_owned()],
        metric_evidence: Vec::new(),
        data_quality_summary: DataQualitySummaryRef {
            market_data_quality_summary_key: Some("market-data-quality/summary.json".to_owned()),
            status: "available".to_owned(),
        },
        selected_market_artifacts: Vec::new(),
        candidate_class: CandidateClass::ResearchCandidate,
        candidate_score: 72,
        research_priority: "p0".to_owned(),
        research_eligible: true,
        validation_requirements: ValidationRequirements {
            required_adapters: vec!["native_replay".to_owned()],
            optional_adapters: Vec::new(),
            min_unseen_windows: 1,
            include_fee: true,
            include_slippage: true,
            include_latency_assumption: true,
            include_liquidity_filter: true,
            required_train_validation_split: true,
            max_adapter_runtime_minutes: 15,
        },
        source_independence: SourceIndependenceSummary {
            source_event_count: 1,
            independent_source_count: 1,
            official_source_present: true,
            duplicate_content_hashes: Vec::new(),
            syndicated_from: None,
            original_source_ids: vec!["official".to_owned()],
        },
        symbol_resolution_trace: vec![SymbolResolutionTrace {
            raw_mentions: vec!["SUI".to_owned()],
            resolved_project: Some("Sui".to_owned()),
            resolved_asset: Some("SUI".to_owned()),
            canonical_symbol: Some("SUI".to_owned()),
            venue_symbols: vec!["SUIUSDT".to_owned()],
            mapping_confidence: ConfidenceBand::Strong,
            ambiguity_reason: None,
        }],
        confidence_summary: BTreeMap::new(),
        observe_or_reject_reasons: Vec::new(),
        parent_artifact_ids: vec!["packet".to_owned()],
        storage_uri: "s3://bucket/key".to_owned(),
        checksum: "checksum".to_owned(),
        idempotency_key: "idem".to_owned(),
    }
}
