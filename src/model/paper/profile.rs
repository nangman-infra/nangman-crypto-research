use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperAccountProfile {
    pub paper_account_profile_id: String,
    pub virtual_starting_balance: f64,
    pub max_notional_per_candidate: f64,
    pub fee_model_version: String,
    pub slippage_model_version: String,
    pub marking_frequency: String,
    pub target_max_holding_hours: u32,
    pub absolute_max_holding_hours: u32,
    pub force_flat_policy: String,
    pub schema_version: String,
}
