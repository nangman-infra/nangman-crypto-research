use super::horizon::horizon_ms;
use super::symbol_mapping::has_allowed_symbol_mapping;
use super::types::AdmissionDecision;
use crate::holding::horizon_within_absolute_limit;
use crate::model::{CANDIDATE_BUNDLE_SCHEMA_VERSION, IntelCandidateEvidenceBundle};

pub fn validate_bundle_admission(bundle: &IntelCandidateEvidenceBundle) -> AdmissionDecision {
    let mut reasons = Vec::new();

    require(
        bundle.schema_version == CANDIDATE_BUNDLE_SCHEMA_VERSION,
        "invalid_candidate_bundle_schema",
        &mut reasons,
    );
    require(
        bundle.research_eligible,
        "candidate_not_research_eligible",
        &mut reasons,
    );
    require(
        bundle.candidate_class.is_research_eligible(),
        "candidate_class_not_research_eligible",
        &mut reasons,
    );
    require(
        bundle.forbidden_lookahead_boundary_ms == bundle.decision_available_at_ms,
        "lookahead_boundary_mismatch",
        &mut reasons,
    );
    require(
        bundle.decision_available_at_ms
            >= bundle
                .published_at_ms
                .unwrap_or(bundle.decision_available_at_ms)
                .max(bundle.fetched_at_ms)
                .max(bundle.structured_at_ms)
                .max(bundle.candidate_created_at_ms),
        "decision_available_at_before_source_time",
        &mut reasons,
    );
    require(
        !bundle.symbol_universe_snapshot_id.trim().is_empty(),
        "missing_symbol_universe_snapshot_id",
        &mut reasons,
    );
    require(
        bundle.universe_as_of_ms <= bundle.decision_available_at_ms,
        "universe_as_of_after_decision_available",
        &mut reasons,
    );
    require(
        bundle.approved_universe_symbol,
        "not_admitted_universe",
        &mut reasons,
    );
    require(
        !bundle.normalized_symbols.is_empty(),
        "missing_symbol_set",
        &mut reasons,
    );
    require(
        !bundle.allowed_horizons.is_empty()
            && bundle
                .allowed_horizons
                .iter()
                .any(|value| horizon_ms(value).is_some()),
        "missing_replay_time_range",
        &mut reasons,
    );
    require(
        bundle
            .allowed_horizons
            .iter()
            .filter_map(|value| horizon_ms(value))
            .all(horizon_within_absolute_limit),
        "holding_horizon_contract_violation",
        &mut reasons,
    );
    require(
        !bundle.data_quality_summary.status.trim().is_empty(),
        "missing_data_quality_summary_status",
        &mut reasons,
    );
    require(
        bundle
            .data_quality_summary
            .market_data_quality_summary_key
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
        "missing_data_quality_summary",
        &mut reasons,
    );
    require(
        bundle.source_independence.independent_source_count > 0,
        "not_admitted_source_independence",
        &mut reasons,
    );
    require(
        has_allowed_symbol_mapping(bundle),
        "not_admitted_symbol_resolution",
        &mut reasons,
    );
    require(
        bundle
            .validation_requirements
            .required_adapters
            .iter()
            .any(|adapter| adapter == "native_replay"),
        "native_replay_not_required",
        &mut reasons,
    );
    require(
        !bundle.source_structured_packet_ids.is_empty()
            || !bundle.parent_artifact_ids.is_empty()
            || !bundle.evidence_refs.is_empty()
            || !bundle.source_story_cluster_ids.is_empty(),
        "missing_source_artifact_lineage",
        &mut reasons,
    );

    AdmissionDecision {
        admitted: reasons.is_empty(),
        reason_codes: reasons,
    }
}

fn require(condition: bool, reason: &'static str, reasons: &mut Vec<String>) {
    if !condition {
        reasons.push(reason.to_owned());
    }
}
