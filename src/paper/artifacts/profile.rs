use crate::model::{
    DEFAULT_PAPER_ACCOUNT_PROFILE_ID, DEFAULT_PAPER_FEE_MODEL_VERSION,
    DEFAULT_PAPER_SLIPPAGE_MODEL_VERSION, PAPER_ACCOUNT_PROFILE_SCHEMA_VERSION,
    PaperAccountProfile,
};

pub fn default_paper_account_profile() -> PaperAccountProfile {
    PaperAccountProfile {
        paper_account_profile_id: DEFAULT_PAPER_ACCOUNT_PROFILE_ID.to_owned(),
        virtual_starting_balance: 10_000.0,
        max_notional_per_candidate: 100.0,
        fee_model_version: DEFAULT_PAPER_FEE_MODEL_VERSION.to_owned(),
        slippage_model_version: DEFAULT_PAPER_SLIPPAGE_MODEL_VERSION.to_owned(),
        marking_frequency: "hourly".to_owned(),
        target_max_holding_hours: 24,
        absolute_max_holding_hours: 72,
        force_flat_policy: "daily_or_ttl_exit".to_owned(),
        schema_version: PAPER_ACCOUNT_PROFILE_SCHEMA_VERSION.to_owned(),
    }
}
