use crate::error::{AppError, AppResult};
use crate::io::{
    read_candidate_bundles_from_bytes, read_market_feature_deltas_matching_symbols_from_bytes,
    read_market_live_ticks_from_bytes, read_market_regime_contexts_from_bytes,
    read_oss_adapter_runs_from_bytes, read_paper_watch_candidates_from_bytes,
    read_paper_watch_live_marks_from_bytes, read_replay_run_index_records_from_bytes,
    read_replay_runs_from_bytes, read_research_input_manifest_from_bytes,
    read_research_run_report_from_bytes, read_shadow_validation_runs_from_bytes,
};
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketLiveTick, MarketRegimeContext,
    OssAdapterRun, PaperWatchCandidate, PaperWatchLiveMark,
    RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION, ReplayRun, ReplayRunIndexRecord,
    ResearchInputManifest, ResearchRunReport, RetestCycleSourceState, ShadowValidationRun,
};
use crate::retest_cycle::read_retest_horizon_status_from_bytes;
use crate::retest_status::read_retest_horizon_plan_from_bytes;
use std::collections::BTreeSet;

mod client;
mod listing;
mod market_l1;
mod objects;
mod partition;
mod read;
mod write;

use client::{aws_error_detail, s3_client};
use listing::{
    discover_latest_part_jsonl_keys_from_s3, list_payload_objects_with_prefix,
    select_latest_payload_keys,
};
use market_l1::discover_latest_market_l1_keys_from_s3;
use objects::{get_object_bytes, is_missing_market_artifact};
pub use partition::hourly_partitioned_prefix;
use partition::normalize_prefix;

pub use read::{
    discover_latest_market_feature_delta_keys_from_s3,
    discover_latest_market_regime_context_keys_from_s3,
    discover_latest_symbol_universe_snapshot_end_ms_from_s3,
    discover_paper_watch_candidate_keys_from_s3, discover_paper_watch_live_mark_keys_from_s3,
    discover_replay_run_index_keys_from_s3, discover_shadow_validation_run_keys_from_s3,
    read_candidate_bundles_from_s3, read_latest_retest_cycle_source_state_from_s3,
    read_latest_retest_horizon_status_from_s3, read_market_feature_deltas_from_s3,
    read_market_live_ticks_from_s3, read_market_regime_contexts_from_s3,
    read_oss_adapter_runs_from_s3, read_paper_watch_candidates_from_s3,
    read_paper_watch_live_marks_from_s3, read_replay_run_index_records_from_s3,
    read_replay_runs_from_s3, read_research_input_manifest_from_s3,
    read_research_run_report_from_s3, read_retest_horizon_plan_from_s3,
    read_retest_horizon_status_from_s3, read_shadow_validation_runs_from_s3,
};

pub use write::{
    write_paper_watch_live_marks_to_s3, write_paper_watch_observer_snapshot_to_s3,
    write_research_input_manifest_to_exact_s3_key_if_absent, write_research_input_manifest_to_s3,
    write_research_outputs_to_s3, write_retest_cycle_source_state_to_s3,
    write_retest_horizon_plan_to_s3, write_retest_horizon_status_to_s3,
    write_shadow_cycle_decision_to_s3,
};
