use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RetestCycleSourceState {
    pub schema_version: String,
    pub generated_at_ms: i64,
    pub research_packet_id: String,
    pub run_scope: String,
    pub source_manifest_s3_bucket: String,
    pub source_manifest_s3_key: String,
    pub source_research_report_s3_bucket: String,
    pub source_research_report_s3_key: String,
    pub source_research_report_id: String,
    pub source_candidate_ids: Vec<String>,
    pub replay_run_id_count: usize,
    pub summary_findings_count: usize,
    pub shadow_validation_run_count: usize,
    pub paper_trade_candidate_count: usize,
    pub safety: RetestCycleSourceStateSafety,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RetestCycleSourceStateSafety {
    pub dispatcher_prefix: String,
    pub state_s3_write: bool,
    pub ecs_task_started: bool,
    pub shadow_paper_live_enabled: bool,
}
