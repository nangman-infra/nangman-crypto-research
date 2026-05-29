use serde::{Deserialize, Serialize};

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
