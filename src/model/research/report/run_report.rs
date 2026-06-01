use serde::{Deserialize, Serialize};

use super::{ResearchGatePolicy, ResearchPartitionAggregate};
use crate::model::{
    HypothesisOutput, PortfolioAllocationSnapshot, PortfolioReduceOnlySignal,
    PortfolioRiskRejectEvent, ResearchRunStatus, ShadowValidationRun, SummaryFinding,
};

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
    #[serde(default)]
    pub paper_watch_candidates: Vec<String>,
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
