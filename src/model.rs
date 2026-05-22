use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CANDIDATE_BUNDLE_SCHEMA_VERSION: &str = "intel_candidate_evidence_bundle_v1";
pub const RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION: &str = "research_input_manifest_v1";
pub const REPLAY_RUN_SCHEMA_VERSION: &str = "replay_run_v1";
pub const REPLAY_RUN_INDEX_SCHEMA_VERSION: &str = "replay_run_index_v1";
pub const RESEARCH_AGGREGATE_REGISTRY_SCHEMA_VERSION: &str =
    "research_aggregate_registry_record_v1";
pub const RESEARCH_RUN_REPORT_SCHEMA_VERSION: &str = "research_run_report_v1";
pub const NATIVE_REPLAY_ADAPTER: &str = "native_replay";
pub const DEFAULT_COST_MODEL_VERSION: &str = "research_cost_model_v0_2026_05_09";
pub const DEFAULT_VALIDATION_RECIPE_VERSION: &str = "native_replay_recipe_v0_2026_05_09";
pub const DEFAULT_RESEARCH_GATE_POLICY_VERSION: &str = "research_gate_policy_v1_2026_05_09";
pub const SHADOW_VALIDATION_RUN_SCHEMA_VERSION: &str = "shadow_validation_run_v1";
pub const OSS_ADAPTER_RUN_SCHEMA_VERSION: &str = "oss_adapter_run_v1";
pub const PORTFOLIO_ALLOCATION_SNAPSHOT_SCHEMA_VERSION: &str = "portfolio_allocation_snapshot_v1";
pub const PORTFOLIO_RISK_REJECT_EVENT_SCHEMA_VERSION: &str = "portfolio_risk_reject_event_v1";
pub const PORTFOLIO_REDUCE_ONLY_SIGNAL_SCHEMA_VERSION: &str = "portfolio_reduce_only_signal_v1";
pub const HOLDING_POLICY_VERSION: &str = "crypto_intraday_holding_policy_v1_2026_05_12";
pub const TARGET_MAX_HOLDING_HOURS: u32 = 24;
pub const ABSOLUTE_MAX_HOLDING_HOURS: u32 = 72;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResearchInputManifest {
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_packet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_scope: Option<String>,
    #[serde(default)]
    pub candidate_bundle_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub market_feature_delta_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub market_regime_context_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub hypothesis_harness_result_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub oss_adapter_run_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub historical_replay_run_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub historical_replay_run_index_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub runtime_budget_policy: ResearchRuntimeBudgetPolicy,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResearchArtifactRef {
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResearchRuntimeBudgetPolicy {
    #[serde(default = "default_max_candidate_bundle_count")]
    pub max_candidate_bundle_count: usize,
    #[serde(default = "default_max_market_artifact_ref_count")]
    pub max_market_artifact_ref_count: usize,
    #[serde(default = "default_max_hypothesis_harness_result_ref_count")]
    pub max_hypothesis_harness_result_ref_count: usize,
    #[serde(default = "default_max_oss_adapter_run_ref_count")]
    pub max_oss_adapter_run_ref_count: usize,
    #[serde(default = "default_max_historical_replay_run_ref_count")]
    pub max_historical_replay_run_ref_count: usize,
    #[serde(default = "default_max_replay_run_count")]
    pub max_replay_run_count: usize,
}

impl Default for ResearchRuntimeBudgetPolicy {
    fn default() -> Self {
        Self {
            max_candidate_bundle_count: default_max_candidate_bundle_count(),
            max_market_artifact_ref_count: default_max_market_artifact_ref_count(),
            max_hypothesis_harness_result_ref_count:
                default_max_hypothesis_harness_result_ref_count(),
            max_oss_adapter_run_ref_count: default_max_oss_adapter_run_ref_count(),
            max_historical_replay_run_ref_count: default_max_historical_replay_run_ref_count(),
            max_replay_run_count: default_max_replay_run_count(),
        }
    }
}

fn default_max_candidate_bundle_count() -> usize {
    500
}

fn default_max_market_artifact_ref_count() -> usize {
    2_000
}

fn default_max_hypothesis_harness_result_ref_count() -> usize {
    10_000
}

fn default_max_oss_adapter_run_ref_count() -> usize {
    10_000
}

fn default_max_historical_replay_run_ref_count() -> usize {
    10_000
}

fn default_max_replay_run_count() -> usize {
    20_000
}

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateClass {
    StrongCandidate,
    ResearchCandidate,
    WeakCandidate,
    ObserveOnly,
    Reject,
    Quarantine,
}

impl CandidateClass {
    pub fn is_research_eligible(&self) -> bool {
        matches!(self, Self::StrongCandidate | Self::ResearchCandidate)
    }

    pub fn as_report_key(&self) -> &'static str {
        match self {
            Self::StrongCandidate => "strong_candidate",
            Self::ResearchCandidate => "research_candidate",
            Self::WeakCandidate => "weak_candidate",
            Self::ObserveOnly => "observe_only",
            Self::Reject => "reject",
            Self::Quarantine => "quarantine",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceBand {
    Weak,
    Low,
    Moderate,
    Medium,
    Strong,
    High,
    #[default]
    Unknown,
}

impl ConfidenceBand {
    pub fn is_research_allowed(&self) -> bool {
        matches!(
            self,
            Self::Moderate | Self::Medium | Self::Strong | Self::High
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MetricEvidence {
    pub metric_name: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub previous_value: Option<f64>,
    #[serde(default)]
    pub delta_pct: Option<f64>,
    #[serde(default)]
    pub window_ms: Option<i64>,
    pub observed_at_ms: i64,
    pub source_event_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DataQualitySummaryRef {
    #[serde(default)]
    pub market_data_quality_summary_key: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SelectedMarketArtifactTrace {
    pub artifact_type: String,
    pub artifact_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_canonical: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    pub quality_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ValidationRequirements {
    #[serde(default)]
    pub required_adapters: Vec<String>,
    #[serde(default)]
    pub optional_adapters: Vec<String>,
    pub min_unseen_windows: usize,
    pub include_fee: bool,
    pub include_slippage: bool,
    pub include_latency_assumption: bool,
    pub include_liquidity_filter: bool,
    pub required_train_validation_split: bool,
    pub max_adapter_runtime_minutes: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SourceIndependenceSummary {
    pub source_event_count: usize,
    pub independent_source_count: usize,
    pub official_source_present: bool,
    #[serde(default)]
    pub duplicate_content_hashes: Vec<String>,
    #[serde(default)]
    pub syndicated_from: Option<String>,
    #[serde(default)]
    pub original_source_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SymbolResolutionTrace {
    #[serde(default)]
    pub raw_mentions: Vec<String>,
    #[serde(default)]
    pub resolved_project: Option<String>,
    #[serde(default)]
    pub resolved_asset: Option<String>,
    #[serde(default)]
    pub canonical_symbol: Option<String>,
    #[serde(default)]
    pub venue_symbols: Vec<String>,
    #[serde(default)]
    pub mapping_confidence: ConfidenceBand,
    #[serde(default)]
    pub ambiguity_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDelta {
    pub schema_version: String,
    pub feature_delta_id: String,
    pub l1_run_id: String,
    pub metric_name: String,
    pub venue: String,
    pub symbol_native: String,
    pub symbol_canonical: String,
    pub market_type: String,
    pub value_now: f64,
    #[serde(default)]
    pub value_15m_ago: Option<f64>,
    #[serde(default)]
    pub value_1h_ago: Option<f64>,
    #[serde(default)]
    pub change_pct_15m: Option<f64>,
    #[serde(default)]
    pub change_pct_1h: Option<f64>,
    #[serde(default)]
    pub price_change_same_window: Option<f64>,
    #[serde(default)]
    pub volume_change_same_window: Option<f64>,
    #[serde(default)]
    pub oi_price_divergence: Option<f64>,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    #[serde(default)]
    pub quality_status: String,
    #[serde(default)]
    pub missing_reasons: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketRegimeContext {
    pub schema_version: String,
    pub regime_context_id: String,
    pub l1_run_id: String,
    pub scope: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    #[serde(default)]
    pub btc_return_same_window: Option<f64>,
    #[serde(default)]
    pub eth_return_same_window: Option<f64>,
    #[serde(default)]
    pub sector_return_same_window: Option<f64>,
    pub volatility_regime: String,
    #[serde(default)]
    pub correlation_to_btc: Option<f64>,
    pub known_as_of_ms: i64,
    #[serde(default)]
    pub quality_status: String,
    #[serde(default)]
    pub missing_reasons: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchRunStatus {
    Completed,
    Partial,
    InvalidInput,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchBias {
    PruneBias,
    RetestBias,
    PromoteToShadowBias,
    PromoteToPaperBias,
}

impl ResearchBias {
    pub fn report_key(&self) -> &'static str {
        match self {
            Self::PruneBias => "PRUNE_BIAS",
            Self::RetestBias => "RETEST_BIAS",
            Self::PromoteToShadowBias => "PROMOTE_TO_SHADOW_BIAS",
            Self::PromoteToPaperBias => "PROMOTE_TO_PAPER_BIAS",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplayRunStatus {
    Completed,
    InvalidInput,
    MissingMarketReplayData,
    InsufficientEvidence,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReplayRun {
    pub replay_run_id: String,
    pub source_candidate_id: String,
    pub source_candidate_lifecycle_key: String,
    pub research_partition_key: String,
    pub research_aggregate_key: String,
    pub symbol_canonical: String,
    pub decision_available_at_ms: i64,
    pub symbol_universe_snapshot_id: String,
    pub universe_as_of_ms: i64,
    pub approved_universe_symbol: bool,
    pub hypothesis_type: String,
    pub validation_adapter: String,
    pub strategy_id_or_family: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub forbidden_lookahead_boundary_ms: i64,
    #[serde(default)]
    pub holding_policy: HoldingPolicy,
    pub data_quality_summary_ref: DataQualitySummaryRef,
    pub source_independence_summary: SourceIndependenceSummary,
    pub symbol_resolution_trace_ref: Vec<SymbolResolutionTrace>,
    pub parameter_variant_id: String,
    pub cost_model_version: String,
    pub validation_recipe_version: String,
    pub result_summary: ReplayResultSummary,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReplayRunIndexRecord {
    pub replay_run_index_record_id: String,
    pub research_run_report_id: String,
    pub research_packet_id: String,
    pub run_scope: String,
    pub replay_run_id: String,
    pub replay_run_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_run_s3_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_run_s3_key: Option<String>,
    pub source_candidate_id: String,
    pub source_candidate_lifecycle_key: String,
    pub research_partition_key: String,
    pub research_aggregate_key: String,
    pub symbol_canonical: String,
    pub decision_available_at_ms: i64,
    pub hypothesis_type: String,
    pub validation_adapter: String,
    pub strategy_id_or_family: String,
    pub parameter_variant_id: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub created_at_ms: i64,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReplayResultSummary {
    pub status: ReplayRunStatus,
    pub bias: ResearchBias,
    pub reason_codes: Vec<String>,
    pub matched_market_delta_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btc_adjusted_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_after_cost_bps: Option<f64>,
    pub estimated_cost_bps: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub market_regime_labels: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResearchRunReport {
    pub research_run_report_id: String,
    pub research_packet_id: String,
    pub source_candidate_ids: Vec<String>,
    pub run_scope: String,
    pub partition_count: usize,
    pub top_symbols: Vec<String>,
    pub top_families: Vec<String>,
    pub surviving_candidate_keys: Vec<String>,
    pub pruned_candidate_keys: Vec<String>,
    pub retest_candidate_keys: Vec<String>,
    pub shadow_validation_runs: Vec<ShadowValidationRun>,
    pub paper_trade_candidates: Vec<String>,
    pub oss_adapter_run_ids: Vec<String>,
    pub oss_adapter_reject_count: usize,
    pub portfolio_allocation_snapshot: Option<PortfolioAllocationSnapshot>,
    pub portfolio_risk_reject_events: Vec<PortfolioRiskRejectEvent>,
    pub portfolio_reduce_only_signals: Vec<PortfolioReduceOnlySignal>,
    pub hypothesis_outputs: HypothesisOutput,
    pub research_gate_policy: ResearchGatePolicy,
    pub partition_aggregates: Vec<ResearchPartitionAggregate>,
    pub summary_findings: Vec<SummaryFinding>,
    pub research_run_status: ResearchRunStatus,
    pub created_at_ms: i64,
    pub replay_run_ids: Vec<String>,
    pub invalid_input_candidate_keys: Vec<String>,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResearchGatePolicy {
    pub policy_version: String,
    pub min_completed_samples_for_shadow: usize,
    pub min_win_rate_ppm_for_shadow: u64,
    pub min_profit_factor_ppm_for_shadow: u64,
    pub min_mean_net_after_cost_bps_for_shadow: f64,
    pub max_missing_or_insufficient_ratio_ppm_for_shadow: u64,
    pub min_market_regime_label_count_for_shadow: usize,
    pub cost_stress_multiplier_for_shadow: f64,
    pub full_weight_sample_max_age_days: u64,
    pub decayed_sample_max_age_days: u64,
    pub expired_sample_max_age_days: u64,
    pub decayed_sample_weight: f64,
    pub stale_sample_weight: f64,
    pub allow_promote_to_paper_bias: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurvivalBand {
    Fragile,
    Conditional,
    Stable,
    Exceptional,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResearchPartitionAggregate {
    pub research_aggregate_key: String,
    pub research_partition_keys: Vec<String>,
    pub source_candidate_ids: Vec<String>,
    pub source_candidate_lifecycle_keys: Vec<String>,
    pub symbol_canonical: String,
    pub hypothesis_type: String,
    pub validation_adapter: String,
    pub strategy_id_or_family: String,
    pub parameter_variant_id: String,
    pub replay_run_count: usize,
    pub active_replay_run_count: usize,
    pub expired_replay_run_count: usize,
    pub completed_count: usize,
    pub decayed_completed_count: usize,
    pub expired_completed_count: usize,
    pub effective_completed_sample_weight: f64,
    pub invalid_input_count: usize,
    pub missing_market_replay_data_count: usize,
    pub insufficient_evidence_count: usize,
    pub positive_net_count: usize,
    pub non_positive_net_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_raw_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_btc_adjusted_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_net_after_cost_bps: Option<f64>,
    pub gross_positive_net_bps: f64,
    pub gross_negative_net_bps_abs: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_factor_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_profit_factor_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_stressed_mean_net_after_cost_bps: Option<f64>,
    pub distinct_replay_window_count: usize,
    pub inferred_unseen_window_count: usize,
    pub market_regime_labels: Vec<String>,
    pub regime_summaries: Vec<RegimeReplaySummary>,
    pub train_validation_split_summary: TrainValidationSplitSummary,
    pub survival_band: SurvivalBand,
    pub gate_bias: ResearchBias,
    pub gate_reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchAggregateRegistryStage {
    Pruned,
    Retest,
    ShadowCandidate,
    PaperCandidateBias,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResearchAggregateRegistryRecord {
    pub research_aggregate_registry_record_id: String,
    pub research_run_report_id: String,
    pub research_packet_id: String,
    pub run_scope: String,
    pub research_aggregate_key: String,
    pub source_candidate_ids: Vec<String>,
    pub source_candidate_lifecycle_keys: Vec<String>,
    pub symbol_canonical: String,
    pub hypothesis_type: String,
    pub validation_adapter: String,
    pub strategy_id_or_family: String,
    pub parameter_variant_id: String,
    pub current_research_stage: ResearchAggregateRegistryStage,
    pub gate_bias: ResearchBias,
    pub survival_band: SurvivalBand,
    pub replay_run_count: usize,
    pub active_replay_run_count: usize,
    pub expired_replay_run_count: usize,
    pub completed_count: usize,
    pub effective_completed_sample_weight: f64,
    pub positive_net_count: usize,
    pub non_positive_net_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_profit_factor_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_stressed_mean_net_after_cost_bps: Option<f64>,
    pub market_regime_labels: Vec<String>,
    pub latest_reason_codes: Vec<String>,
    pub linked_shadow_validation_run_ids: Vec<String>,
    pub created_at_ms: i64,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RegimeReplaySummary {
    pub regime_label: String,
    pub completed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_net_after_cost_bps: Option<f64>,
    pub positive_net_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TrainValidationSplitSummary {
    pub required: bool,
    pub materialized: bool,
    pub train_completed_count: usize,
    pub validation_completed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub train_mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_mean_net_after_cost_bps: Option<f64>,
    pub train_positive_net_count: usize,
    pub validation_positive_net_count: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ShadowValidationRun {
    pub shadow_validation_run_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub trigger_research_run_id: String,
    pub start_condition_summary: ShadowStartConditionSummary,
    pub expected_survival_band: SurvivalBand,
    pub watch_window_policy: ShadowWatchWindowPolicy,
    pub termination_policy: ShadowTerminationPolicy,
    pub holding_policy: HoldingPolicy,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HoldingPolicy {
    pub target_max_holding_hours: u32,
    pub absolute_max_holding_hours: u32,
    pub absolute_exit_deadline_ms: i64,
    pub force_flat_policy: String,
    pub overnight_risk_exception: bool,
    pub holding_policy_version: String,
}

impl Default for HoldingPolicy {
    fn default() -> Self {
        Self {
            target_max_holding_hours: TARGET_MAX_HOLDING_HOURS,
            absolute_max_holding_hours: ABSOLUTE_MAX_HOLDING_HOURS,
            absolute_exit_deadline_ms: 0,
            force_flat_policy: "daily_or_ttl_exit".to_owned(),
            overnight_risk_exception: false,
            holding_policy_version: HOLDING_POLICY_VERSION.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ShadowStartConditionSummary {
    pub research_aggregate_key: String,
    pub gate_policy_version: String,
    pub completed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_factor_ppm: Option<u64>,
    pub gate_reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShadowWatchWindowPolicy {
    pub mode: String,
    pub min_shadow_samples: usize,
    pub max_shadow_age_days: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShadowTerminationPolicy {
    pub prune_on_non_positive_mean_net: bool,
    pub prune_on_max_age_without_samples: bool,
    pub no_order_execution: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisOutput {
    None,
    L1SummaryOnly,
    L2HypothesisAttached,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SummaryFinding {
    pub candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub bias: ResearchBias,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct OssAdapterRun {
    pub oss_adapter_run_id: String,
    pub adapter_name: String,
    pub adapter_version: String,
    pub candidate_lifecycle_key: String,
    #[serde(default)]
    pub input_artifact_refs: Vec<String>,
    pub market_window: String,
    pub fee_model_used: String,
    pub slippage_model_used: String,
    pub trade_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_drawdown_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharpe_like_score: Option<f64>,
    pub lookahead_check_result: String,
    pub holding_horizon_check_result: String,
    #[serde(default)]
    pub adapter_warnings: Vec<String>,
    pub normalized_verdict_bias: OssAdapterVerdictBias,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OssAdapterVerdictBias {
    PruneBias,
    RetestBias,
    PromoteToReplayBias,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PortfolioAllocationSnapshot {
    pub portfolio_allocation_snapshot_id: String,
    pub schema_version: String,
    pub allocation_policy_version: String,
    pub computed_at_ms: i64,
    pub market_regime: String,
    pub active_candidate_count: usize,
    pub max_total_notional_pct: f64,
    pub max_symbol_notional_pct: f64,
    pub max_candidate_notional_pct: f64,
    pub max_regime_notional_pct: f64,
    pub candidate_allocations: Vec<CandidateAllocation>,
    pub rejected_candidates: Vec<PortfolioRiskRejectEvent>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CandidateAllocation {
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub strategy_id: String,
    pub allocation_weight: f64,
    pub max_notional_pct: f64,
    pub correlation_bucket: String,
    pub holding_deadline_ms: i64,
    pub paper_survival_band: SurvivalBand,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PortfolioRiskRejectEvent {
    pub portfolio_risk_reject_event_id: String,
    pub schema_version: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub reason: String,
    pub computed_at_ms: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PortfolioReduceOnlySignal {
    pub portfolio_reduce_only_signal_id: String,
    pub schema_version: String,
    pub symbol_canonical: String,
    pub reason: String,
    pub computed_at_ms: i64,
}
