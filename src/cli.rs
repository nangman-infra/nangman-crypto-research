use crate::admission::{horizon_ms, validate_bundle_admission};
use crate::alert::{
    emit_paper_watch_live_mark_alert_from_env, emit_research_report_alert_from_env,
    emit_shadow_cycle_decision_alert_from_env,
};
use crate::error::{AppError, AppResult};
use crate::focused_retest::{
    FocusedRetestBuildOptions, FocusedRetestManifestBuild, build_focused_retest_manifest,
};
use crate::hash::stable_id;
use crate::io::{
    ResearchOutputArtifacts, read_candidate_bundles, read_market_feature_deltas,
    read_market_regime_contexts, read_oss_adapter_runs, read_replay_run_index_records,
    read_replay_runs, read_research_input_manifest, read_research_run_report,
    read_shadow_validation_runs, write_paper_watch_live_marks, write_pretty_json_file,
    write_research_input_manifest, write_research_outputs, write_shadow_cycle_decision,
    write_shadow_cycle_decision_to_dir,
};
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketRegimeContext,
    OSS_ADAPTER_RUN_SCHEMA_VERSION, OssAdapterRun, RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION,
    RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION, ReplayRun, ReplayRunIndexRecord, ResearchArtifactRef,
    ResearchInputManifest, ResearchRuntimeBudgetPolicy, RetestCycleSourceState,
    RetestCycleSourceStateSafety, SelectedMarketArtifactTrace, ShadowCycleSchedulerAction,
    ShadowValidationRun,
};
use crate::paper::{build_paper_artifacts, build_paper_watch_candidates};
use crate::paper_live::{
    DEFAULT_MARKET_LIVE_NATS_CONSUMER, DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY,
    DEFAULT_MARKET_LIVE_NATS_SUBJECT, MarketLiveNatsConfig, build_paper_watch_live_marks,
    read_market_live_ticks, read_market_live_ticks_from_nats, read_paper_watch_candidates,
};
use crate::paper_watch_observer::{
    PAPER_WATCH_OBSERVER_SNAPSHOT_SCHEMA_VERSION, PaperWatchObserverState, active_candidates,
};
use crate::replay::{build_invalid_replay_run, run_native_replay};
use crate::report::build_report;
use crate::retest_cycle::{read_retest_horizon_status, validate_retest_horizon_status};
use crate::retest_plan::{RetestHorizonPlanBuildOptions, build_retest_horizon_plan};
use crate::retest_status::{
    RetestHorizonStatusBuildOptions, build_retest_horizon_status, read_retest_horizon_plan,
};
use crate::shadow_cycle::{
    build_shadow_cycle_decision, read_shadow_cycle_decision, shadow_sample_deficit_lifecycle_keys,
    validate_shadow_cycle_decision,
};
use crate::storage::{
    discover_latest_market_feature_delta_keys_from_s3,
    discover_latest_market_regime_context_keys_from_s3,
    discover_latest_symbol_universe_snapshot_end_ms_from_s3,
    discover_paper_watch_candidate_keys_from_s3, discover_paper_watch_live_mark_keys_from_s3,
    discover_replay_run_index_keys_from_s3, discover_shadow_validation_run_keys_from_s3,
    hourly_partitioned_prefix, read_candidate_bundles_from_s3,
    read_latest_retest_cycle_source_state_from_s3, read_latest_retest_horizon_status_from_s3,
    read_market_feature_deltas_from_s3, read_market_regime_contexts_from_s3,
    read_oss_adapter_runs_from_s3, read_paper_watch_candidates_from_s3,
    read_paper_watch_live_marks_from_s3, read_replay_run_index_records_from_s3,
    read_replay_runs_from_s3, read_research_input_manifest_from_s3,
    read_research_run_report_from_s3, read_retest_horizon_plan_from_s3,
    read_retest_horizon_status_from_s3, read_shadow_validation_runs_from_s3,
    write_paper_watch_live_marks_to_s3, write_paper_watch_observer_snapshot_to_s3,
    write_research_input_manifest_to_exact_s3_key_if_absent, write_research_input_manifest_to_s3,
    write_research_outputs_to_s3, write_retest_cycle_source_state_to_s3,
    write_retest_horizon_plan_to_s3, write_retest_horizon_status_to_s3,
    write_shadow_cycle_decision_to_s3,
};
use crate::time::now_ms;
mod args;
pub use args::{Args, parse_args};
mod environment;
use environment::*;
mod help;
pub(crate) use help::help_text;
pub use help::print_help;
mod inputs;
use inputs::*;
mod paper_watch;
use paper_watch::*;
mod retest;
use retest::*;
mod run;
pub use run::run;
mod shadow_cycle_mode;
use shadow_cycle_mode::*;
mod summary;
pub use summary::RunSummary;
mod validation;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use validation::*;

const DEFAULT_MARKET_L1_S3_BUCKET: &str = "nangman-crypto-dev-market-ingest-l1-<account-suffix>";
const MARKET_FEATURE_DELTA_ARTIFACT_TYPE: &str = "market_feature_delta";
const MARKET_FEATURE_DELTA_SUMMARY_ARTIFACT_TYPE: &str = "market_feature_delta_summary";
const MARKET_REGIME_CONTEXT_ARTIFACT_TYPE: &str = "market_regime_context";
const MARKET_L1_REPLAY_WINDOW_MS: i64 = 15 * 60 * 1000;
const DEFAULT_HISTORICAL_REPLAY_RUN_INDEX_READ_LIMIT: usize = 20;
const DEFAULT_HISTORICAL_REPLAY_RUN_INDEX_SCAN_LIMIT: usize = 1_000;
const DEFAULT_SHADOW_VALIDATION_RUN_READ_LIMIT: usize = 100;
const DEFAULT_SHADOW_VALIDATION_RUN_SCAN_LIMIT: usize = 1_000;
const DEFAULT_SHADOW_VALIDATION_RUN_PREFIX: &str =
    "shadow-validation-run/schema=shadow_validation_run_v1";
const DEFAULT_PAPER_WATCH_CANDIDATE_PREFIX: &str =
    "paper-watch-candidate/schema=paper_watch_candidate_v1";
const DEFAULT_PAPER_WATCH_LIVE_MARK_PREFIX: &str =
    "paper-watch-live-mark/schema=paper_watch_live_mark_v1";
const DEFAULT_PAPER_WATCH_OBSERVER_OUTPUT_PREFIX: &str =
    "paper-watch-observer-state/schema=paper_watch_observer_snapshot_v1";
const DEFAULT_PAPER_WATCH_OBSERVER_READ_LIMIT: usize = 100;
const DEFAULT_PAPER_WATCH_OBSERVER_SCAN_LIMIT: usize = 2_000;
const DEFAULT_PAPER_WATCH_OBSERVER_POLL_SECS: u64 = 5;

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
