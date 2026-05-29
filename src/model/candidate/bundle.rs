use super::{
    CandidateClass, DataQualitySummaryRef, MetricEvidence, SelectedMarketArtifactTrace,
    SourceIndependenceSummary, SymbolResolutionTrace, ValidationRequirements,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IntelCandidateEvidenceBundle {
    pub candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub bundle_key: String,
    pub producer_app: String,
    pub producer_run_id: String,
    pub created_at_ms: i64,
    pub event_time_ms: i64,
    #[serde(default)]
    pub published_at_ms: Option<i64>,
    pub fetched_at_ms: i64,
    pub structured_at_ms: i64,
    pub candidate_created_at_ms: i64,
    pub decision_available_at_ms: i64,
    pub forbidden_lookahead_boundary_ms: i64,
    pub schema_version: String,
    pub scoring_policy_version: String,
    #[serde(default)]
    pub normalized_symbols: Vec<String>,
    pub symbol_universe_snapshot_id: String,
    pub universe_as_of_ms: i64,
    pub approved_universe_symbol: bool,
    #[serde(default)]
    pub event_types: Vec<String>,
    pub hypothesis_type: String,
    #[serde(default)]
    pub allowed_horizons: Vec<String>,
    #[serde(default)]
    pub source_story_cluster_ids: Vec<String>,
    #[serde(default)]
    pub source_structured_packet_ids: Vec<String>,
    #[serde(default)]
    pub source_context_flag_packet_ids: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub metric_evidence: Vec<MetricEvidence>,
    pub data_quality_summary: DataQualitySummaryRef,
    #[serde(default)]
    pub selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    pub candidate_class: CandidateClass,
    pub candidate_score: i64,
    pub research_priority: String,
    pub research_eligible: bool,
    pub validation_requirements: ValidationRequirements,
    pub source_independence: SourceIndependenceSummary,
    #[serde(default)]
    pub symbol_resolution_trace: Vec<SymbolResolutionTrace>,
    #[serde(default)]
    pub confidence_summary: BTreeMap<String, String>,
    #[serde(default)]
    pub observe_or_reject_reasons: Vec<String>,
    #[serde(default)]
    pub parent_artifact_ids: Vec<String>,
    #[serde(default)]
    pub storage_uri: String,
    #[serde(default)]
    pub checksum: String,
    pub idempotency_key: String,
}
