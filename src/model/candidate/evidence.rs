use super::ConfidenceBand;
use serde::{Deserialize, Serialize};

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
