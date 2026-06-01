use super::config::{deliver_policy, validate_nats_config};
use super::io::read_json_array_or_jsonl_bytes;
use super::nats::validate_tick;
use super::*;
use crate::model::{
    MARKET_LIVE_TICK_SCHEMA_VERSION, MarketLiveTick, PaperExpectedCostProfile,
    PaperExpectedRiskProfile, PaperWatchCandidate, PaperWatchReplaySampleSummary, PaperWatchSafety,
    ResearchBias, SurvivalBand,
};

#[path = "tests/fixtures.rs"]
mod fixtures;
#[path = "tests/io_contract.rs"]
mod io_contract;
#[path = "tests/lifecycle.rs"]
mod lifecycle;
#[path = "tests/marks.rs"]
mod marks;
#[path = "tests/nats_validation.rs"]
mod nats_validation;

use fixtures::*;
