use crate::admission::{horizon_ms, validate_bundle_admission};
use crate::alert::{
    emit_research_report_alert_from_env, emit_shadow_cycle_decision_alert_from_env,
};
use crate::error::{AppError, AppResult};
use crate::focused_retest::{
    FocusedRetestBuildOptions, FocusedRetestManifestBuild, HistoricalReplayIndexRefMode,
    build_focused_retest_manifest, default_focused_retest_actions, parse_focused_retest_actions,
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
    DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS, DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE,
    DEFAULT_MARKET_LIVE_NATS_CONSUMER, DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY,
    DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES, DEFAULT_MARKET_LIVE_NATS_STREAM,
    DEFAULT_MARKET_LIVE_NATS_SUBJECT, MarketLiveNatsConfig, build_paper_watch_live_marks,
    read_market_live_ticks, read_market_live_ticks_from_nats, read_paper_watch_candidates,
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
    discover_replay_run_index_keys_from_s3, discover_shadow_validation_run_keys_from_s3,
    read_candidate_bundles_from_s3, read_latest_retest_cycle_source_state_from_s3,
    read_latest_retest_horizon_status_from_s3, read_market_feature_deltas_from_s3,
    read_market_regime_contexts_from_s3, read_oss_adapter_runs_from_s3,
    read_paper_watch_candidates_from_s3, read_replay_run_index_records_from_s3,
    read_replay_runs_from_s3, read_research_input_manifest_from_s3,
    read_research_run_report_from_s3, read_retest_horizon_plan_from_s3,
    read_retest_horizon_status_from_s3, read_shadow_validation_runs_from_s3,
    write_paper_watch_live_marks_to_s3, write_research_input_manifest_to_exact_s3_key_if_absent,
    write_research_input_manifest_to_s3, write_research_outputs_to_s3,
    write_retest_cycle_source_state_to_s3, write_retest_horizon_plan_to_s3,
    write_retest_horizon_status_to_s3, write_shadow_cycle_decision_to_s3,
};
use crate::time::now_ms;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub build_shadow_cycle_decision: bool,
    pub run_shadow_cycle_from_latest_state: bool,
    pub build_retest_horizon_plan: bool,
    pub run_retest_refresh_cycle: bool,
    pub run_retest_refresh_cycle_from_latest_state: bool,
    pub run_retest_cycle_scheduler: bool,
    pub build_retest_horizon_status: bool,
    pub build_focused_retest_manifest: bool,
    pub run_paper_watch_live_cycle: bool,
    pub shadow_cycle_decision_file: Option<PathBuf>,
    pub shadow_cycle_decision_output_file: Option<PathBuf>,
    pub shadow_cycle_latest_l1_as_of_ms: Option<i64>,
    pub retest_horizon_plan_file: Option<PathBuf>,
    pub retest_horizon_plan_s3_bucket: Option<String>,
    pub retest_horizon_plan_s3_key: Option<String>,
    pub retest_horizon_plan_output_file: Option<PathBuf>,
    pub retest_horizon_latest_l1_as_of_ms: Option<i64>,
    pub retest_horizon_status_output_file: Option<PathBuf>,
    pub retest_driver_summary_file: Option<PathBuf>,
    pub retest_horizon_status_file: Option<PathBuf>,
    pub retest_horizon_status_s3_bucket: Option<String>,
    pub retest_horizon_status_s3_key: Option<String>,
    pub focused_retest_manifest_output_file: Option<PathBuf>,
    pub focused_retest_summary_output_file: Option<PathBuf>,
    pub focused_retest_next_actions: Vec<String>,
    pub focused_retest_historical_replay_index_ref_mode: HistoricalReplayIndexRefMode,
    pub input_manifest_file: Option<PathBuf>,
    pub input_manifest_s3_bucket: Option<String>,
    pub input_manifest_s3_key: Option<String>,
    pub research_report_file: Option<PathBuf>,
    pub research_report_s3_bucket: Option<String>,
    pub research_report_s3_key: Option<String>,
    pub input_bundle_file: Option<PathBuf>,
    pub input_bundle_s3_bucket: Option<String>,
    pub input_bundle_s3_key: Option<String>,
    pub market_feature_delta_file: Option<PathBuf>,
    pub market_regime_context_file: Option<PathBuf>,
    pub market_l1_s3_bucket: Option<String>,
    pub market_feature_delta_s3_keys: Vec<String>,
    pub market_regime_context_s3_keys: Vec<String>,
    pub historical_replay_run_files: Vec<PathBuf>,
    pub historical_replay_run_index_files: Vec<PathBuf>,
    pub oss_adapter_run_files: Vec<PathBuf>,
    pub shadow_validation_run_files: Vec<PathBuf>,
    pub oss_adapter_run_s3_bucket: Option<String>,
    pub oss_adapter_run_s3_keys: Vec<String>,
    pub shadow_validation_run_s3_bucket: Option<String>,
    pub shadow_validation_run_s3_keys: Vec<String>,
    pub paper_watch_candidate_file: Option<PathBuf>,
    pub paper_watch_candidate_s3_bucket: Option<String>,
    pub paper_watch_candidate_s3_key: Option<String>,
    pub market_live_tick_file: Option<PathBuf>,
    pub market_live_nats_url: Option<String>,
    pub market_live_nats_stream: String,
    pub market_live_nats_subject: String,
    pub market_live_nats_consumer: String,
    pub market_live_nats_deliver_policy: String,
    pub market_live_nats_batch_size: usize,
    pub market_live_nats_max_messages: usize,
    pub market_live_nats_ack_wait_secs: u64,
    pub historical_replay_run_s3_bucket: Option<String>,
    pub historical_replay_run_s3_keys: Vec<String>,
    pub historical_replay_run_index_s3_bucket: Option<String>,
    pub historical_replay_run_index_s3_keys: Vec<String>,
    pub output_dir: Option<PathBuf>,
    pub output_s3_bucket: Option<String>,
    pub output_s3_prefix: Option<String>,
    pub research_packet_id: String,
    pub run_scope: String,
    pub now_ms: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct RunSummary {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub retest_horizon_plans_created: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub retest_horizon_statuses_validated: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retest_cycle_scheduler_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retest_cycle_run_not_before_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub focused_retest_manifests_created: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub focused_retest_horizon_count: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub focused_retest_candidate_bundle_refs: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub shadow_cycle_decisions_validated: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub shadow_cycle_decisions_created: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_cycle_scheduler_action: Option<ShadowCycleSchedulerAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_cycle_run_not_before_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_cycle_focused_research_manifest_file: Option<String>,
    pub processed_bundles: usize,
    pub replay_runs_created: usize,
    pub historical_replay_runs_loaded: usize,
    pub oss_adapter_runs_loaded: usize,
    pub shadow_validation_runs_loaded: usize,
    pub shadow_validation_runs_created: usize,
    pub paper_trade_candidates_created: usize,
    pub paper_trade_runs_created: usize,
    pub paper_trade_summaries_created: usize,
    pub paper_trade_marks_created: usize,
    pub paper_watch_live_marks_created: usize,
    pub portfolio_risk_reject_events_created: usize,
    pub portfolio_reduce_only_signals_created: usize,
    pub output_files: Vec<String>,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

pub fn parse_args<I>(mut values: I) -> AppResult<Option<Args>>
where
    I: Iterator<Item = String>,
{
    let shadow_cycle_latest_l1_as_of_ms =
        env_non_negative_i64("RESEARCH_SHADOW_CYCLE_LATEST_L1_AS_OF_MS")?;
    let retest_horizon_latest_l1_as_of_ms =
        env_non_negative_i64("RESEARCH_RETEST_PLAN_LATEST_L1_AS_OF_MS")?;
    let focused_retest_historical_replay_index_ref_mode =
        env_string("RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS")
            .map(|value| HistoricalReplayIndexRefMode::parse(&value))
            .transpose()?
            .unwrap_or(HistoricalReplayIndexRefMode::Auto);
    let focused_retest_next_actions = env_string("RESEARCH_FOCUS_NEXT_ACTIONS")
        .map(|value| parse_focused_retest_actions(&value))
        .filter(|actions| !actions.is_empty())
        .unwrap_or_else(default_focused_retest_actions);
    let mut args = Args {
        build_shadow_cycle_decision: env_bool("RESEARCH_BUILD_SHADOW_CYCLE_DECISION"),
        run_shadow_cycle_from_latest_state: env_bool("RESEARCH_RUN_SHADOW_CYCLE_FROM_LATEST_STATE"),
        build_retest_horizon_plan: env_bool("RESEARCH_BUILD_RETEST_HORIZON_PLAN"),
        run_retest_refresh_cycle: env_bool("RESEARCH_RUN_RETEST_REFRESH_CYCLE"),
        run_retest_refresh_cycle_from_latest_state: env_bool(
            "RESEARCH_RUN_RETEST_REFRESH_CYCLE_FROM_LATEST_STATE",
        ),
        run_retest_cycle_scheduler: env_bool("RESEARCH_RUN_RETEST_CYCLE_SCHEDULER"),
        build_retest_horizon_status: env_bool("RESEARCH_BUILD_RETEST_HORIZON_STATUS"),
        build_focused_retest_manifest: env_bool("RESEARCH_BUILD_FOCUSED_RETEST_MANIFEST"),
        run_paper_watch_live_cycle: env_bool("RESEARCH_RUN_PAPER_WATCH_LIVE_CYCLE"),
        shadow_cycle_decision_file: None,
        shadow_cycle_decision_output_file: env_string("RESEARCH_SHADOW_CYCLE_DECISION_OUTPUT_FILE")
            .map(PathBuf::from),
        shadow_cycle_latest_l1_as_of_ms,
        retest_horizon_plan_file: env_string("RESEARCH_RETEST_HORIZON_PLAN_FILE")
            .map(PathBuf::from),
        retest_horizon_plan_s3_bucket: env_string("RESEARCH_RETEST_HORIZON_PLAN_S3_BUCKET"),
        retest_horizon_plan_s3_key: env_string("RESEARCH_RETEST_HORIZON_PLAN_S3_KEY"),
        retest_horizon_plan_output_file: env_string("RESEARCH_RETEST_HORIZON_PLAN_OUTPUT_FILE")
            .map(PathBuf::from),
        retest_horizon_latest_l1_as_of_ms,
        retest_horizon_status_output_file: env_string("RESEARCH_RETEST_HORIZON_STATUS_OUTPUT_FILE")
            .map(PathBuf::from),
        retest_driver_summary_file: env_string("RESEARCH_RETEST_DRIVER_SUMMARY_FILE")
            .map(PathBuf::from),
        retest_horizon_status_file: env_string("RESEARCH_HORIZON_STATUS_FILE").map(PathBuf::from),
        retest_horizon_status_s3_bucket: env_string("RESEARCH_HORIZON_STATUS_S3_BUCKET"),
        retest_horizon_status_s3_key: env_string("RESEARCH_HORIZON_STATUS_S3_KEY"),
        focused_retest_manifest_output_file: env_string("RESEARCH_FOCUS_MANIFEST_OUTPUT")
            .map(PathBuf::from),
        focused_retest_summary_output_file: env_string("RESEARCH_FOCUS_SUMMARY_OUTPUT")
            .map(PathBuf::from),
        focused_retest_next_actions,
        focused_retest_historical_replay_index_ref_mode,
        input_manifest_file: None,
        input_manifest_s3_bucket: env_string("RESEARCH_INPUT_MANIFEST_S3_BUCKET"),
        input_manifest_s3_key: env_string("RESEARCH_INPUT_MANIFEST_S3_KEY"),
        research_report_file: env_string("RESEARCH_REPORT_FILE").map(PathBuf::from),
        research_report_s3_bucket: env_string("RESEARCH_REPORT_S3_BUCKET"),
        research_report_s3_key: env_string("RESEARCH_REPORT_S3_KEY"),
        input_bundle_file: None,
        input_bundle_s3_bucket: env_string("RESEARCH_INPUT_S3_BUCKET"),
        input_bundle_s3_key: env_string("RESEARCH_INPUT_S3_KEY"),
        market_feature_delta_file: None,
        market_regime_context_file: None,
        market_l1_s3_bucket: env_string("RESEARCH_MARKET_L1_S3_BUCKET"),
        market_feature_delta_s3_keys: env_list("RESEARCH_MARKET_FEATURE_DELTA_S3_KEYS"),
        market_regime_context_s3_keys: env_list("RESEARCH_MARKET_REGIME_CONTEXT_S3_KEYS"),
        historical_replay_run_files: Vec::new(),
        historical_replay_run_index_files: Vec::new(),
        oss_adapter_run_files: Vec::new(),
        shadow_validation_run_files: Vec::new(),
        oss_adapter_run_s3_bucket: env_string("RESEARCH_OSS_ADAPTER_RUN_S3_BUCKET"),
        oss_adapter_run_s3_keys: env_list("RESEARCH_OSS_ADAPTER_RUN_S3_KEYS"),
        shadow_validation_run_s3_bucket: env_string("RESEARCH_SHADOW_VALIDATION_RUN_S3_BUCKET"),
        shadow_validation_run_s3_keys: env_list("RESEARCH_SHADOW_VALIDATION_RUN_S3_KEYS"),
        paper_watch_candidate_file: env_string("RESEARCH_PAPER_WATCH_CANDIDATE_FILE")
            .map(PathBuf::from),
        paper_watch_candidate_s3_bucket: env_string("RESEARCH_PAPER_WATCH_CANDIDATE_S3_BUCKET"),
        paper_watch_candidate_s3_key: env_string("RESEARCH_PAPER_WATCH_CANDIDATE_S3_KEY"),
        market_live_tick_file: env_string("RESEARCH_MARKET_LIVE_TICK_FILE").map(PathBuf::from),
        market_live_nats_url: env_string("RESEARCH_MARKET_LIVE_NATS_URL"),
        market_live_nats_stream: env_string("RESEARCH_MARKET_LIVE_NATS_STREAM")
            .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_STREAM.to_owned()),
        market_live_nats_subject: env_string("RESEARCH_MARKET_LIVE_NATS_SUBJECT")
            .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_SUBJECT.to_owned()),
        market_live_nats_consumer: env_string("RESEARCH_MARKET_LIVE_NATS_CONSUMER")
            .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_CONSUMER.to_owned()),
        market_live_nats_deliver_policy: env_string("RESEARCH_MARKET_LIVE_NATS_DELIVER_POLICY")
            .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY.to_owned()),
        market_live_nats_batch_size: env_usize(
            "RESEARCH_MARKET_LIVE_NATS_BATCH_SIZE",
            DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE,
        )?,
        market_live_nats_max_messages: env_usize(
            "RESEARCH_MARKET_LIVE_NATS_MAX_MESSAGES",
            DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES,
        )?,
        market_live_nats_ack_wait_secs: env_u64(
            "RESEARCH_MARKET_LIVE_NATS_ACK_WAIT_SECS",
            DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS,
        )?,
        historical_replay_run_s3_bucket: env_string("RESEARCH_HISTORICAL_REPLAY_RUN_S3_BUCKET"),
        historical_replay_run_s3_keys: env_list("RESEARCH_HISTORICAL_REPLAY_RUN_S3_KEYS"),
        historical_replay_run_index_s3_bucket: env_string(
            "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET",
        ),
        historical_replay_run_index_s3_keys: env_list(
            "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_KEYS",
        ),
        output_dir: None,
        output_s3_bucket: env_string("RESEARCH_OUTPUT_S3_BUCKET"),
        output_s3_prefix: env_string("RESEARCH_OUTPUT_S3_PREFIX"),
        research_packet_id: "local_research_packet".to_owned(),
        run_scope: "p0_candidate_bundle_local".to_owned(),
        now_ms: None,
    };

    while let Some(arg) = values.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--build-shadow-cycle-decision" => {
                args.build_shadow_cycle_decision = true;
            }
            "--run-shadow-cycle-from-latest-state" => {
                args.run_shadow_cycle_from_latest_state = true;
            }
            "--build-retest-horizon-plan" => {
                args.build_retest_horizon_plan = true;
            }
            "--run-retest-refresh-cycle" => {
                args.run_retest_refresh_cycle = true;
            }
            "--run-retest-refresh-cycle-from-latest-state" => {
                args.run_retest_refresh_cycle_from_latest_state = true;
            }
            "--run-retest-cycle-scheduler" => {
                args.run_retest_cycle_scheduler = true;
            }
            "--build-retest-horizon-status" => {
                args.build_retest_horizon_status = true;
            }
            "--build-focused-retest-manifest" => {
                args.build_focused_retest_manifest = true;
            }
            "--run-paper-watch-live-cycle" => {
                args.run_paper_watch_live_cycle = true;
            }
            "--shadow-cycle-decision-file" => {
                args.shadow_cycle_decision_file = Some(absolute_path_arg(
                    values.next(),
                    "--shadow-cycle-decision-file requires an absolute path",
                )?);
            }
            "--shadow-cycle-decision-output-file" => {
                args.shadow_cycle_decision_output_file = Some(absolute_path_arg(
                    values.next(),
                    "--shadow-cycle-decision-output-file requires an absolute path",
                )?);
            }
            "--shadow-cycle-latest-l1-as-of-ms" => {
                let raw = values.next().ok_or_else(|| {
                    AppError::config("--shadow-cycle-latest-l1-as-of-ms requires a number")
                })?;
                args.shadow_cycle_latest_l1_as_of_ms = Some(parse_non_negative_i64(
                    "--shadow-cycle-latest-l1-as-of-ms",
                    &raw,
                )?);
            }
            "--retest-horizon-plan-file" => {
                args.retest_horizon_plan_file = Some(absolute_path_arg(
                    values.next(),
                    "--retest-horizon-plan-file requires an absolute path",
                )?);
            }
            "--retest-horizon-plan-s3-bucket" => {
                args.retest_horizon_plan_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--retest-horizon-plan-s3-bucket requires a value",
                )?);
            }
            "--retest-horizon-plan-s3-key" => {
                args.retest_horizon_plan_s3_key = Some(non_empty_arg(
                    values.next(),
                    "--retest-horizon-plan-s3-key requires a value",
                )?);
            }
            "--retest-horizon-plan-output-file" => {
                args.retest_horizon_plan_output_file = Some(absolute_path_arg(
                    values.next(),
                    "--retest-horizon-plan-output-file requires an absolute path",
                )?);
            }
            "--retest-horizon-latest-l1-as-of-ms" => {
                let raw = values.next().ok_or_else(|| {
                    AppError::config("--retest-horizon-latest-l1-as-of-ms requires a number")
                })?;
                args.retest_horizon_latest_l1_as_of_ms = Some(parse_non_negative_i64(
                    "--retest-horizon-latest-l1-as-of-ms",
                    &raw,
                )?);
            }
            "--retest-horizon-status-output-file" => {
                args.retest_horizon_status_output_file = Some(absolute_path_arg(
                    values.next(),
                    "--retest-horizon-status-output-file requires an absolute path",
                )?);
            }
            "--retest-driver-summary-file" => {
                args.retest_driver_summary_file = Some(absolute_path_arg(
                    values.next(),
                    "--retest-driver-summary-file requires an absolute path",
                )?);
            }
            "--retest-horizon-status-file" => {
                args.retest_horizon_status_file = Some(absolute_path_arg(
                    values.next(),
                    "--retest-horizon-status-file requires an absolute path",
                )?);
            }
            "--retest-horizon-status-s3-bucket" => {
                args.retest_horizon_status_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--retest-horizon-status-s3-bucket requires a value",
                )?);
            }
            "--retest-horizon-status-s3-key" => {
                args.retest_horizon_status_s3_key = Some(non_empty_arg(
                    values.next(),
                    "--retest-horizon-status-s3-key requires a value",
                )?);
            }
            "--focused-retest-manifest-output-file" => {
                args.focused_retest_manifest_output_file = Some(absolute_path_arg(
                    values.next(),
                    "--focused-retest-manifest-output-file requires an absolute path",
                )?);
            }
            "--focused-retest-summary-output-file" => {
                args.focused_retest_summary_output_file = Some(absolute_path_arg(
                    values.next(),
                    "--focused-retest-summary-output-file requires an absolute path",
                )?);
            }
            "--focused-retest-next-actions" => {
                let raw = non_empty_arg(
                    values.next(),
                    "--focused-retest-next-actions requires a comma-separated value",
                )?;
                let actions = parse_focused_retest_actions(&raw);
                if actions.is_empty() {
                    return Err(AppError::config(
                        "--focused-retest-next-actions must contain at least one action",
                    ));
                }
                args.focused_retest_next_actions = actions;
            }
            "--focused-retest-include-historical-index-refs" => {
                let raw = non_empty_arg(
                    values.next(),
                    "--focused-retest-include-historical-index-refs requires auto, true, or false",
                )?;
                args.focused_retest_historical_replay_index_ref_mode =
                    HistoricalReplayIndexRefMode::parse(&raw)?;
            }
            "--input-manifest-file" => {
                args.input_manifest_file = Some(absolute_path_arg(
                    values.next(),
                    "--input-manifest-file requires an absolute path",
                )?);
            }
            "--input-manifest-s3-bucket" => {
                args.input_manifest_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--input-manifest-s3-bucket requires a value",
                )?);
            }
            "--input-manifest-s3-key" => {
                args.input_manifest_s3_key = Some(non_empty_arg(
                    values.next(),
                    "--input-manifest-s3-key requires a value",
                )?);
            }
            "--research-report-file" => {
                args.research_report_file = Some(absolute_path_arg(
                    values.next(),
                    "--research-report-file requires an absolute path",
                )?);
            }
            "--research-report-s3-bucket" => {
                args.research_report_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--research-report-s3-bucket requires a value",
                )?);
            }
            "--research-report-s3-key" => {
                args.research_report_s3_key = Some(non_empty_arg(
                    values.next(),
                    "--research-report-s3-key requires a value",
                )?);
            }
            "--input-bundle-file" => {
                args.input_bundle_file = Some(absolute_path_arg(
                    values.next(),
                    "--input-bundle-file requires an absolute path",
                )?);
            }
            "--input-bundle-s3-bucket" => {
                args.input_bundle_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--input-bundle-s3-bucket requires a value",
                )?);
            }
            "--input-bundle-s3-key" => {
                args.input_bundle_s3_key = Some(non_empty_arg(
                    values.next(),
                    "--input-bundle-s3-key requires a value",
                )?);
            }
            "--market-feature-delta-file" => {
                args.market_feature_delta_file = Some(absolute_path_arg(
                    values.next(),
                    "--market-feature-delta-file requires an absolute path",
                )?);
            }
            "--market-regime-context-file" => {
                args.market_regime_context_file = Some(absolute_path_arg(
                    values.next(),
                    "--market-regime-context-file requires an absolute path",
                )?);
            }
            "--market-l1-s3-bucket" => {
                args.market_l1_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--market-l1-s3-bucket requires a value",
                )?);
            }
            "--market-feature-delta-s3-key" => {
                args.market_feature_delta_s3_keys.push(non_empty_arg(
                    values.next(),
                    "--market-feature-delta-s3-key requires a value",
                )?);
            }
            "--market-regime-context-s3-key" => {
                args.market_regime_context_s3_keys.push(non_empty_arg(
                    values.next(),
                    "--market-regime-context-s3-key requires a value",
                )?);
            }
            "--historical-replay-run-file" => {
                args.historical_replay_run_files.push(absolute_path_arg(
                    values.next(),
                    "--historical-replay-run-file requires an absolute path",
                )?);
            }
            "--historical-replay-run-index-file" => {
                args.historical_replay_run_index_files
                    .push(absolute_path_arg(
                        values.next(),
                        "--historical-replay-run-index-file requires an absolute path",
                    )?);
            }
            "--oss-adapter-run-file" => {
                args.oss_adapter_run_files.push(absolute_path_arg(
                    values.next(),
                    "--oss-adapter-run-file requires an absolute path",
                )?);
            }
            "--shadow-validation-run-file" => {
                args.shadow_validation_run_files.push(absolute_path_arg(
                    values.next(),
                    "--shadow-validation-run-file requires an absolute path",
                )?);
            }
            "--oss-adapter-run-s3-bucket" => {
                args.oss_adapter_run_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--oss-adapter-run-s3-bucket requires a value",
                )?);
            }
            "--oss-adapter-run-s3-key" => {
                args.oss_adapter_run_s3_keys.push(non_empty_arg(
                    values.next(),
                    "--oss-adapter-run-s3-key requires a value",
                )?);
            }
            "--shadow-validation-run-s3-bucket" => {
                args.shadow_validation_run_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--shadow-validation-run-s3-bucket requires a value",
                )?);
            }
            "--shadow-validation-run-s3-key" => {
                args.shadow_validation_run_s3_keys.push(non_empty_arg(
                    values.next(),
                    "--shadow-validation-run-s3-key requires a value",
                )?);
            }
            "--paper-watch-candidate-file" => {
                args.paper_watch_candidate_file = Some(absolute_path_arg(
                    values.next(),
                    "--paper-watch-candidate-file requires an absolute path",
                )?);
            }
            "--paper-watch-candidate-s3-bucket" => {
                args.paper_watch_candidate_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--paper-watch-candidate-s3-bucket requires a value",
                )?);
            }
            "--paper-watch-candidate-s3-key" => {
                args.paper_watch_candidate_s3_key = Some(non_empty_arg(
                    values.next(),
                    "--paper-watch-candidate-s3-key requires a value",
                )?);
            }
            "--market-live-tick-file" => {
                args.market_live_tick_file = Some(absolute_path_arg(
                    values.next(),
                    "--market-live-tick-file requires an absolute path",
                )?);
            }
            "--market-live-nats-url" => {
                args.market_live_nats_url = Some(non_empty_arg(
                    values.next(),
                    "--market-live-nats-url requires a value",
                )?);
            }
            "--market-live-nats-stream" => {
                args.market_live_nats_stream =
                    non_empty_arg(values.next(), "--market-live-nats-stream requires a value")?;
            }
            "--market-live-nats-subject" => {
                args.market_live_nats_subject =
                    non_empty_arg(values.next(), "--market-live-nats-subject requires a value")?;
            }
            "--market-live-nats-consumer" => {
                args.market_live_nats_consumer = non_empty_arg(
                    values.next(),
                    "--market-live-nats-consumer requires a value",
                )?;
            }
            "--market-live-nats-deliver-policy" => {
                args.market_live_nats_deliver_policy = non_empty_arg(
                    values.next(),
                    "--market-live-nats-deliver-policy requires a value",
                )?;
            }
            "--market-live-nats-batch-size" => {
                let raw = non_empty_arg(
                    values.next(),
                    "--market-live-nats-batch-size requires a positive integer",
                )?;
                args.market_live_nats_batch_size =
                    parse_positive_usize("--market-live-nats-batch-size", &raw)?;
            }
            "--market-live-nats-max-messages" => {
                let raw = non_empty_arg(
                    values.next(),
                    "--market-live-nats-max-messages requires a positive integer",
                )?;
                args.market_live_nats_max_messages =
                    parse_positive_usize("--market-live-nats-max-messages", &raw)?;
            }
            "--market-live-nats-ack-wait-secs" => {
                let raw = non_empty_arg(
                    values.next(),
                    "--market-live-nats-ack-wait-secs requires a positive integer",
                )?;
                args.market_live_nats_ack_wait_secs =
                    parse_positive_u64("--market-live-nats-ack-wait-secs", &raw)?;
            }
            "--historical-replay-run-s3-bucket" => {
                args.historical_replay_run_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--historical-replay-run-s3-bucket requires a value",
                )?);
            }
            "--historical-replay-run-s3-key" => {
                args.historical_replay_run_s3_keys.push(non_empty_arg(
                    values.next(),
                    "--historical-replay-run-s3-key requires a value",
                )?);
            }
            "--historical-replay-run-index-s3-bucket" => {
                args.historical_replay_run_index_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--historical-replay-run-index-s3-bucket requires a value",
                )?);
            }
            "--historical-replay-run-index-s3-key" => {
                args.historical_replay_run_index_s3_keys.push(non_empty_arg(
                    values.next(),
                    "--historical-replay-run-index-s3-key requires a value",
                )?);
            }
            "--output-dir" => {
                args.output_dir = Some(absolute_path_arg(
                    values.next(),
                    "--output-dir requires an absolute path",
                )?);
            }
            "--output-s3-bucket" => {
                args.output_s3_bucket = Some(non_empty_arg(
                    values.next(),
                    "--output-s3-bucket requires a value",
                )?);
            }
            "--output-s3-prefix" => {
                args.output_s3_prefix = Some(non_empty_arg(
                    values.next(),
                    "--output-s3-prefix requires a value",
                )?);
            }
            "--research-packet-id" => {
                args.research_packet_id =
                    non_empty_arg(values.next(), "--research-packet-id requires a value")?;
            }
            "--run-scope" => {
                args.run_scope = non_empty_arg(values.next(), "--run-scope requires a value")?;
            }
            "--now-ms" => {
                let raw = values
                    .next()
                    .ok_or_else(|| AppError::config("--now-ms requires a number"))?;
                let value = raw
                    .parse::<i64>()
                    .map_err(|_| AppError::config("--now-ms must be an integer"))?;
                if value < 0 {
                    return Err(AppError::config("--now-ms must be non-negative"));
                }
                args.now_ms = Some(value);
            }
            other => {
                return Err(AppError::config(format!(
                    "unknown argument: {other}\n\n{}",
                    help_text()
                )));
            }
        }
    }

    if args.shadow_cycle_decision_file.is_some()
        && (args.build_shadow_cycle_decision || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use shadow cycle decision validation separately from shadow cycle build modes",
        ));
    }
    if args.run_retest_cycle_scheduler && args.build_focused_retest_manifest {
        return Err(AppError::config(
            "use either --run-retest-cycle-scheduler or --build-focused-retest-manifest, not both",
        ));
    }
    if args.build_retest_horizon_plan
        && (args.build_retest_horizon_status
            || args.run_retest_refresh_cycle
            || args.run_retest_refresh_cycle_from_latest_state
            || args.run_retest_cycle_scheduler
            || args.build_focused_retest_manifest
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use --build-retest-horizon-plan separately from retest status, scheduler, or focused manifest modes",
        ));
    }
    if args.run_retest_refresh_cycle
        && (args.build_retest_horizon_status
            || args.run_retest_refresh_cycle_from_latest_state
            || args.run_retest_cycle_scheduler
            || args.build_focused_retest_manifest
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use --run-retest-refresh-cycle separately from retest status, scheduler, or focused manifest modes",
        ));
    }
    if args.run_retest_refresh_cycle_from_latest_state
        && (args.run_retest_cycle_scheduler
            || args.build_focused_retest_manifest
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use --run-retest-refresh-cycle-from-latest-state separately from retest scheduler or focused manifest modes",
        ));
    }
    if args.build_retest_horizon_status
        && (args.run_retest_refresh_cycle_from_latest_state
            || args.run_retest_cycle_scheduler
            || args.build_focused_retest_manifest
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use --build-retest-horizon-status separately from retest scheduler or focused manifest modes",
        ));
    }
    if has_retest_horizon_status_input(&args)
        && (args.shadow_cycle_decision_file.is_some()
            || args.build_shadow_cycle_decision
            || args.run_shadow_cycle_from_latest_state)
    {
        return Err(AppError::config(
            "use retest horizon status inputs separately from shadow cycle decision modes",
        ));
    }
    if args.run_paper_watch_live_cycle {
        validate_paper_watch_live_cycle_args(&args)?;
        return Ok(Some(args));
    }
    validate_retest_horizon_plan_input_args(&args)?;
    validate_retest_horizon_status_input_args(&args)?;
    validate_research_report_input_args(&args)?;
    if args.build_retest_horizon_plan {
        validate_retest_horizon_plan_build_args(&args)?;
        return Ok(Some(args));
    }
    if args.run_retest_refresh_cycle {
        validate_retest_refresh_cycle_args(&args)?;
        return Ok(Some(args));
    }
    if args.run_retest_refresh_cycle_from_latest_state {
        validate_retest_refresh_cycle_from_latest_state_args(&args)?;
        return Ok(Some(args));
    }
    if args.build_retest_horizon_status {
        validate_retest_horizon_status_build_args(&args)?;
        return Ok(Some(args));
    }
    if args.run_retest_cycle_scheduler {
        validate_retest_cycle_scheduler_args(&args)?;
        return Ok(Some(args));
    }
    if args.build_focused_retest_manifest {
        validate_focused_retest_manifest_build_args(&args)?;
        return Ok(Some(args));
    }
    if args.run_shadow_cycle_from_latest_state {
        validate_shadow_cycle_from_latest_state_args(&args)?;
        return Ok(Some(args));
    }
    if has_retest_horizon_status_input(&args) {
        return Ok(Some(args));
    }
    if args.shadow_cycle_decision_file.is_some() {
        return Ok(Some(args));
    }
    if args.build_shadow_cycle_decision {
        validate_shadow_cycle_build_args(&args)?;
        return Ok(Some(args));
    }

    if args.input_bundle_file.is_some()
        && (args.input_bundle_s3_bucket.is_some() || args.input_bundle_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --input-bundle-file or --input-bundle-s3-bucket/--input-bundle-s3-key, not both",
        ));
    }
    if args.input_manifest_file.is_some()
        && (args.input_manifest_s3_bucket.is_some() || args.input_manifest_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        ));
    }
    if args.input_manifest_s3_bucket.is_some() != args.input_manifest_s3_key.is_some() {
        return Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        ));
    }
    if args.input_bundle_file.is_none()
        && args.input_manifest_file.is_none()
        && args.input_manifest_s3_key.is_none()
        && (args.input_bundle_s3_bucket.is_none() || args.input_bundle_s3_key.is_none())
    {
        return Err(AppError::config(
            "--input-bundle-file or --input-manifest-file is required unless S3 input environment is set",
        ));
    }
    if args.output_dir.is_some() && args.output_s3_bucket.is_some() {
        return Err(AppError::config(
            "use either --output-dir or --output-s3-bucket, not both",
        ));
    }
    if args.market_feature_delta_file.is_some() && !args.market_feature_delta_s3_keys.is_empty() {
        return Err(AppError::config(
            "use either --market-feature-delta-file or --market-feature-delta-s3-key, not both",
        ));
    }
    if args.market_regime_context_file.is_some() && !args.market_regime_context_s3_keys.is_empty() {
        return Err(AppError::config(
            "use either --market-regime-context-file or --market-regime-context-s3-key, not both",
        ));
    }
    if !args.historical_replay_run_s3_keys.is_empty()
        && args.historical_replay_run_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--historical-replay-run-s3-bucket is required when --historical-replay-run-s3-key is set",
        ));
    }
    if !args.historical_replay_run_index_s3_keys.is_empty()
        && args.historical_replay_run_index_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--historical-replay-run-index-s3-bucket is required when --historical-replay-run-index-s3-key is set",
        ));
    }
    if !args.oss_adapter_run_s3_keys.is_empty() && args.oss_adapter_run_s3_bucket.is_none() {
        return Err(AppError::config(
            "--oss-adapter-run-s3-bucket is required when --oss-adapter-run-s3-key is set",
        ));
    }
    if !args.shadow_validation_run_s3_keys.is_empty()
        && args.shadow_validation_run_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--shadow-validation-run-s3-bucket is required when --shadow-validation-run-s3-key is set",
        ));
    }

    Ok(Some(args))
}

pub async fn run(args: Args) -> AppResult<RunSummary> {
    if args.run_paper_watch_live_cycle {
        return run_paper_watch_live_cycle_mode(&args).await;
    }
    if args.run_retest_refresh_cycle {
        return run_retest_refresh_cycle_mode(&args).await;
    }
    if args.run_retest_refresh_cycle_from_latest_state {
        return run_retest_refresh_cycle_from_latest_state_mode(&args).await;
    }
    if args.run_shadow_cycle_from_latest_state {
        return run_shadow_cycle_from_latest_state_mode(&args).await;
    }
    if args.build_retest_horizon_plan {
        return build_retest_horizon_plan_mode(&args).await;
    }
    if args.build_retest_horizon_status {
        return build_retest_horizon_status_mode(&args).await;
    }
    if args.run_retest_cycle_scheduler {
        return run_retest_cycle_scheduler_mode(&args).await;
    }
    if args.build_focused_retest_manifest {
        return build_focused_retest_manifest_mode(&args).await;
    }
    if has_retest_horizon_status_input(&args) {
        let status = load_retest_horizon_status(&args).await?;
        let validation = validate_retest_horizon_status(&status)?;
        return Ok(RunSummary {
            retest_horizon_plans_created: 0,
            retest_horizon_statuses_validated: 1,
            retest_cycle_scheduler_action: Some(validation.scheduler_action),
            retest_cycle_run_not_before_ms: validation.run_not_before_ms,
            focused_retest_manifests_created: 0,
            focused_retest_horizon_count: 0,
            focused_retest_candidate_bundle_refs: 0,
            shadow_cycle_decisions_validated: 0,
            shadow_cycle_decisions_created: 0,
            shadow_cycle_scheduler_action: None,
            shadow_cycle_run_not_before_ms: None,
            shadow_cycle_focused_research_manifest_file: None,
            processed_bundles: 0,
            replay_runs_created: 0,
            historical_replay_runs_loaded: 0,
            oss_adapter_runs_loaded: 0,
            shadow_validation_runs_loaded: 0,
            shadow_validation_runs_created: 0,
            paper_trade_candidates_created: 0,
            paper_trade_runs_created: 0,
            paper_trade_summaries_created: 0,
            paper_trade_marks_created: 0,
            paper_watch_live_marks_created: 0,
            portfolio_risk_reject_events_created: 0,
            portfolio_reduce_only_signals_created: 0,
            output_files: Vec::new(),
        });
    }
    if let Some(path) = args.shadow_cycle_decision_file.as_deref() {
        let decision = read_shadow_cycle_decision(path)?;
        validate_shadow_cycle_decision(&decision)?;
        return Ok(RunSummary {
            retest_horizon_plans_created: 0,
            retest_horizon_statuses_validated: 0,
            retest_cycle_scheduler_action: None,
            retest_cycle_run_not_before_ms: None,
            focused_retest_manifests_created: 0,
            focused_retest_horizon_count: 0,
            focused_retest_candidate_bundle_refs: 0,
            shadow_cycle_decisions_validated: 1,
            shadow_cycle_decisions_created: 0,
            shadow_cycle_scheduler_action: Some(decision.scheduler_action),
            shadow_cycle_run_not_before_ms: decision.run_not_before_ms,
            shadow_cycle_focused_research_manifest_file: decision.focused_research_manifest_file,
            processed_bundles: 0,
            replay_runs_created: 0,
            historical_replay_runs_loaded: 0,
            oss_adapter_runs_loaded: 0,
            shadow_validation_runs_loaded: 0,
            shadow_validation_runs_created: 0,
            paper_trade_candidates_created: 0,
            paper_trade_runs_created: 0,
            paper_trade_summaries_created: 0,
            paper_trade_marks_created: 0,
            paper_watch_live_marks_created: 0,
            portfolio_risk_reject_events_created: 0,
            portfolio_reduce_only_signals_created: 0,
            output_files: Vec::new(),
        });
    }
    if args.build_shadow_cycle_decision {
        return build_shadow_cycle_decision_mode(&args).await;
    }

    let manifest = load_input_manifest(&args).await?;
    validate_input_manifest(manifest.as_ref())?;
    let budget = manifest
        .as_ref()
        .map(|manifest| manifest.runtime_budget_policy.clone())
        .unwrap_or_default();
    validate_manifest_budget(manifest.as_ref(), &budget)?;

    let bundles = read_input_bundles(&args, manifest.as_ref()).await?;
    if bundles.is_empty() {
        return Err(AppError::validation("input bundle file must not be empty"));
    }
    enforce_budget(
        "candidate_bundle_count",
        bundles.len(),
        budget.max_candidate_bundle_count,
    )?;
    let market_deltas = load_market_deltas(
        &args,
        &bundles,
        manifest.as_ref(),
        budget.max_market_artifact_ref_count,
    )
    .await?;
    let regime_contexts = load_regime_contexts(
        &args,
        &bundles,
        manifest.as_ref(),
        budget.max_market_artifact_ref_count,
    )
    .await?;
    let oss_adapter_runs = load_oss_adapter_runs(&args, manifest.as_ref()).await?;
    let completed_shadow_validation_runs =
        load_shadow_validation_runs(&args, manifest.as_ref()).await?;
    validate_oss_adapter_runs(&oss_adapter_runs)?;
    let created_at_ms = args
        .now_ms
        .unwrap_or_else(|| deterministic_report_created_at_ms(&bundles));
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let replay_runs = build_replay_runs(&bundles, &market_deltas, &regime_contexts);
    enforce_budget(
        "new_replay_run_count",
        replay_runs.len(),
        budget.max_replay_run_count,
    )?;
    let historical_replay_runs = filter_historical_replay_runs_for_current_research(
        load_historical_replay_runs(
            &args,
            manifest.as_ref(),
            budget.max_historical_replay_run_ref_count,
        )
        .await?,
        &replay_runs,
    );
    enforce_budget(
        "historical_replay_run_count",
        historical_replay_runs.len(),
        budget.max_replay_run_count,
    )?;
    enforce_budget(
        "oss_adapter_run_count",
        oss_adapter_runs.len(),
        budget.max_oss_adapter_run_ref_count,
    )?;
    enforce_budget(
        "shadow_validation_run_count",
        completed_shadow_validation_runs.len(),
        budget.max_shadow_validation_run_ref_count,
    )?;
    let mut aggregate_replay_runs = historical_replay_runs.clone();
    aggregate_replay_runs.extend(replay_runs.clone());
    enforce_budget(
        "aggregate_replay_run_count",
        aggregate_replay_runs.len(),
        budget.max_replay_run_count,
    )?;
    let research_packet_id = manifest
        .as_ref()
        .and_then(|manifest| manifest.research_packet_id.as_deref())
        .unwrap_or(&args.research_packet_id);
    let run_scope = manifest
        .as_ref()
        .and_then(|manifest| manifest.run_scope.as_deref())
        .unwrap_or(&args.run_scope);
    let mut report = build_report(
        research_packet_id,
        run_scope,
        created_at_ms,
        &bundles,
        &aggregate_replay_runs,
        &oss_adapter_runs,
        &completed_shadow_validation_runs,
    );
    let paper_watch_candidates = build_paper_watch_candidates(&report, &bundles, created_at_ms);
    report.paper_watch_candidates = paper_watch_candidates
        .iter()
        .map(|candidate| candidate.paper_watch_candidate_id.clone())
        .collect();
    let paper_artifacts = build_paper_artifacts(
        &report,
        &bundles,
        &completed_shadow_validation_runs,
        created_at_ms,
    );
    report.paper_trade_candidates = paper_artifacts
        .candidates
        .iter()
        .map(|candidate| candidate.paper_trade_candidate_id.clone())
        .collect();
    let output_artifacts = ResearchOutputArtifacts {
        report: &report,
        replay_runs: &replay_runs,
        shadow_validation_runs: &report.shadow_validation_runs,
        paper_watch_candidates: &paper_watch_candidates,
        paper_trade_candidates: &paper_artifacts.candidates,
        paper_trade_runs: &paper_artifacts.runs,
        paper_trade_summaries: &paper_artifacts.summaries,
        paper_trade_marks: &paper_artifacts.marks,
        output_partition_at_ms,
    };
    let mut output_files = if let Some(output_dir) = args.output_dir.as_deref() {
        write_research_outputs(output_dir, &output_artifacts)?
            .into_iter()
            .map(|path| path.display().to_string())
            .collect()
    } else if let Some(output_bucket) = args.output_s3_bucket.as_deref() {
        write_research_outputs_to_s3(
            output_bucket,
            args.output_s3_prefix.as_deref().unwrap_or(""),
            &output_artifacts,
        )
        .await?
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
        Vec::new()
    };
    output_files.extend(
        write_retest_cycle_source_state_output(
            &args,
            &report,
            &output_files,
            output_partition_at_ms,
        )
        .await?,
    );
    emit_research_report_alert_from_env(&report).await;

    Ok(RunSummary {
        retest_horizon_plans_created: 0,
        retest_horizon_statuses_validated: 0,
        retest_cycle_scheduler_action: None,
        retest_cycle_run_not_before_ms: None,
        focused_retest_manifests_created: 0,
        focused_retest_horizon_count: 0,
        focused_retest_candidate_bundle_refs: 0,
        shadow_cycle_decisions_validated: 0,
        shadow_cycle_decisions_created: 0,
        shadow_cycle_scheduler_action: None,
        shadow_cycle_run_not_before_ms: None,
        shadow_cycle_focused_research_manifest_file: None,
        processed_bundles: bundles.len(),
        replay_runs_created: replay_runs.len(),
        historical_replay_runs_loaded: historical_replay_runs.len(),
        oss_adapter_runs_loaded: oss_adapter_runs.len(),
        shadow_validation_runs_loaded: completed_shadow_validation_runs.len(),
        shadow_validation_runs_created: report.shadow_validation_runs.len(),
        paper_trade_candidates_created: paper_artifacts.candidates.len(),
        paper_trade_runs_created: paper_artifacts.runs.len(),
        paper_trade_summaries_created: paper_artifacts.summaries.len(),
        paper_trade_marks_created: paper_artifacts.marks.len(),
        paper_watch_live_marks_created: 0,
        portfolio_risk_reject_events_created: report.portfolio_risk_reject_events.len(),
        portfolio_reduce_only_signals_created: report.portfolio_reduce_only_signals.len(),
        output_files,
    })
}

async fn run_paper_watch_live_cycle_mode(args: &Args) -> AppResult<RunSummary> {
    let candidates = load_paper_watch_candidates(args).await?;
    if candidates.is_empty() {
        return Err(AppError::validation(
            "paper watch candidate input must not be empty",
        ));
    }
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let ticks = load_market_live_ticks(args, &candidates, output_partition_at_ms).await?;
    let marks = build_paper_watch_live_marks(&candidates, &ticks);
    let output_files = if let Some(output_dir) = args.output_dir.as_deref() {
        write_paper_watch_live_marks(output_dir, &marks, output_partition_at_ms)?
            .into_iter()
            .map(|path| path.display().to_string())
            .collect()
    } else if let Some(output_bucket) = args.output_s3_bucket.as_deref() {
        write_paper_watch_live_marks_to_s3(
            output_bucket,
            args.output_s3_prefix.as_deref().unwrap_or(""),
            &marks,
            output_partition_at_ms,
        )
        .await?
    } else {
        println!("{}", serde_json::to_string_pretty(&marks)?);
        Vec::new()
    };

    Ok(RunSummary {
        paper_watch_live_marks_created: marks.len(),
        output_files,
        ..RunSummary::default()
    })
}

async fn write_retest_cycle_source_state_output(
    args: &Args,
    report: &crate::model::ResearchRunReport,
    output_files: &[String],
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    let Some(output_bucket) = args.output_s3_bucket.as_deref() else {
        return Ok(Vec::new());
    };
    let (Some(source_manifest_s3_bucket), Some(source_manifest_s3_key)) = (
        args.input_manifest_s3_bucket.as_deref(),
        args.input_manifest_s3_key.as_deref(),
    ) else {
        return Ok(Vec::new());
    };
    let source_research_report_s3_key = research_report_s3_key_from_output_files(
        output_bucket,
        output_files,
        &report.research_run_report_id,
    )?;
    let state = build_retest_cycle_source_state(
        output_partition_at_ms,
        source_manifest_s3_bucket,
        source_manifest_s3_key,
        output_bucket,
        &source_research_report_s3_key,
        report,
    );
    write_retest_cycle_source_state_to_s3(output_bucket, "", &state, output_partition_at_ms)
        .await
        .map(|uri| vec![uri])
}

fn build_retest_cycle_source_state(
    generated_at_ms: i64,
    source_manifest_s3_bucket: &str,
    source_manifest_s3_key: &str,
    source_research_report_s3_bucket: &str,
    source_research_report_s3_key: &str,
    report: &crate::model::ResearchRunReport,
) -> RetestCycleSourceState {
    RetestCycleSourceState {
        schema_version: RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION.to_owned(),
        generated_at_ms,
        research_packet_id: report.research_packet_id.clone(),
        run_scope: report.run_scope.clone(),
        source_manifest_s3_bucket: source_manifest_s3_bucket.to_owned(),
        source_manifest_s3_key: source_manifest_s3_key.to_owned(),
        source_research_report_s3_bucket: source_research_report_s3_bucket.to_owned(),
        source_research_report_s3_key: source_research_report_s3_key.to_owned(),
        source_research_report_id: report.research_run_report_id.clone(),
        source_candidate_ids: report.source_candidate_ids.clone(),
        replay_run_id_count: report.replay_run_ids.len(),
        summary_findings_count: report.summary_findings.len(),
        shadow_validation_run_count: report.shadow_validation_runs.len(),
        paper_trade_candidate_count: report.paper_trade_candidates.len(),
        safety: RetestCycleSourceStateSafety {
            dispatcher_prefix: "research-input-manifest/".to_owned(),
            state_s3_write: true,
            ecs_task_started: false,
            shadow_paper_live_enabled: false,
        },
    }
}

fn research_report_s3_key_from_output_files(
    bucket: &str,
    output_files: &[String],
    research_run_report_id: &str,
) -> AppResult<String> {
    let uri_prefix = format!("s3://{bucket}/");
    let report_path = format!("research_run_report_id={research_run_report_id}/report.json");
    output_files
        .iter()
        .find_map(|file| {
            file.strip_prefix(&uri_prefix)
                .filter(|key| {
                    key.starts_with("research-run-report/")
                        || key.contains("/research-run-report/")
                })
                .filter(|key| key.ends_with(&report_path))
                .map(ToOwned::to_owned)
        })
        .ok_or_else(|| {
            AppError::validation(format!(
                "research output files missing S3 report for research_run_report_id={research_run_report_id}"
            ))
        })
}

async fn build_shadow_cycle_decision_mode(args: &Args) -> AppResult<RunSummary> {
    let manifest = load_input_manifest(args).await?;
    validate_input_manifest(manifest.as_ref())?;
    let shadow_validation_runs = load_shadow_validation_runs(args, manifest.as_ref()).await?;
    if shadow_validation_runs.is_empty() {
        return Err(AppError::validation(
            "shadow cycle decision build requires at least one shadow validation run",
        ));
    }

    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let decision = build_shadow_cycle_decision(
        &shadow_validation_runs,
        args.shadow_cycle_latest_l1_as_of_ms,
        output_partition_at_ms,
    );
    validate_shadow_cycle_decision(&decision)?;

    let output_files =
        write_shadow_cycle_decision_outputs(args, &decision, output_partition_at_ms).await?;
    emit_shadow_cycle_decision_alert_from_env(&decision).await;

    Ok(RunSummary {
        retest_horizon_plans_created: 0,
        retest_horizon_statuses_validated: 0,
        retest_cycle_scheduler_action: None,
        retest_cycle_run_not_before_ms: None,
        focused_retest_manifests_created: 0,
        focused_retest_horizon_count: 0,
        focused_retest_candidate_bundle_refs: 0,
        shadow_cycle_decisions_validated: 1,
        shadow_cycle_decisions_created: 1,
        shadow_cycle_scheduler_action: Some(decision.scheduler_action),
        shadow_cycle_run_not_before_ms: decision.run_not_before_ms,
        shadow_cycle_focused_research_manifest_file: decision.focused_research_manifest_file,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: shadow_validation_runs.len(),
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files,
    })
}

async fn build_retest_horizon_plan_mode(args: &Args) -> AppResult<RunSummary> {
    let manifest = load_input_manifest(args).await?.ok_or_else(|| {
        AppError::config(
            "--build-retest-horizon-plan requires --input-manifest-file or S3 manifest input",
        )
    })?;
    validate_input_manifest(Some(&manifest))?;
    let bundles = read_input_bundles(args, Some(&manifest)).await?;
    validate_manifest_budget(Some(&manifest), &manifest.runtime_budget_policy)?;
    enforce_budget(
        "candidate_bundle_count",
        bundles.len(),
        manifest.runtime_budget_policy.max_candidate_bundle_count,
    )?;
    let report = load_research_report(args).await?;
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let latest_l1_as_of_ms = retest_plan_latest_l1_as_of_ms(args).await?;
    let plan = build_retest_horizon_plan(
        &bundles,
        &report,
        &RetestHorizonPlanBuildOptions {
            generated_at_ms: output_partition_at_ms,
            manifest_label: input_manifest_label(args),
            report_label: research_report_label(args),
            latest_l1_as_of_ms,
        },
    )?;
    let output_files =
        write_retest_horizon_plan_outputs(args, &plan, output_partition_at_ms).await?;

    Ok(RunSummary {
        retest_horizon_plans_created: 1,
        retest_horizon_statuses_validated: 0,
        retest_cycle_scheduler_action: None,
        retest_cycle_run_not_before_ms: None,
        focused_retest_manifests_created: 0,
        focused_retest_horizon_count: 0,
        focused_retest_candidate_bundle_refs: 0,
        shadow_cycle_decisions_validated: 0,
        shadow_cycle_decisions_created: 0,
        shadow_cycle_scheduler_action: None,
        shadow_cycle_run_not_before_ms: None,
        shadow_cycle_focused_research_manifest_file: None,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: 0,
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files,
    })
}

async fn run_retest_refresh_cycle_mode(args: &Args) -> AppResult<RunSummary> {
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let manifest = load_input_manifest(args).await?.ok_or_else(|| {
        AppError::config(
            "--run-retest-refresh-cycle requires --input-manifest-file or S3 manifest input",
        )
    })?;
    validate_input_manifest(Some(&manifest))?;
    let bundles = read_input_bundles(args, Some(&manifest)).await?;
    validate_manifest_budget(Some(&manifest), &manifest.runtime_budget_policy)?;
    enforce_budget(
        "candidate_bundle_count",
        bundles.len(),
        manifest.runtime_budget_policy.max_candidate_bundle_count,
    )?;
    let report = load_research_report(args).await?;
    let latest_l1_as_of_ms = retest_plan_latest_l1_as_of_ms(args).await?;
    let plan = build_retest_horizon_plan(
        &bundles,
        &report,
        &RetestHorizonPlanBuildOptions {
            generated_at_ms: output_partition_at_ms,
            manifest_label: input_manifest_label(args),
            report_label: research_report_label(args),
            latest_l1_as_of_ms,
        },
    )?;
    let mut output_files =
        write_retest_refresh_cycle_plan_output(args, &plan, output_partition_at_ms).await?;

    let status = build_retest_horizon_status(
        &plan,
        None,
        &RetestHorizonStatusBuildOptions {
            generated_at_ms: output_partition_at_ms,
            plan_file: output_files.first().cloned(),
            driver_summary_file: None,
            checkpoint_s3_write: args.output_s3_bucket.is_some(),
        },
    )?;
    let validation = validate_retest_horizon_status(&status)?;
    output_files.extend(
        write_retest_refresh_cycle_status_output(args, &status, output_partition_at_ms).await?,
    );

    let mut retest_cycle_scheduler_action = validation.scheduler_action;
    let mut focused_retest_manifests_created = 0;
    let mut focused_retest_horizon_count = 0;
    let mut focused_retest_candidate_bundle_refs = 0;
    if retest_cycle_scheduler_action == "RUN_FOCUSED_RETEST_RESEARCH" {
        let mut build = build_focused_retest_manifest(
            &status,
            &manifest,
            &FocusedRetestBuildOptions {
                generated_at_ms: output_partition_at_ms,
                research_packet_id: focused_retest_packet_id(args, output_partition_at_ms),
                run_scope: focused_retest_run_scope(args),
                next_actions: args.focused_retest_next_actions.clone(),
                candidate_lifecycle_key_filter: Vec::new(),
                historical_replay_index_ref_mode: args
                    .focused_retest_historical_replay_index_ref_mode,
                s3_write: args.output_s3_bucket.is_some(),
            },
        )?;
        if args.output_s3_bucket.is_some() {
            let dispatch_packet_id =
                focused_retest_dispatch_packet_id(args, latest_l1_as_of_ms, &build)?;
            build.manifest.research_packet_id = Some(dispatch_packet_id);
        }
        focused_retest_horizon_count = build.summary.focused.focus_horizon_count;
        focused_retest_candidate_bundle_refs =
            build.summary.focused.selected_candidate_bundle_ref_count;
        let write_result = write_retest_refresh_cycle_focused_manifest_output(args, &build).await?;
        if write_result.created {
            focused_retest_manifests_created = 1;
        } else {
            retest_cycle_scheduler_action =
                "SKIP_FOCUSED_RETEST_RESEARCH_ALREADY_DISPATCHED".to_owned();
        }
        output_files.extend(write_result.output_files);
    }

    Ok(RunSummary {
        retest_horizon_plans_created: 1,
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: Some(retest_cycle_scheduler_action),
        retest_cycle_run_not_before_ms: validation.run_not_before_ms,
        focused_retest_manifests_created,
        focused_retest_horizon_count,
        focused_retest_candidate_bundle_refs,
        shadow_cycle_decisions_validated: 0,
        shadow_cycle_decisions_created: 0,
        shadow_cycle_scheduler_action: None,
        shadow_cycle_run_not_before_ms: None,
        shadow_cycle_focused_research_manifest_file: None,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: 0,
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files,
    })
}

async fn run_retest_refresh_cycle_from_latest_state_mode(args: &Args) -> AppResult<RunSummary> {
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --output-s3-bucket",
        ));
    };
    let state = read_latest_retest_cycle_source_state_from_s3(bucket, "").await?;
    let mut derived_args = args.clone();
    derived_args.run_retest_refresh_cycle_from_latest_state = false;
    derived_args.run_retest_refresh_cycle = true;
    derived_args.input_manifest_file = None;
    derived_args.input_manifest_s3_bucket = Some(state.source_manifest_s3_bucket);
    derived_args.input_manifest_s3_key = Some(state.source_manifest_s3_key);
    derived_args.research_report_file = None;
    derived_args.research_report_s3_bucket = Some(state.source_research_report_s3_bucket);
    derived_args.research_report_s3_key = Some(state.source_research_report_s3_key);
    run_retest_refresh_cycle_mode(&derived_args).await
}

async fn build_retest_horizon_status_mode(args: &Args) -> AppResult<RunSummary> {
    let plan = load_retest_horizon_plan(args).await?;
    let driver_summary = load_retest_driver_summary(args)?;
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let status = build_retest_horizon_status(
        &plan,
        driver_summary.as_ref(),
        &RetestHorizonStatusBuildOptions {
            generated_at_ms: output_partition_at_ms,
            plan_file: args
                .retest_horizon_plan_file
                .as_ref()
                .map(|path| path.display().to_string()),
            driver_summary_file: args
                .retest_driver_summary_file
                .as_ref()
                .map(|path| path.display().to_string()),
            checkpoint_s3_write: args.output_s3_bucket.is_some(),
        },
    )?;
    let validation = validate_retest_horizon_status(&status)?;
    let output_files =
        write_retest_horizon_status_outputs(args, &status, output_partition_at_ms).await?;

    Ok(RunSummary {
        retest_horizon_plans_created: 0,
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: Some(validation.scheduler_action),
        retest_cycle_run_not_before_ms: validation.run_not_before_ms,
        focused_retest_manifests_created: 0,
        focused_retest_horizon_count: 0,
        focused_retest_candidate_bundle_refs: 0,
        shadow_cycle_decisions_validated: 0,
        shadow_cycle_decisions_created: 0,
        shadow_cycle_scheduler_action: None,
        shadow_cycle_run_not_before_ms: None,
        shadow_cycle_focused_research_manifest_file: None,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: 0,
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files,
    })
}

async fn build_focused_retest_manifest_mode(args: &Args) -> AppResult<RunSummary> {
    let status = load_retest_horizon_status(args).await?;
    build_focused_retest_manifest_from_status(args, &status, None).await
}

async fn run_retest_cycle_scheduler_mode(args: &Args) -> AppResult<RunSummary> {
    let status = load_retest_horizon_status(args).await?;
    let validation = validate_retest_horizon_status(&status)?;
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);

    if validation.scheduler_action == "WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES" {
        let run_not_before_ms = validation.run_not_before_ms.ok_or_else(|| {
            AppError::validation("WAIT scheduler action requires run_not_before_ms")
        })?;
        if output_partition_at_ms < run_not_before_ms {
            return Ok(retest_scheduler_summary(
                validation.scheduler_action,
                Some(run_not_before_ms),
            ));
        }
        return Ok(retest_scheduler_summary(
            "REFRESH_RETEST_HORIZON_STATUS_AFTER_WAIT_DEADLINE".to_owned(),
            Some(run_not_before_ms),
        ));
    }

    if validation.scheduler_action == "RUN_FOCUSED_RETEST_RESEARCH" {
        return build_focused_retest_manifest_from_status(
            args,
            &status,
            Some(validation.scheduler_action),
        )
        .await;
    }

    Ok(retest_scheduler_summary(
        validation.scheduler_action,
        validation.run_not_before_ms,
    ))
}

async fn build_focused_retest_manifest_from_status(
    args: &Args,
    status: &serde_json::Value,
    scheduler_action: Option<String>,
) -> AppResult<RunSummary> {
    let source_manifest = load_input_manifest(args).await?.ok_or_else(|| {
        AppError::config(
            "--build-focused-retest-manifest requires --input-manifest-file or S3 manifest input",
        )
    })?;
    validate_input_manifest(Some(&source_manifest))?;
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let build = build_focused_retest_manifest(
        status,
        &source_manifest,
        &FocusedRetestBuildOptions {
            generated_at_ms: output_partition_at_ms,
            research_packet_id: focused_retest_packet_id(args, output_partition_at_ms),
            run_scope: focused_retest_run_scope(args),
            next_actions: args.focused_retest_next_actions.clone(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode: args.focused_retest_historical_replay_index_ref_mode,
            s3_write: args.output_s3_bucket.is_some(),
        },
    )?;
    let focused_horizon_count = build.summary.focused.focus_horizon_count;
    let focused_candidate_bundle_refs = build.summary.focused.selected_candidate_bundle_ref_count;
    let output_files =
        write_focused_retest_manifest_outputs(args, &build, output_partition_at_ms).await?;

    Ok(RunSummary {
        retest_horizon_plans_created: 0,
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: scheduler_action,
        retest_cycle_run_not_before_ms: None,
        focused_retest_manifests_created: 1,
        focused_retest_horizon_count: focused_horizon_count,
        focused_retest_candidate_bundle_refs: focused_candidate_bundle_refs,
        shadow_cycle_decisions_validated: 0,
        shadow_cycle_decisions_created: 0,
        shadow_cycle_scheduler_action: None,
        shadow_cycle_run_not_before_ms: None,
        shadow_cycle_focused_research_manifest_file: None,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: 0,
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files,
    })
}

fn retest_scheduler_summary(
    scheduler_action: String,
    run_not_before_ms: Option<i64>,
) -> RunSummary {
    RunSummary {
        retest_horizon_plans_created: 0,
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: Some(scheduler_action),
        retest_cycle_run_not_before_ms: run_not_before_ms,
        focused_retest_manifests_created: 0,
        focused_retest_horizon_count: 0,
        focused_retest_candidate_bundle_refs: 0,
        shadow_cycle_decisions_validated: 0,
        shadow_cycle_decisions_created: 0,
        shadow_cycle_scheduler_action: None,
        shadow_cycle_run_not_before_ms: None,
        shadow_cycle_focused_research_manifest_file: None,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: 0,
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files: Vec::new(),
    }
}

async fn load_retest_horizon_status(args: &Args) -> AppResult<serde_json::Value> {
    match (
        args.retest_horizon_status_file.as_deref(),
        args.retest_horizon_status_s3_bucket.as_deref(),
        args.retest_horizon_status_s3_key.as_deref(),
    ) {
        (Some(path), None, None) => read_retest_horizon_status(path),
        (None, Some(bucket), Some(key)) => read_retest_horizon_status_from_s3(bucket, key).await,
        _ => Err(AppError::config(
            "provide either --retest-horizon-status-file or --retest-horizon-status-s3-bucket/--retest-horizon-status-s3-key",
        )),
    }
}

async fn load_retest_horizon_plan(args: &Args) -> AppResult<serde_json::Value> {
    match (
        args.retest_horizon_plan_file.as_deref(),
        args.retest_horizon_plan_s3_bucket.as_deref(),
        args.retest_horizon_plan_s3_key.as_deref(),
    ) {
        (Some(path), None, None) => read_retest_horizon_plan(path),
        (None, Some(bucket), Some(key)) => read_retest_horizon_plan_from_s3(bucket, key).await,
        _ => Err(AppError::config(
            "provide either --retest-horizon-plan-file or --retest-horizon-plan-s3-bucket/--retest-horizon-plan-s3-key",
        )),
    }
}

async fn load_research_report(args: &Args) -> AppResult<crate::model::ResearchRunReport> {
    match (
        args.research_report_file.as_deref(),
        args.research_report_s3_bucket.as_deref(),
        args.research_report_s3_key.as_deref(),
    ) {
        (Some(path), None, None) => read_research_run_report(path),
        (None, Some(bucket), Some(key)) => read_research_run_report_from_s3(bucket, key).await,
        _ => Err(AppError::config(
            "provide either --research-report-file or --research-report-s3-bucket/--research-report-s3-key",
        )),
    }
}

async fn load_paper_watch_candidates(
    args: &Args,
) -> AppResult<Vec<crate::model::PaperWatchCandidate>> {
    match (
        args.paper_watch_candidate_file.as_deref(),
        args.paper_watch_candidate_s3_bucket.as_deref(),
        args.paper_watch_candidate_s3_key.as_deref(),
    ) {
        (Some(path), None, None) => read_paper_watch_candidates(path),
        (None, Some(bucket), Some(key)) => read_paper_watch_candidates_from_s3(bucket, key).await,
        _ => Err(AppError::config(
            "provide either --paper-watch-candidate-file or --paper-watch-candidate-s3-bucket/--paper-watch-candidate-s3-key",
        )),
    }
}

async fn load_market_live_ticks(
    args: &Args,
    candidates: &[crate::model::PaperWatchCandidate],
    run_id_ms: i64,
) -> AppResult<Vec<crate::model::MarketLiveTick>> {
    if let Some(path) = args.market_live_tick_file.as_deref() {
        return read_market_live_ticks(path);
    }
    let Some(url) = args.market_live_nats_url.as_deref() else {
        return Err(AppError::config(
            "provide --market-live-tick-file or --market-live-nats-url",
        ));
    };
    let configs = market_live_nats_configs_for_candidates(args, candidates, url, run_id_ms);
    let mut ticks = Vec::new();
    for config in configs {
        ticks.extend(read_market_live_ticks_from_nats(&config).await?);
    }
    Ok(ticks)
}

fn market_live_nats_configs_for_candidates(
    args: &Args,
    candidates: &[crate::model::PaperWatchCandidate],
    url: &str,
    run_id_ms: i64,
) -> Vec<MarketLiveNatsConfig> {
    let base_config = MarketLiveNatsConfig {
        url: url.to_owned(),
        stream: args.market_live_nats_stream.clone(),
        subject: args.market_live_nats_subject.clone(),
        consumer: args.market_live_nats_consumer.clone(),
        deliver_policy: args.market_live_nats_deliver_policy.clone(),
        batch_size: args.market_live_nats_batch_size,
        max_messages: args.market_live_nats_max_messages,
        ack_wait_secs: args.market_live_nats_ack_wait_secs,
        delete_consumer_after_read: false,
    };
    if args.market_live_nats_subject != DEFAULT_MARKET_LIVE_NATS_SUBJECT {
        return vec![base_config];
    }

    let symbols = candidates
        .iter()
        .map(|candidate| market_live_subject_symbol_token(&candidate.symbol_canonical))
        .filter(|symbol| !symbol.is_empty())
        .collect::<BTreeSet<_>>();
    if symbols.is_empty() {
        return vec![base_config];
    }

    symbols
        .into_iter()
        .map(|symbol| MarketLiveNatsConfig {
            subject: format!("market_live_tick.created.*.{symbol}"),
            consumer: format!("{}-{run_id_ms}-{symbol}", args.market_live_nats_consumer),
            delete_consumer_after_read: true,
            ..base_config.clone()
        })
        .collect()
}

fn market_live_subject_symbol_token(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if character == '_' || character == '-' {
                Some(character)
            } else {
                None
            }
        })
        .collect()
}

async fn retest_plan_latest_l1_as_of_ms(args: &Args) -> AppResult<Option<i64>> {
    if let Some(latest_l1_as_of_ms) = args.retest_horizon_latest_l1_as_of_ms {
        return Ok(Some(latest_l1_as_of_ms));
    }
    let Some(bucket) = args.market_l1_s3_bucket.as_deref() else {
        return Ok(None);
    };
    if bucket.contains('<') || bucket.contains('>') {
        return Ok(None);
    }
    discover_latest_symbol_universe_snapshot_end_ms_from_s3(bucket).await
}

async fn shadow_cycle_latest_l1_as_of_ms(args: &Args) -> AppResult<Option<i64>> {
    if let Some(latest_l1_as_of_ms) = args.shadow_cycle_latest_l1_as_of_ms {
        return Ok(Some(latest_l1_as_of_ms));
    }
    let Some(bucket) = args.market_l1_s3_bucket.as_deref() else {
        return Ok(None);
    };
    if bucket.contains('<') || bucket.contains('>') {
        return Ok(None);
    }
    discover_latest_symbol_universe_snapshot_end_ms_from_s3(bucket).await
}

fn input_manifest_label(args: &Args) -> String {
    if let Some(path) = args.input_manifest_file.as_deref() {
        return path.display().to_string();
    }
    match (
        args.input_manifest_s3_bucket.as_deref(),
        args.input_manifest_s3_key.as_deref(),
    ) {
        (Some(bucket), Some(key)) => format!("s3://{bucket}/{key}"),
        _ => "unknown".to_owned(),
    }
}

fn research_report_label(args: &Args) -> String {
    if let Some(path) = args.research_report_file.as_deref() {
        return path.display().to_string();
    }
    match (
        args.research_report_s3_bucket.as_deref(),
        args.research_report_s3_key.as_deref(),
    ) {
        (Some(bucket), Some(key)) => format!("s3://{bucket}/{key}"),
        _ => "unknown".to_owned(),
    }
}

fn load_retest_driver_summary(args: &Args) -> AppResult<Option<serde_json::Value>> {
    let Some(path) = args.retest_driver_summary_file.as_deref() else {
        return Ok(None);
    };
    if !path.is_absolute() {
        return Err(AppError::config(
            "retest driver summary file must be an absolute path",
        ));
    }
    let bytes = fs::read(path)?;
    let value = serde_json::from_slice(&bytes)?;
    Ok(Some(value))
}

async fn write_retest_refresh_cycle_plan_output(
    args: &Args,
    plan: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    write_retest_refresh_cycle_checkpoint_output(
        args,
        plan,
        output_partition_at_ms,
        RetestRefreshCheckpointKind::Plan,
    )
    .await
}

async fn write_retest_refresh_cycle_status_output(
    args: &Args,
    status: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    write_retest_refresh_cycle_checkpoint_output(
        args,
        status,
        output_partition_at_ms,
        RetestRefreshCheckpointKind::Status,
    )
    .await
}

enum RetestRefreshCheckpointKind {
    Plan,
    Status,
}

impl RetestRefreshCheckpointKind {
    fn local_filename(&self) -> &'static str {
        match self {
            Self::Plan => "retest-horizon-plan.json",
            Self::Status => "retest-horizon-status.json",
        }
    }

    fn s3_prefix(&self) -> &'static str {
        match self {
            Self::Plan => "retest-horizon-plan/schema=research_retest_horizon_plan_v1",
            Self::Status => "retest-horizon-status/schema=research_horizon_status_checkpoint_v1",
        }
    }
}

async fn write_retest_refresh_cycle_checkpoint_output(
    args: &Args,
    value: &serde_json::Value,
    output_partition_at_ms: i64,
    kind: RetestRefreshCheckpointKind,
) -> AppResult<Vec<String>> {
    if let Some(output_dir) = args.output_dir.as_deref() {
        let path = output_dir.join(kind.local_filename());
        return Ok(vec![
            write_pretty_json_file(&path, value)?.display().to_string(),
        ]);
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-retest-refresh-cycle requires --output-dir or --output-s3-bucket",
        ));
    };
    let uri = match kind {
        RetestRefreshCheckpointKind::Plan => {
            write_retest_horizon_plan_to_s3(bucket, kind.s3_prefix(), value, output_partition_at_ms)
                .await?
        }
        RetestRefreshCheckpointKind::Status => {
            write_retest_horizon_status_to_s3(
                bucket,
                kind.s3_prefix(),
                value,
                output_partition_at_ms,
            )
            .await?
        }
    };
    Ok(vec![uri])
}

async fn write_retest_refresh_cycle_focused_manifest_output(
    args: &Args,
    build: &FocusedRetestManifestBuild,
) -> AppResult<FocusedRetestManifestWriteResult> {
    if let Some(output_dir) = args.output_dir.as_deref() {
        let manifest_path = output_dir.join("research-input-manifest.json");
        let summary_path = output_dir.join("research-input-manifest.summary.json");
        return Ok(FocusedRetestManifestWriteResult {
            created: true,
            output_files: vec![
                write_research_input_manifest(&manifest_path, &build.manifest)?
                    .display()
                    .to_string(),
                write_pretty_json_file(&summary_path, &build.summary)?
                    .display()
                    .to_string(),
            ],
        });
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-retest-refresh-cycle requires --output-dir or --output-s3-bucket",
        ));
    };
    let packet_id = build
        .manifest
        .research_packet_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::validation("focused retest manifest missing research_packet_id")
        })?;
    let key = focused_retest_dispatch_manifest_s3_key(packet_id)?;
    match write_research_input_manifest_to_exact_s3_key_if_absent(bucket, &key, &build.manifest)
        .await?
    {
        Some(uri) => Ok(FocusedRetestManifestWriteResult {
            created: true,
            output_files: vec![uri],
        }),
        None => Ok(FocusedRetestManifestWriteResult {
            created: false,
            output_files: vec![format!("s3://{bucket}/{key}")],
        }),
    }
}

struct FocusedRetestManifestWriteResult {
    created: bool,
    output_files: Vec<String>,
}

async fn write_retest_horizon_plan_outputs(
    args: &Args,
    plan: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(path) = args.retest_horizon_plan_output_file.as_deref() {
        return Ok(vec![
            write_pretty_json_file(path, plan)?.display().to_string(),
        ]);
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--build-retest-horizon-plan requires --retest-horizon-plan-output-file or --output-s3-bucket",
        ));
    };
    let uri = write_retest_horizon_plan_to_s3(
        bucket,
        args.output_s3_prefix.as_deref().unwrap_or(""),
        plan,
        output_partition_at_ms,
    )
    .await?;
    Ok(vec![uri])
}

async fn write_retest_horizon_status_outputs(
    args: &Args,
    status: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(path) = args.retest_horizon_status_output_file.as_deref() {
        return Ok(vec![
            write_pretty_json_file(path, status)?.display().to_string(),
        ]);
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--build-retest-horizon-status requires --retest-horizon-status-output-file or --output-s3-bucket",
        ));
    };
    let uri = write_retest_horizon_status_to_s3(
        bucket,
        args.output_s3_prefix.as_deref().unwrap_or(""),
        status,
        output_partition_at_ms,
    )
    .await?;
    Ok(vec![uri])
}

async fn write_focused_retest_manifest_outputs(
    args: &Args,
    build: &FocusedRetestManifestBuild,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(path) = args.focused_retest_manifest_output_file.as_deref() {
        let mut output_files = Vec::new();
        output_files.push(
            write_research_input_manifest(path, &build.manifest)?
                .display()
                .to_string(),
        );
        let summary_path = focused_retest_summary_output_path(args, path);
        output_files.push(
            write_pretty_json_file(&summary_path, &build.summary)?
                .display()
                .to_string(),
        );
        return Ok(output_files);
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires --focused-retest-manifest-output-file or --output-s3-bucket",
        ));
    };
    let uri = write_research_input_manifest_to_s3(
        bucket,
        args.output_s3_prefix.as_deref().unwrap_or(""),
        &build.manifest,
        output_partition_at_ms,
    )
    .await?;
    Ok(vec![uri])
}

fn focused_retest_summary_output_path(
    args: &Args,
    manifest_output_file: &std::path::Path,
) -> PathBuf {
    args.focused_retest_summary_output_file
        .clone()
        .unwrap_or_else(|| {
            PathBuf::from(format!("{}.summary.json", manifest_output_file.display()))
        })
}

fn focused_retest_packet_id(args: &Args, output_partition_at_ms: i64) -> String {
    if args.research_packet_id == "local_research_packet" {
        format!("research_focus_{output_partition_at_ms}")
    } else {
        args.research_packet_id.clone()
    }
}

fn focused_retest_dispatch_packet_id(
    args: &Args,
    latest_l1_as_of_ms: Option<i64>,
    build: &FocusedRetestManifestBuild,
) -> AppResult<String> {
    let latest_l1_part = latest_l1_as_of_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let focus_next_actions = serde_json::to_string(&build.summary.focus_next_actions)?;
    let focus_rows = serde_json::to_string(&build.summary.focused.rows)?;
    let candidate_refs = serde_json::to_string(&build.manifest.candidate_bundle_refs)?;
    let historical_index_refs =
        serde_json::to_string(&build.manifest.historical_replay_run_index_refs)?;
    let manifest_label = input_manifest_label(args);
    let report_label = research_report_label(args);
    let run_scope = focused_retest_run_scope(args);
    let parts = [
        "research_retest_refresh_focused_dispatch_v1".to_owned(),
        manifest_label,
        report_label,
        latest_l1_part,
        run_scope,
        focus_next_actions,
        focus_rows,
        candidate_refs,
        historical_index_refs,
    ];
    let part_refs = parts.iter().map(String::as_str).collect::<Vec<_>>();
    Ok(stable_id("research_focus", &part_refs))
}

fn focused_retest_dispatch_manifest_s3_key(packet_id: &str) -> AppResult<String> {
    let packet_id = packet_id.trim();
    if packet_id.is_empty() {
        return Err(AppError::validation(
            "focused retest dispatch packet id must not be empty",
        ));
    }
    if packet_id.contains('/') {
        return Err(AppError::validation(
            "focused retest dispatch packet id must not contain /",
        ));
    }
    Ok(format!(
        "research-input-manifest/schema=research_input_manifest_v1/dedupe_key={packet_id}/manifest.json"
    ))
}

fn focused_retest_run_scope(args: &Args) -> String {
    if args.run_scope == "p0_candidate_bundle_local" {
        "focused_retest_local_validation".to_owned()
    } else {
        args.run_scope.clone()
    }
}

async fn write_shadow_cycle_decision_outputs(
    args: &Args,
    decision: &crate::model::ShadowCycleDecision,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(output_file) = args.shadow_cycle_decision_output_file.as_deref() {
        return write_shadow_cycle_decision(output_file, decision)
            .map(|path| vec![path.display().to_string()]);
    }
    if let Some(output_dir) = args.output_dir.as_deref() {
        return write_shadow_cycle_decision_to_dir(output_dir, decision, output_partition_at_ms)
            .map(|path| vec![path.display().to_string()]);
    }
    if let Some(output_bucket) = args.output_s3_bucket.as_deref() {
        return write_shadow_cycle_decision_to_s3(
            output_bucket,
            args.output_s3_prefix.as_deref().unwrap_or(""),
            decision,
            output_partition_at_ms,
        )
        .await
        .map(|uri| vec![uri]);
    }
    Err(AppError::config(
        "shadow cycle decision output target is required",
    ))
}

async fn run_shadow_cycle_from_latest_state_mode(args: &Args) -> AppResult<RunSummary> {
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let output_bucket = args.output_s3_bucket.as_deref().ok_or_else(|| {
        AppError::config("--run-shadow-cycle-from-latest-state requires --output-s3-bucket")
    })?;
    let shadow_keys = discover_shadow_validation_run_keys_from_s3(
        output_bucket,
        DEFAULT_SHADOW_VALIDATION_RUN_PREFIX,
        DEFAULT_SHADOW_VALIDATION_RUN_READ_LIMIT,
        DEFAULT_SHADOW_VALIDATION_RUN_SCAN_LIMIT,
    )
    .await?;
    let shadow_runs = if shadow_keys.is_empty() {
        Vec::new()
    } else {
        read_shadow_validation_runs_from_s3(output_bucket, &shadow_keys).await?
    };
    let latest_l1_as_of_ms = shadow_cycle_latest_l1_as_of_ms(args).await?;
    let mut decision =
        build_shadow_cycle_decision(&shadow_runs, latest_l1_as_of_ms, output_partition_at_ms);
    let mut focused_retest_manifests_created = 0usize;
    let mut focused_retest_horizon_count = 0usize;
    let mut focused_retest_candidate_bundle_refs = 0usize;
    let mut output_files = Vec::new();

    if decision.scheduler_action == ShadowCycleSchedulerAction::HoldForOperatorReview
        && let Some(dispatch) = try_build_shadow_accumulation_manifest_from_latest_state(
            args,
            &shadow_runs,
            latest_l1_as_of_ms,
            output_partition_at_ms,
        )
        .await?
    {
        if dispatch.created {
            focused_retest_manifests_created = 1;
        }
        focused_retest_horizon_count = dispatch.focused_horizon_count;
        focused_retest_candidate_bundle_refs = dispatch.focused_candidate_bundle_refs;
        output_files.push(dispatch.manifest_uri.clone());
        decision.scheduler_action =
            ShadowCycleSchedulerAction::RunFocusedShadowSampleAccumulationResearch;
        let latest_l1_part = latest_l1_as_of_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_owned());
        let deficit_lifecycle_key_part = dispatch.deficit_lifecycle_keys.join("|");
        decision.decision_id = stable_id(
            "shadow_cycle_decision",
            &[
                "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION",
                latest_l1_part.as_str(),
                dispatch.manifest_uri.as_str(),
                deficit_lifecycle_key_part.as_str(),
            ],
        );
        decision.focused_research_manifest_file = Some(dispatch.manifest_uri);
        decision.safe_next_actions = vec![
            "run_focused_shadow_sample_accumulation_research".to_owned(),
            "keep_shadow_status_pending_until_completion_evidence_exists".to_owned(),
        ];
    }
    validate_shadow_cycle_decision(&decision)?;
    let output_files = append_output_files(
        output_files,
        write_shadow_cycle_decision_outputs(args, &decision, output_partition_at_ms).await?,
    );
    emit_shadow_cycle_decision_alert_from_env(&decision).await;

    Ok(RunSummary {
        shadow_cycle_decisions_validated: 1,
        shadow_cycle_decisions_created: 1,
        shadow_cycle_scheduler_action: Some(decision.scheduler_action),
        shadow_cycle_run_not_before_ms: decision.run_not_before_ms,
        shadow_cycle_focused_research_manifest_file: decision.focused_research_manifest_file,
        focused_retest_manifests_created,
        focused_retest_horizon_count,
        focused_retest_candidate_bundle_refs,
        shadow_validation_runs_loaded: shadow_runs.len(),
        output_files,
        ..RunSummary::default()
    })
}

#[derive(Debug)]
struct ShadowAccumulationDispatch {
    manifest_uri: String,
    created: bool,
    focused_horizon_count: usize,
    focused_candidate_bundle_refs: usize,
    deficit_lifecycle_keys: Vec<String>,
}

#[derive(Debug)]
struct ShadowAccumulationManifestDispatch {
    key: String,
    manifest: ResearchInputManifest,
    focused_horizon_count: usize,
    focused_candidate_bundle_refs: usize,
    deficit_lifecycle_keys: Vec<String>,
}

async fn try_build_shadow_accumulation_manifest_from_latest_state(
    args: &Args,
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
    output_partition_at_ms: i64,
) -> AppResult<Option<ShadowAccumulationDispatch>> {
    let deficit_lifecycle_keys =
        shadow_sample_deficit_lifecycle_keys(shadow_runs, latest_l1_as_of_ms);
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Ok(None);
    };
    let state = match read_latest_retest_cycle_source_state_from_s3(bucket, "").await {
        Ok(state) => state,
        Err(AppError::AwsNotFound(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    let status = match read_latest_retest_horizon_status_from_s3(bucket, "").await {
        Ok(status) => status,
        Err(AppError::AwsNotFound(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    let source_manifest = read_research_input_manifest_from_s3(
        &state.source_manifest_s3_bucket,
        &state.source_manifest_s3_key,
    )
    .await?;
    validate_input_manifest(Some(&source_manifest))?;

    let Some(dispatch_build) = build_shadow_accumulation_manifest_dispatch(
        args,
        &state,
        &status,
        &source_manifest,
        latest_l1_as_of_ms,
        output_partition_at_ms,
        deficit_lifecycle_keys,
    )?
    else {
        return Ok(None);
    };

    let write_result = write_research_input_manifest_to_exact_s3_key_if_absent(
        bucket,
        &dispatch_build.key,
        &dispatch_build.manifest,
    )
    .await?;
    let created = write_result.is_some();
    let manifest_uri =
        write_result.unwrap_or_else(|| format!("s3://{bucket}/{}", dispatch_build.key));

    Ok(Some(ShadowAccumulationDispatch {
        manifest_uri,
        created,
        focused_horizon_count: dispatch_build.focused_horizon_count,
        focused_candidate_bundle_refs: dispatch_build.focused_candidate_bundle_refs,
        deficit_lifecycle_keys: dispatch_build.deficit_lifecycle_keys,
    }))
}

fn build_shadow_accumulation_manifest_dispatch(
    args: &Args,
    state: &RetestCycleSourceState,
    status: &serde_json::Value,
    source_manifest: &ResearchInputManifest,
    latest_l1_as_of_ms: Option<i64>,
    output_partition_at_ms: i64,
    deficit_lifecycle_keys: Vec<String>,
) -> AppResult<Option<ShadowAccumulationManifestDispatch>> {
    if deficit_lifecycle_keys.is_empty() {
        return Ok(None);
    }
    let mut build = match build_focused_retest_manifest(
        status,
        source_manifest,
        &FocusedRetestBuildOptions {
            generated_at_ms: output_partition_at_ms,
            research_packet_id: "research_shadow_accumulation_pending".to_owned(),
            run_scope: "shadow_sample_accumulation_local_validation".to_owned(),
            next_actions: args.focused_retest_next_actions.clone(),
            candidate_lifecycle_key_filter: deficit_lifecycle_keys.clone(),
            historical_replay_index_ref_mode: args.focused_retest_historical_replay_index_ref_mode,
            s3_write: true,
        },
    ) {
        Ok(build) => build,
        Err(AppError::Validation(message))
            if message.contains("selected zero candidate bundle refs") =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let packet_id = shadow_accumulation_dispatch_packet_id(
        state,
        latest_l1_as_of_ms,
        &deficit_lifecycle_keys,
        &build,
    )?;
    build.manifest.research_packet_id = Some(packet_id.clone());
    let key = focused_retest_dispatch_manifest_s3_key(&packet_id)?;

    Ok(Some(ShadowAccumulationManifestDispatch {
        key,
        manifest: build.manifest,
        focused_horizon_count: build.summary.focused.focus_horizon_count,
        focused_candidate_bundle_refs: build.summary.focused.selected_candidate_bundle_ref_count,
        deficit_lifecycle_keys,
    }))
}

fn append_output_files(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    left.extend(right);
    left
}

fn shadow_accumulation_dispatch_packet_id(
    state: &RetestCycleSourceState,
    latest_l1_as_of_ms: Option<i64>,
    deficit_lifecycle_keys: &[String],
    build: &FocusedRetestManifestBuild,
) -> AppResult<String> {
    let latest_l1_part = latest_l1_as_of_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let focus_rows = serde_json::to_string(&build.summary.focused.rows)?;
    let candidate_refs = serde_json::to_string(&build.manifest.candidate_bundle_refs)?;
    let historical_index_refs =
        serde_json::to_string(&build.manifest.historical_replay_run_index_refs)?;
    let deficit_keys = serde_json::to_string(deficit_lifecycle_keys)?;
    let parts = [
        "research_shadow_accumulation_dispatch_v1",
        state.source_manifest_s3_key.as_str(),
        state.source_research_report_s3_key.as_str(),
        latest_l1_part.as_str(),
        deficit_keys.as_str(),
        focus_rows.as_str(),
        candidate_refs.as_str(),
        historical_index_refs.as_str(),
    ];
    Ok(stable_id("research_shadow_accumulation", &parts))
}

async fn load_input_manifest(args: &Args) -> AppResult<Option<ResearchInputManifest>> {
    if let Some(path) = args.input_manifest_file.as_deref() {
        return read_research_input_manifest(path).map(Some);
    }
    match (
        args.input_manifest_s3_bucket.as_deref(),
        args.input_manifest_s3_key.as_deref(),
    ) {
        (Some(bucket), Some(key)) => read_research_input_manifest_from_s3(bucket, key)
            .await
            .map(Some),
        _ => Ok(None),
    }
}

fn validate_input_manifest(manifest: Option<&ResearchInputManifest>) -> AppResult<()> {
    let Some(manifest) = manifest else {
        return Ok(());
    };
    if manifest.schema_version != RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "research input manifest schema_version must be {RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION}; got {}",
            manifest.schema_version
        )));
    }
    for artifact_ref in all_manifest_refs(manifest) {
        validate_artifact_ref(artifact_ref)?;
    }
    Ok(())
}

fn validate_manifest_budget(
    manifest: Option<&ResearchInputManifest>,
    budget: &ResearchRuntimeBudgetPolicy,
) -> AppResult<()> {
    let Some(manifest) = manifest else {
        return Ok(());
    };
    enforce_budget(
        "candidate_bundle_ref_count",
        manifest.candidate_bundle_refs.len(),
        budget.max_candidate_bundle_count,
    )?;
    enforce_budget(
        "market_artifact_ref_count",
        manifest.market_feature_delta_refs.len() + manifest.market_regime_context_refs.len(),
        budget.max_market_artifact_ref_count,
    )?;
    enforce_budget(
        "shadow_validation_run_ref_count",
        manifest.shadow_validation_run_refs.len(),
        budget.max_shadow_validation_run_ref_count,
    )?;
    enforce_budget(
        "hypothesis_harness_result_ref_count",
        manifest.hypothesis_harness_result_refs.len(),
        budget.max_hypothesis_harness_result_ref_count,
    )?;
    enforce_budget(
        "oss_adapter_run_ref_count",
        manifest.oss_adapter_run_refs.len(),
        budget.max_oss_adapter_run_ref_count,
    )?;
    enforce_budget(
        "historical_replay_run_ref_count",
        manifest.historical_replay_run_refs.len() + manifest.historical_replay_run_index_refs.len(),
        budget.max_historical_replay_run_ref_count,
    )?;
    Ok(())
}

fn enforce_budget(name: &str, actual: usize, maximum: usize) -> AppResult<()> {
    if maximum == 0 {
        return Err(AppError::config(format!(
            "runtime_budget_policy.{name} maximum must be greater than zero"
        )));
    }
    if actual > maximum {
        return Err(AppError::validation(format!(
            "runtime budget exceeded for {name}: actual={actual}, max={maximum}"
        )));
    }
    Ok(())
}

fn all_manifest_refs(manifest: &ResearchInputManifest) -> Vec<&ResearchArtifactRef> {
    manifest
        .candidate_bundle_refs
        .iter()
        .chain(manifest.market_feature_delta_refs.iter())
        .chain(manifest.market_regime_context_refs.iter())
        .chain(manifest.shadow_validation_run_refs.iter())
        .chain(manifest.hypothesis_harness_result_refs.iter())
        .chain(manifest.oss_adapter_run_refs.iter())
        .chain(manifest.historical_replay_run_refs.iter())
        .chain(manifest.historical_replay_run_index_refs.iter())
        .collect()
}

fn validate_artifact_ref(artifact_ref: &ResearchArtifactRef) -> AppResult<()> {
    let location = ArtifactLocation::from_uri(&artifact_ref.uri)?;
    match location {
        ArtifactLocation::Local(path) if !path.is_absolute() => Err(AppError::config(format!(
            "manifest artifact uri must be an absolute path or s3 URI: {}",
            artifact_ref.uri
        ))),
        _ => Ok(()),
    }
}

async fn read_input_bundles(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    let mut bundles = Vec::new();
    if let Some(path) = args.input_bundle_file.as_deref() {
        append_unique_bundles(&mut bundles, read_candidate_bundles(path)?);
    }
    if let (Some(bucket), Some(key)) = (
        args.input_bundle_s3_bucket.as_deref(),
        args.input_bundle_s3_key.as_deref(),
    ) {
        append_unique_bundles(
            &mut bundles,
            read_candidate_bundles_from_s3(bucket, key).await?,
        );
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.candidate_bundle_refs {
            append_unique_bundles(
                &mut bundles,
                read_candidate_bundles_from_ref(artifact_ref).await?,
            );
        }
    }
    Ok(bundles)
}

pub fn print_help() {
    println!("{}", help_text());
}

fn build_replay_runs(
    bundles: &[crate::model::IntelCandidateEvidenceBundle],
    market_deltas: &[MarketFeatureDelta],
    regime_contexts: &[MarketRegimeContext],
) -> Vec<ReplayRun> {
    let mut replay_runs = Vec::new();
    for bundle in bundles {
        let admission = validate_bundle_admission(bundle);
        if !admission.admitted {
            replay_runs.push(build_invalid_replay_run(bundle, &admission));
            continue;
        }
        replay_runs.extend(run_native_replay(bundle, market_deltas, regime_contexts));
    }
    replay_runs
}

async fn load_market_deltas(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
    manifest: Option<&ResearchInputManifest>,
    max_market_artifact_ref_count: usize,
) -> AppResult<Vec<MarketFeatureDelta>> {
    let mut deltas = Vec::new();
    if let Some(path) = args.market_feature_delta_file.as_deref() {
        deltas.extend(read_market_feature_deltas(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.market_feature_delta_refs {
            deltas.extend(read_market_feature_deltas_from_ref(artifact_ref).await?);
        }
    }
    if !should_read_market_s3(args) {
        return Ok(deltas);
    }
    let keys = market_feature_delta_s3_keys(args, bundles).await?;
    enforce_budget(
        "market_feature_delta_s3_key_count",
        keys.len(),
        max_market_artifact_ref_count,
    )?;
    if keys.is_empty() {
        return Ok(deltas);
    }
    let symbols = bundle_symbol_filter(bundles);
    deltas.extend(
        read_market_feature_deltas_from_s3(market_l1_s3_bucket(args), &keys, &symbols).await?,
    );
    Ok(deltas)
}

async fn load_regime_contexts(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
    manifest: Option<&ResearchInputManifest>,
    max_market_artifact_ref_count: usize,
) -> AppResult<Vec<MarketRegimeContext>> {
    let mut contexts = Vec::new();
    if let Some(path) = args.market_regime_context_file.as_deref() {
        contexts.extend(read_market_regime_contexts(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.market_regime_context_refs {
            contexts.extend(read_market_regime_contexts_from_ref(artifact_ref).await?);
        }
    }
    if !should_read_market_s3(args) {
        return Ok(contexts);
    }
    let keys = market_regime_context_s3_keys(args, bundles).await?;
    enforce_budget(
        "market_regime_context_s3_key_count",
        keys.len(),
        max_market_artifact_ref_count,
    )?;
    if keys.is_empty() {
        return Ok(contexts);
    }
    contexts.extend(read_market_regime_contexts_from_s3(market_l1_s3_bucket(args), &keys).await?);
    Ok(contexts)
}

async fn load_historical_replay_runs(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
    max_historical_replay_run_ref_count: usize,
) -> AppResult<Vec<ReplayRun>> {
    let mut replay_runs = Vec::new();
    for path in &args.historical_replay_run_files {
        append_unique_replay_runs(&mut replay_runs, read_replay_runs(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.historical_replay_run_refs {
            append_unique_replay_runs(
                &mut replay_runs,
                read_replay_runs_from_ref(artifact_ref).await?,
            );
        }
    }
    if !args.historical_replay_run_s3_keys.is_empty() {
        let bucket = args
            .historical_replay_run_s3_bucket
            .as_deref()
            .ok_or_else(|| {
                AppError::config("RESEARCH_HISTORICAL_REPLAY_RUN_S3_BUCKET is required")
            })?;
        append_unique_replay_runs(
            &mut replay_runs,
            read_replay_runs_from_s3(bucket, &args.historical_replay_run_s3_keys).await?,
        );
    }
    let index_records = load_historical_replay_run_index_records(
        args,
        manifest,
        max_historical_replay_run_ref_count,
    )
    .await?;
    append_unique_replay_runs(
        &mut replay_runs,
        load_replay_runs_from_index_records(&index_records).await?,
    );
    Ok(replay_runs)
}

async fn load_historical_replay_run_index_records(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
    max_historical_replay_run_ref_count: usize,
) -> AppResult<Vec<ReplayRunIndexRecord>> {
    let mut records = Vec::new();
    for path in &args.historical_replay_run_index_files {
        records.extend(read_replay_run_index_records(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.historical_replay_run_index_refs {
            records.extend(read_replay_run_index_records_from_ref(artifact_ref).await?);
        }
    }
    if !args.historical_replay_run_index_s3_keys.is_empty() {
        let bucket = args
            .historical_replay_run_index_s3_bucket
            .as_deref()
            .ok_or_else(|| {
                AppError::config("RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET is required")
            })?;
        records.extend(
            read_replay_run_index_records_from_s3(
                bucket,
                &args.historical_replay_run_index_s3_keys,
            )
            .await?,
        );
    }
    if let Some(prefix) = env_string("RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX") {
        let bucket = args
            .historical_replay_run_index_s3_bucket
            .as_deref()
            .or(args.output_s3_bucket.as_deref())
            .ok_or_else(|| {
                AppError::config(
                    "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET or RESEARCH_OUTPUT_S3_BUCKET is required when RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX is set",
                )
            })?;
        let read_limit = env_usize(
            "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_READ_LIMIT",
            DEFAULT_HISTORICAL_REPLAY_RUN_INDEX_READ_LIMIT,
        )?;
        let scan_limit = env_usize(
            "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_SCAN_LIMIT",
            DEFAULT_HISTORICAL_REPLAY_RUN_INDEX_SCAN_LIMIT,
        )?;
        let discovered_keys =
            discover_replay_run_index_keys_from_s3(bucket, &prefix, read_limit, scan_limit).await?;
        enforce_budget(
            "historical_replay_run_index_s3_prefix_key_count",
            discovered_keys.len(),
            max_historical_replay_run_ref_count,
        )?;
        records.extend(read_replay_run_index_records_from_s3(bucket, &discovered_keys).await?);
    }
    Ok(records)
}

async fn load_oss_adapter_runs(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
) -> AppResult<Vec<OssAdapterRun>> {
    let mut runs = Vec::new();
    for path in &args.oss_adapter_run_files {
        append_unique_oss_adapter_runs(&mut runs, read_oss_adapter_runs(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.oss_adapter_run_refs {
            append_unique_oss_adapter_runs(
                &mut runs,
                read_oss_adapter_runs_from_ref(artifact_ref).await?,
            );
        }
    }
    if !args.oss_adapter_run_s3_keys.is_empty() {
        let bucket = args
            .oss_adapter_run_s3_bucket
            .as_deref()
            .ok_or_else(|| AppError::config("RESEARCH_OSS_ADAPTER_RUN_S3_BUCKET is required"))?;
        append_unique_oss_adapter_runs(
            &mut runs,
            read_oss_adapter_runs_from_s3(bucket, &args.oss_adapter_run_s3_keys).await?,
        );
    }
    Ok(runs)
}

async fn load_shadow_validation_runs(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
) -> AppResult<Vec<ShadowValidationRun>> {
    let mut runs = Vec::new();
    for path in &args.shadow_validation_run_files {
        append_unique_shadow_validation_runs(&mut runs, read_shadow_validation_runs(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.shadow_validation_run_refs {
            append_unique_shadow_validation_runs(
                &mut runs,
                read_shadow_validation_runs_from_ref(artifact_ref).await?,
            );
        }
    }
    if !args.shadow_validation_run_s3_keys.is_empty() {
        let bucket = args
            .shadow_validation_run_s3_bucket
            .as_deref()
            .ok_or_else(|| {
                AppError::config("RESEARCH_SHADOW_VALIDATION_RUN_S3_BUCKET is required")
            })?;
        append_unique_shadow_validation_runs(
            &mut runs,
            read_shadow_validation_runs_from_s3(bucket, &args.shadow_validation_run_s3_keys)
                .await?,
        );
    }
    Ok(runs)
}

fn validate_oss_adapter_runs(runs: &[OssAdapterRun]) -> AppResult<()> {
    for run in runs {
        if run.schema_version != OSS_ADAPTER_RUN_SCHEMA_VERSION {
            return Err(AppError::validation(format!(
                "oss adapter run schema_version must be {OSS_ADAPTER_RUN_SCHEMA_VERSION}; got {}",
                run.schema_version
            )));
        }
        if !run.lookahead_check_result.eq_ignore_ascii_case("passed") {
            return Err(AppError::validation(format!(
                "oss adapter run {} failed lookahead check: {}",
                run.oss_adapter_run_id, run.lookahead_check_result
            )));
        }
        if !run
            .holding_horizon_check_result
            .eq_ignore_ascii_case("passed")
        {
            return Err(AppError::validation(format!(
                "oss adapter run {} failed holding horizon check: {}",
                run.oss_adapter_run_id, run.holding_horizon_check_result
            )));
        }
    }
    Ok(())
}

async fn read_candidate_bundles_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    match ArtifactLocation::from_uri(&artifact_ref.uri)? {
        ArtifactLocation::Local(path) => read_candidate_bundles(&path),
        ArtifactLocation::S3 { bucket, key } => read_candidate_bundles_from_s3(&bucket, &key).await,
    }
}

async fn read_market_feature_deltas_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<MarketFeatureDelta>> {
    match ArtifactLocation::from_uri(&artifact_ref.uri)? {
        ArtifactLocation::Local(path) => read_market_feature_deltas(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_market_feature_deltas_from_s3(
                &bucket,
                std::slice::from_ref(&key),
                &BTreeSet::new(),
            )
            .await
        }
    }
}

async fn read_market_regime_contexts_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<MarketRegimeContext>> {
    match ArtifactLocation::from_uri(&artifact_ref.uri)? {
        ArtifactLocation::Local(path) => read_market_regime_contexts(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_market_regime_contexts_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

async fn read_replay_runs_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<ReplayRun>> {
    match ArtifactLocation::from_uri(&artifact_ref.uri)? {
        ArtifactLocation::Local(path) => read_replay_runs(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_replay_runs_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

async fn read_replay_run_index_records_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<ReplayRunIndexRecord>> {
    match ArtifactLocation::from_uri(&artifact_ref.uri)? {
        ArtifactLocation::Local(path) => read_replay_run_index_records(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_replay_run_index_records_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

async fn read_oss_adapter_runs_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<OssAdapterRun>> {
    match ArtifactLocation::from_uri(&artifact_ref.uri)? {
        ArtifactLocation::Local(path) => read_oss_adapter_runs(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_oss_adapter_runs_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

async fn read_shadow_validation_runs_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<ShadowValidationRun>> {
    match ArtifactLocation::from_uri(&artifact_ref.uri)? {
        ArtifactLocation::Local(path) => read_shadow_validation_runs(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_shadow_validation_runs_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

enum ArtifactLocation {
    Local(PathBuf),
    S3 { bucket: String, key: String },
}

impl ArtifactLocation {
    fn from_uri(uri: &str) -> AppResult<Self> {
        let trimmed = uri.trim();
        if trimmed.is_empty() {
            return Err(AppError::config("manifest artifact uri must not be empty"));
        }
        if let Some((bucket, key)) = parse_s3_uri(trimmed) {
            return Ok(Self::S3 { bucket, key });
        }
        Ok(Self::Local(PathBuf::from(trimmed)))
    }
}

async fn load_replay_runs_from_index_records(
    records: &[ReplayRunIndexRecord],
) -> AppResult<Vec<ReplayRun>> {
    let mut local_locations = BTreeMap::<PathBuf, BTreeSet<String>>::new();
    let mut s3_locations = BTreeMap::<(String, String), BTreeSet<String>>::new();

    for record in records {
        if let (Some(bucket), Some(key)) = (
            record.replay_run_s3_bucket.as_deref(),
            record.replay_run_s3_key.as_deref(),
        ) {
            s3_locations
                .entry((bucket.to_owned(), key.to_owned()))
                .or_default()
                .insert(record.replay_run_id.clone());
            continue;
        }
        if let Some((bucket, key)) = parse_s3_uri(&record.replay_run_uri) {
            s3_locations
                .entry((bucket, key))
                .or_default()
                .insert(record.replay_run_id.clone());
            continue;
        }
        let path = PathBuf::from(&record.replay_run_uri);
        if !path.is_absolute() {
            return Err(AppError::config(format!(
                "replay_run_index replay_run_uri must be an absolute path or s3 URI: {}",
                record.replay_run_uri
            )));
        }
        local_locations
            .entry(path)
            .or_default()
            .insert(record.replay_run_id.clone());
    }

    let mut replay_runs = Vec::new();
    for (path, expected_ids) in local_locations {
        let runs = read_replay_runs(&path)?;
        append_indexed_replay_runs(
            &mut replay_runs,
            runs,
            &expected_ids,
            &path.display().to_string(),
        )?;
    }
    for ((bucket, key), expected_ids) in s3_locations {
        let runs = read_replay_runs_from_s3(&bucket, std::slice::from_ref(&key)).await?;
        append_indexed_replay_runs(
            &mut replay_runs,
            runs,
            &expected_ids,
            &format!("s3://{bucket}/{key}"),
        )?;
    }
    Ok(replay_runs)
}

fn append_unique_bundles(
    target: &mut Vec<IntelCandidateEvidenceBundle>,
    bundles: Vec<IntelCandidateEvidenceBundle>,
) {
    let mut existing_ids = target
        .iter()
        .map(|bundle| bundle.candidate_id.clone())
        .collect::<BTreeSet<_>>();
    for bundle in bundles {
        if existing_ids.insert(bundle.candidate_id.clone()) {
            target.push(bundle);
        }
    }
}

fn append_unique_replay_runs(target: &mut Vec<ReplayRun>, runs: Vec<ReplayRun>) {
    let mut existing_ids = target
        .iter()
        .map(|run| run.replay_run_id.clone())
        .collect::<BTreeSet<_>>();
    for run in runs {
        if existing_ids.insert(run.replay_run_id.clone()) {
            target.push(run);
        }
    }
}

fn filter_historical_replay_runs_for_current_research(
    runs: Vec<ReplayRun>,
    current_replay_runs: &[ReplayRun],
) -> Vec<ReplayRun> {
    let current_aggregate_keys = current_replay_runs
        .iter()
        .map(|run| run.research_aggregate_key.as_str())
        .collect::<BTreeSet<_>>();
    runs.into_iter()
        .filter(|run| current_aggregate_keys.contains(run.research_aggregate_key.as_str()))
        .collect()
}

fn append_unique_oss_adapter_runs(target: &mut Vec<OssAdapterRun>, runs: Vec<OssAdapterRun>) {
    let mut existing_ids = target
        .iter()
        .map(|run| run.oss_adapter_run_id.clone())
        .collect::<BTreeSet<_>>();
    for run in runs {
        if existing_ids.insert(run.oss_adapter_run_id.clone()) {
            target.push(run);
        }
    }
}

fn append_unique_shadow_validation_runs(
    target: &mut Vec<ShadowValidationRun>,
    runs: Vec<ShadowValidationRun>,
) {
    let mut existing_ids = target
        .iter()
        .map(|run| run.shadow_validation_run_id.clone())
        .collect::<BTreeSet<_>>();
    for run in runs {
        if existing_ids.insert(run.shadow_validation_run_id.clone()) {
            target.push(run);
        }
    }
}

fn append_indexed_replay_runs(
    target: &mut Vec<ReplayRun>,
    runs: Vec<ReplayRun>,
    expected_ids: &BTreeSet<String>,
    label: &str,
) -> AppResult<()> {
    let mut matched_ids = BTreeSet::new();
    let selected_runs = runs
        .into_iter()
        .filter(|run| {
            let matched = expected_ids.contains(&run.replay_run_id);
            if matched {
                matched_ids.insert(run.replay_run_id.clone());
            }
            matched
        })
        .collect::<Vec<_>>();

    let missing_ids = expected_ids
        .difference(&matched_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_ids.is_empty() {
        return Err(AppError::validation(format!(
            "replay_run_index points to missing replay_run_id(s) in {label}: {}",
            missing_ids.join(",")
        )));
    }

    append_unique_replay_runs(target, selected_runs);
    Ok(())
}

fn parse_s3_uri(value: &str) -> Option<(String, String)> {
    let rest = value.strip_prefix("s3://")?;
    let (bucket, key) = rest.split_once('/')?;
    if bucket.trim().is_empty() || key.trim().is_empty() {
        return None;
    }
    Some((bucket.to_owned(), key.to_owned()))
}

fn should_read_market_s3(args: &Args) -> bool {
    args.input_bundle_s3_bucket.is_some()
        || args.market_l1_s3_bucket.is_some()
        || !args.market_feature_delta_s3_keys.is_empty()
        || !args.market_regime_context_s3_keys.is_empty()
}

fn bundle_symbol_filter(bundles: &[IntelCandidateEvidenceBundle]) -> BTreeSet<String> {
    bundles
        .iter()
        .flat_map(|bundle| bundle.normalized_symbols.iter().cloned())
        .collect()
}

fn market_l1_s3_bucket(args: &Args) -> &str {
    args.market_l1_s3_bucket
        .as_deref()
        .unwrap_or(DEFAULT_MARKET_L1_S3_BUCKET)
}

async fn market_feature_delta_s3_keys(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
) -> AppResult<Vec<String>> {
    market_s3_keys(args, bundles, MarketArtifactFamily::FeatureDelta).await
}

async fn market_regime_context_s3_keys(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
) -> AppResult<Vec<String>> {
    market_s3_keys(args, bundles, MarketArtifactFamily::RegimeContext).await
}

async fn market_s3_keys(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
    family: MarketArtifactFamily,
) -> AppResult<Vec<String>> {
    let mut keys = BTreeSet::new();
    for key in family.manual_keys(args) {
        insert_normalized_s3_key(&mut keys, key);
    }
    for bundle in bundles {
        for artifact in &bundle.selected_market_artifacts {
            if let Some(key) = family.key_from_selected_artifact(artifact) {
                insert_normalized_s3_key(&mut keys, &key);
            }
        }
        if let Some(key) = bundle
            .data_quality_summary
            .market_data_quality_summary_key
            .as_deref()
            && let Some(run_id) = market_l1_run_id_from_key(key)
        {
            keys.insert(family.key_from_run_id(&run_id));
        }
    }
    keys.extend(family.discover_keys(args, bundles).await?);
    Ok(keys.into_iter().collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarketArtifactFamily {
    FeatureDelta,
    RegimeContext,
}

impl MarketArtifactFamily {
    fn manual_keys(self, args: &Args) -> &[String] {
        match self {
            Self::FeatureDelta => &args.market_feature_delta_s3_keys,
            Self::RegimeContext => &args.market_regime_context_s3_keys,
        }
    }

    fn key_from_selected_artifact(self, artifact: &SelectedMarketArtifactTrace) -> Option<String> {
        match self {
            Self::FeatureDelta => feature_delta_key_from_selected_artifact(artifact),
            Self::RegimeContext => (artifact.artifact_type == MARKET_REGIME_CONTEXT_ARTIFACT_TYPE)
                .then(|| artifact.artifact_key.clone())
                .flatten(),
        }
    }

    fn key_from_run_id(self, run_id: &str) -> String {
        match self {
            Self::FeatureDelta => format!("market_feature_delta/run_id={run_id}/delta.json"),
            Self::RegimeContext => {
                format!("market_regime_context/run_id={run_id}/context.json")
            }
        }
    }

    async fn discover_keys(
        self,
        args: &Args,
        bundles: &[IntelCandidateEvidenceBundle],
    ) -> AppResult<Vec<String>> {
        let starts = market_l1_replay_window_starts(bundles, args.now_ms.unwrap_or_else(now_ms));
        let discovered = match self {
            Self::FeatureDelta => {
                discover_latest_market_feature_delta_keys_from_s3(
                    market_l1_s3_bucket(args),
                    &starts,
                )
                .await?
            }
            Self::RegimeContext => {
                discover_latest_market_regime_context_keys_from_s3(
                    market_l1_s3_bucket(args),
                    &starts,
                )
                .await?
            }
        };
        Ok(discovered
            .into_iter()
            .filter_map(|key| normalize_s3_key(&key))
            .collect())
    }
}

fn feature_delta_key_from_selected_artifact(
    artifact: &SelectedMarketArtifactTrace,
) -> Option<String> {
    if artifact.artifact_type == MARKET_FEATURE_DELTA_ARTIFACT_TYPE {
        return artifact.artifact_key.clone();
    }
    if artifact.artifact_type != MARKET_FEATURE_DELTA_SUMMARY_ARTIFACT_TYPE {
        return None;
    }
    artifact
        .l1_run_id
        .clone()
        .or_else(|| {
            artifact
                .artifact_key
                .as_deref()
                .and_then(market_l1_run_id_from_key)
        })
        .map(|run_id| format!("market_feature_delta/run_id={run_id}/delta.json"))
}

fn insert_normalized_s3_key(keys: &mut BTreeSet<String>, value: &str) {
    if let Some(key) = normalize_s3_key(value) {
        keys.insert(key);
    }
}

fn normalize_s3_key(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(uri_without_scheme) = trimmed.strip_prefix("s3://") {
        let (_, key) = uri_without_scheme.split_once('/')?;
        let key = key.trim_start_matches('/').trim();
        return (!key.is_empty()).then(|| key.to_owned());
    }
    Some(trimmed.to_owned())
}

fn market_l1_run_id_from_key(value: &str) -> Option<String> {
    let key = normalize_s3_key(value)?;
    let marker = "run_id=";
    let start = key.find(marker)? + marker.len();
    let remainder = &key[start..];
    let end = remainder.find('/').unwrap_or(remainder.len());
    let run_id = remainder[..end].trim();
    (!run_id.is_empty()).then(|| run_id.to_owned())
}

fn market_l1_replay_window_starts(
    bundles: &[IntelCandidateEvidenceBundle],
    discovery_cutoff_ms: i64,
) -> Vec<i64> {
    let mut starts = BTreeSet::new();
    for bundle in bundles {
        if !validate_bundle_admission(bundle).admitted {
            continue;
        }
        let Some(max_horizon_ms) = bundle
            .allowed_horizons
            .iter()
            .filter_map(|horizon| horizon_ms(horizon))
            .max()
        else {
            continue;
        };
        let replay_start_ms = bundle.forbidden_lookahead_boundary_ms;
        let replay_end_ms = (replay_start_ms + max_horizon_ms).min(discovery_cutoff_ms);
        if replay_end_ms < replay_start_ms {
            continue;
        }
        let mut window_start_ms = align_market_l1_window_start(replay_start_ms);
        let last_window_start_ms = align_market_l1_window_start(replay_end_ms);
        while window_start_ms <= last_window_start_ms {
            starts.insert(window_start_ms);
            window_start_ms += MARKET_L1_REPLAY_WINDOW_MS;
        }
    }
    starts.into_iter().collect()
}

fn align_market_l1_window_start(timestamp_ms: i64) -> i64 {
    timestamp_ms.div_euclid(MARKET_L1_REPLAY_WINDOW_MS) * MARKET_L1_REPLAY_WINDOW_MS
}

fn deterministic_report_created_at_ms(bundles: &[IntelCandidateEvidenceBundle]) -> i64 {
    bundles
        .iter()
        .map(|bundle| {
            bundle
                .created_at_ms
                .max(bundle.candidate_created_at_ms)
                .max(bundle.decision_available_at_ms)
                .max(bundle.forbidden_lookahead_boundary_ms)
        })
        .max()
        .unwrap_or_else(now_ms)
}

fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(AppError::config(format!(
            "{message}; got {}",
            path.display()
        )));
    }
    Ok(path)
}

fn has_retest_horizon_status_input(args: &Args) -> bool {
    args.retest_horizon_status_file.is_some() || args.retest_horizon_status_s3_key.is_some()
}

fn has_retest_horizon_plan_input(args: &Args) -> bool {
    args.retest_horizon_plan_file.is_some() || args.retest_horizon_plan_s3_key.is_some()
}

fn has_research_report_input(args: &Args) -> bool {
    args.research_report_file.is_some() || args.research_report_s3_key.is_some()
}

fn validate_paper_watch_live_cycle_args(args: &Args) -> AppResult<()> {
    validate_paper_watch_live_cycle_mode_is_isolated(args)?;
    validate_paper_watch_candidate_input(args)?;
    validate_market_live_tick_input(args)?;
    validate_paper_watch_live_cycle_output(args)?;
    Ok(())
}

fn validate_paper_watch_live_cycle_mode_is_isolated(args: &Args) -> AppResult<()> {
    let has_other_mode = [
        args.build_shadow_cycle_decision,
        args.run_shadow_cycle_from_latest_state,
        args.build_retest_horizon_plan,
        args.run_retest_refresh_cycle,
        args.run_retest_refresh_cycle_from_latest_state,
        args.run_retest_cycle_scheduler,
        args.build_retest_horizon_status,
        args.build_focused_retest_manifest,
    ]
    .into_iter()
    .any(|enabled| enabled);
    if has_other_mode {
        return Err(AppError::config(
            "use --run-paper-watch-live-cycle separately from research/retest/shadow modes",
        ));
    }
    Ok(())
}

fn validate_paper_watch_candidate_input(args: &Args) -> AppResult<()> {
    if args.paper_watch_candidate_file.is_some()
        && (args.paper_watch_candidate_s3_bucket.is_some()
            || args.paper_watch_candidate_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --paper-watch-candidate-file or --paper-watch-candidate-s3-bucket/--paper-watch-candidate-s3-key, not both",
        ));
    }
    if args.paper_watch_candidate_s3_bucket.is_some() != args.paper_watch_candidate_s3_key.is_some()
    {
        return Err(AppError::config(
            "--paper-watch-candidate-s3-bucket and --paper-watch-candidate-s3-key must be set together",
        ));
    }
    if args.paper_watch_candidate_file.is_none() && args.paper_watch_candidate_s3_key.is_none() {
        return Err(AppError::config(
            "--run-paper-watch-live-cycle requires paper watch candidate input",
        ));
    }
    if let Some(path) = args.paper_watch_candidate_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_PAPER_WATCH_CANDIDATE_FILE must be an absolute path",
        ));
    }
    Ok(())
}

fn validate_market_live_tick_input(args: &Args) -> AppResult<()> {
    if let Some(path) = args.market_live_tick_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_MARKET_LIVE_TICK_FILE must be an absolute path",
        ));
    }
    if args.market_live_tick_file.is_some() && args.market_live_nats_url.is_some() {
        return Err(AppError::config(
            "use either --market-live-tick-file or --market-live-nats-url, not both",
        ));
    }
    if args.market_live_tick_file.is_none() && args.market_live_nats_url.is_none() {
        return Err(AppError::config(
            "--run-paper-watch-live-cycle requires market live tick file or NATS url",
        ));
    }
    if let Some(url) = args.market_live_nats_url.as_deref()
        && !url.starts_with("nats://")
    {
        return Err(AppError::config(
            "--market-live-nats-url must start with nats://",
        ));
    }
    Ok(())
}

fn validate_paper_watch_live_cycle_output(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() && args.output_s3_bucket.is_some() {
        return Err(AppError::config(
            "use either --output-dir or --output-s3-bucket, not both",
        ));
    }
    Ok(())
}

fn validate_retest_horizon_plan_input_args(args: &Args) -> AppResult<()> {
    if args.retest_horizon_plan_file.is_some()
        && (args.retest_horizon_plan_s3_bucket.is_some()
            || args.retest_horizon_plan_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --retest-horizon-plan-file or --retest-horizon-plan-s3-bucket/--retest-horizon-plan-s3-key, not both",
        ));
    }
    if args.retest_horizon_plan_s3_bucket.is_some() != args.retest_horizon_plan_s3_key.is_some() {
        return Err(AppError::config(
            "--retest-horizon-plan-s3-bucket and --retest-horizon-plan-s3-key must be set together",
        ));
    }
    if let Some(path) = args.retest_horizon_plan_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_RETEST_HORIZON_PLAN_FILE must be an absolute path",
        ));
    }
    if let Some(path) = args.retest_driver_summary_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_RETEST_DRIVER_SUMMARY_FILE must be an absolute path",
        ));
    }
    Ok(())
}

fn validate_research_report_input_args(args: &Args) -> AppResult<()> {
    if args.research_report_file.is_some()
        && (args.research_report_s3_bucket.is_some() || args.research_report_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --research-report-file or --research-report-s3-bucket/--research-report-s3-key, not both",
        ));
    }
    if args.research_report_s3_bucket.is_some() != args.research_report_s3_key.is_some() {
        return Err(AppError::config(
            "--research-report-s3-bucket and --research-report-s3-key must be set together",
        ));
    }
    if let Some(path) = args.research_report_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_REPORT_FILE must be an absolute path",
        ));
    }
    Ok(())
}

fn validate_retest_horizon_status_input_args(args: &Args) -> AppResult<()> {
    if args.retest_horizon_status_file.is_some()
        && (args.retest_horizon_status_s3_bucket.is_some()
            || args.retest_horizon_status_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --retest-horizon-status-file or --retest-horizon-status-s3-bucket/--retest-horizon-status-s3-key, not both",
        ));
    }
    if args.retest_horizon_status_s3_bucket.is_some() != args.retest_horizon_status_s3_key.is_some()
    {
        return Err(AppError::config(
            "--retest-horizon-status-s3-bucket and --retest-horizon-status-s3-key must be set together",
        ));
    }
    if let Some(path) = args.retest_horizon_status_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_HORIZON_STATUS_FILE must be an absolute path",
        ));
    }
    Ok(())
}

fn validate_retest_horizon_plan_build_args(args: &Args) -> AppResult<()> {
    if args.input_manifest_file.is_some()
        && (args.input_manifest_s3_bucket.is_some() || args.input_manifest_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        ));
    }
    if args.input_manifest_s3_bucket.is_some() != args.input_manifest_s3_key.is_some() {
        return Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        ));
    }
    if args.input_manifest_file.is_none() && args.input_manifest_s3_key.is_none() {
        return Err(AppError::config(
            "--build-retest-horizon-plan requires --input-manifest-file or S3 manifest input",
        ));
    }
    if !has_research_report_input(args) {
        return Err(AppError::config(
            "--build-retest-horizon-plan requires --research-report-file or S3 report input",
        ));
    }
    if has_retest_horizon_plan_input(args) {
        return Err(AppError::config(
            "--build-retest-horizon-plan creates a plan; do not also pass retest horizon plan input",
        ));
    }
    if args.output_s3_bucket.is_some() && args.retest_horizon_plan_output_file.is_some() {
        return Err(AppError::config(
            "use either --retest-horizon-plan-output-file or --output-s3-bucket, not both",
        ));
    }
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--build-retest-horizon-plan uses --retest-horizon-plan-output-file or --output-s3-bucket, not --output-dir",
        ));
    }
    if args.retest_horizon_plan_output_file.is_none() && args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--build-retest-horizon-plan requires --retest-horizon-plan-output-file or --output-s3-bucket",
        ));
    }
    Ok(())
}

fn validate_retest_refresh_cycle_args(args: &Args) -> AppResult<()> {
    validate_retest_refresh_manifest_input(args)?;
    if !has_research_report_input(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle requires --research-report-file or S3 report input",
        ));
    }
    if has_retest_horizon_plan_input(args) || has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle creates fresh plan/status; do not pass retest horizon plan/status inputs",
        ));
    }
    if has_retest_refresh_individual_output(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle uses --output-dir or --output-s3-bucket, not individual retest/focus output files",
        ));
    }
    validate_retest_refresh_output_target(args)?;
    if args.output_s3_prefix.is_some() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle writes multiple artifact families; do not pass --output-s3-prefix",
        ));
    }
    if args.focused_retest_next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    Ok(())
}

fn validate_retest_refresh_cycle_from_latest_state_args(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state uses --output-s3-bucket, not --output-dir",
        ));
    }
    if args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --output-s3-bucket",
        ));
    }
    if args.market_l1_s3_bucket.is_none() && args.retest_horizon_latest_l1_as_of_ms.is_none() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --market-l1-s3-bucket or --retest-horizon-latest-l1-as-of-ms",
        ));
    }
    if args.input_manifest_file.is_some()
        || args.input_manifest_s3_bucket.is_some()
        || args.input_manifest_s3_key.is_some()
        || args.research_report_file.is_some()
        || args.research_report_s3_bucket.is_some()
        || args.research_report_s3_key.is_some()
    {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state discovers manifest/report inputs from retest-cycle-source-state; do not pass manifest/report inputs",
        ));
    }
    if has_retest_horizon_plan_input(args) || has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state creates fresh plan/status; do not pass retest horizon plan/status inputs",
        ));
    }
    if has_retest_refresh_individual_output(args) {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state uses --output-s3-bucket, not individual retest/focus output files",
        ));
    }
    if args.output_s3_prefix.is_some() {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state writes multiple artifact families; do not pass --output-s3-prefix",
        ));
    }
    if args.focused_retest_next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    Ok(())
}

fn validate_retest_refresh_manifest_input(args: &Args) -> AppResult<()> {
    match (
        args.input_manifest_file.is_some(),
        args.input_manifest_s3_bucket.is_some(),
        args.input_manifest_s3_key.is_some(),
    ) {
        (true, false, false) | (false, true, true) => Ok(()),
        (true, _, _) => Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        )),
        (false, true, false) | (false, false, true) => Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        )),
        (false, false, false) => Err(AppError::config(
            "--run-retest-refresh-cycle requires --input-manifest-file or S3 manifest input",
        )),
    }
}

fn has_retest_refresh_individual_output(args: &Args) -> bool {
    [
        args.retest_horizon_plan_output_file.is_some(),
        args.retest_horizon_status_output_file.is_some(),
        args.focused_retest_manifest_output_file.is_some(),
        args.focused_retest_summary_output_file.is_some(),
        args.retest_driver_summary_file.is_some(),
    ]
    .into_iter()
    .any(|present| present)
}

fn validate_retest_refresh_output_target(args: &Args) -> AppResult<()> {
    match (args.output_dir.is_some(), args.output_s3_bucket.is_some()) {
        (true, false) | (false, true) => Ok(()),
        (true, true) => Err(AppError::config(
            "use either --output-dir or --output-s3-bucket, not both",
        )),
        (false, false) => Err(AppError::config(
            "--run-retest-refresh-cycle requires --output-dir or --output-s3-bucket",
        )),
    }
}

fn validate_focused_retest_manifest_build_args(args: &Args) -> AppResult<()> {
    if !has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires a retest horizon status input",
        ));
    }
    if args.input_manifest_file.is_some()
        && (args.input_manifest_s3_bucket.is_some() || args.input_manifest_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        ));
    }
    if args.input_manifest_s3_bucket.is_some() != args.input_manifest_s3_key.is_some() {
        return Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        ));
    }
    if args.input_manifest_file.is_none() && args.input_manifest_s3_key.is_none() {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires --input-manifest-file or S3 manifest input",
        ));
    }
    if args.output_s3_bucket.is_some() && args.focused_retest_manifest_output_file.is_some() {
        return Err(AppError::config(
            "use either --focused-retest-manifest-output-file or --output-s3-bucket, not both",
        ));
    }
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--build-focused-retest-manifest uses --focused-retest-manifest-output-file or --output-s3-bucket, not --output-dir",
        ));
    }
    if args.focused_retest_manifest_output_file.is_none() && args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires --focused-retest-manifest-output-file or --output-s3-bucket",
        ));
    }
    if args.focused_retest_next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    Ok(())
}

fn validate_retest_horizon_status_build_args(args: &Args) -> AppResult<()> {
    if !has_retest_horizon_plan_input(args) {
        return Err(AppError::config(
            "--build-retest-horizon-status requires a retest horizon plan input",
        ));
    }
    if has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--build-retest-horizon-status creates a status; do not also pass retest horizon status input",
        ));
    }
    if args.output_s3_bucket.is_some() && args.retest_horizon_status_output_file.is_some() {
        return Err(AppError::config(
            "use either --retest-horizon-status-output-file or --output-s3-bucket, not both",
        ));
    }
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--build-retest-horizon-status uses --retest-horizon-status-output-file or --output-s3-bucket, not --output-dir",
        ));
    }
    if args.retest_horizon_status_output_file.is_none() && args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--build-retest-horizon-status requires --retest-horizon-status-output-file or --output-s3-bucket",
        ));
    }
    Ok(())
}

fn validate_retest_cycle_scheduler_args(args: &Args) -> AppResult<()> {
    if !has_retest_horizon_status_input(args) {
        return Err(AppError::config(
            "--run-retest-cycle-scheduler requires a retest horizon status input",
        ));
    }
    validate_focused_retest_manifest_build_args(args).map_err(|error| {
        AppError::config(format!(
            "--run-retest-cycle-scheduler uses focused retest manifest inputs when execution is due: {error}"
        ))
    })
}

fn validate_shadow_cycle_build_args(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() && args.output_s3_bucket.is_some() {
        return Err(AppError::config(
            "use either --output-dir or --output-s3-bucket, not both",
        ));
    }
    if let Some(path) = args.shadow_cycle_decision_output_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_SHADOW_CYCLE_DECISION_OUTPUT_FILE must be an absolute path",
        ));
    }
    if args.shadow_cycle_decision_output_file.is_none()
        && args.output_dir.is_none()
        && args.output_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--build-shadow-cycle-decision requires --shadow-cycle-decision-output-file, --output-dir, or --output-s3-bucket",
        ));
    }
    if !args.shadow_validation_run_s3_keys.is_empty()
        && args.shadow_validation_run_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--shadow-validation-run-s3-bucket is required when --shadow-validation-run-s3-key is set",
        ));
    }
    if args.shadow_validation_run_files.is_empty()
        && args.shadow_validation_run_s3_keys.is_empty()
        && args.input_manifest_file.is_none()
        && args.input_manifest_s3_key.is_none()
    {
        return Err(AppError::config(
            "--build-shadow-cycle-decision requires a shadow validation run file, shadow validation S3 key, or manifest with shadow_validation_run_refs",
        ));
    }
    Ok(())
}

fn validate_shadow_cycle_from_latest_state_args(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state uses --output-s3-bucket, not --output-dir",
        ));
    }
    if args.shadow_cycle_decision_output_file.is_some() {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state uses --output-s3-bucket, not --shadow-cycle-decision-output-file",
        ));
    }
    if args.output_s3_bucket.is_none() {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state requires --output-s3-bucket",
        ));
    }
    if args.input_manifest_file.is_some() || args.input_manifest_s3_key.is_some() {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state discovers shadow inputs from S3; do not pass manifest input",
        ));
    }
    if !args.shadow_validation_run_files.is_empty()
        || !args.shadow_validation_run_s3_keys.is_empty()
    {
        return Err(AppError::config(
            "--run-shadow-cycle-from-latest-state discovers shadow inputs from S3; do not pass explicit shadow validation inputs",
        ));
    }
    Ok(())
}

fn non_empty_arg(value: Option<String>, message: &str) -> AppResult<String> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    if value.trim().is_empty() {
        return Err(AppError::config(message));
    }
    Ok(value)
}

fn env_string(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn env_bool(name: &str) -> bool {
    env_string(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "y"
            )
        })
        .unwrap_or(false)
}

fn env_non_negative_i64(name: &str) -> AppResult<Option<i64>> {
    let Some(raw) = env_string(name) else {
        return Ok(None);
    };
    parse_non_negative_i64(name, &raw).map(Some)
}

fn parse_non_negative_i64(name: &str, raw: &str) -> AppResult<i64> {
    let value = raw
        .parse::<i64>()
        .map_err(|_| AppError::config(format!("{name} must be an integer")))?;
    if value < 0 {
        return Err(AppError::config(format!("{name} must be non-negative")));
    }
    Ok(value)
}

fn env_usize(name: &str, fallback: usize) -> AppResult<usize> {
    let Some(raw) = env_string(name) else {
        return Ok(fallback);
    };
    parse_positive_usize(name, &raw)
}

fn env_u64(name: &str, fallback: u64) -> AppResult<u64> {
    let Some(raw) = env_string(name) else {
        return Ok(fallback);
    };
    parse_positive_u64(name, &raw)
}

fn parse_positive_usize(name: &str, raw: &str) -> AppResult<usize> {
    let value = raw
        .parse::<usize>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if value == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(value)
}

fn parse_positive_u64(name: &str, raw: &str) -> AppResult<u64> {
    let value = raw
        .parse::<u64>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if value == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(value)
}

fn env_list(name: &str) -> Vec<String> {
    env::var(name)
        .ok()
        .map(|value| {
            value
                .split([',', '\n'])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn help_text() -> &'static str {
    r#"research-app
Usage:
  research-app \
    --input-bundle-file /Volumes/WD/Developments/nangman-crypto/data/examples/candidate-bundles.jsonl \
    --market-feature-delta-file /Volumes/WD/Developments/nangman-crypto/data/examples/market-feature-delta.json \
    --market-regime-context-file /Volumes/WD/Developments/nangman-crypto/data/examples/market-regime-context.json \
    --output-dir /Volumes/WD/Developments/nangman-crypto/data/reports/research-local

Batch research input can be declared with a manifest:
  --input-manifest-file /Volumes/WD/Developments/nangman-crypto/data/examples/research-input-manifest.json
  --input-manifest-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --input-manifest-s3-key research-input-manifest/schema=research_input_manifest_v1/run_id=.../manifest.json

Shadow cycle scheduler decisions can be validated without running research:
  --shadow-cycle-decision-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/shadow-cycle-decision.json

Retest horizon scheduler handoff can be validated without running research:
  --retest-horizon-status-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json

Retest horizon plans can be built inside the app from a research input manifest,
candidate bundle refs, a research-run report, and an optional/latest Market-L1
watermark. S3 output uses the non-dispatching retest-horizon-plan/ prefix:
  --build-retest-horizon-plan
  --input-manifest-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-input-manifest.json
  --research-report-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-report.json
  --retest-horizon-latest-l1-as-of-ms 1779715800000
  --retest-horizon-plan-output-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-plan.json

Retest horizon status can be built inside the app from an existing retest
horizon plan, removing the local shell/JQ status summarizer from scheduler paths:
  --build-retest-horizon-status
  --retest-horizon-plan-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-plan.json
  --retest-horizon-status-output-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json

Retest refresh cycle mode rebuilds the plan/status from current manifest,
research report, and latest Market-L1 watermark. It writes no dispatching
manifest while the status is WAIT; when replay is ready it writes a focused
research_input_manifest_v1 under the existing dispatcher prefix:
  --run-retest-refresh-cycle
  --input-manifest-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --input-manifest-s3-key research-input-manifest/schema=research_input_manifest_v1/...
  --research-report-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --research-report-s3-key research-run-report/schema=research_run_report_v1/.../report.json
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix>
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix>

Retest refresh can also discover its manifest/report pair from the latest
retest-cycle-source-state checkpoint, which is the scheduler-friendly runtime
entrypoint after prior S3 research output has completed:
  --run-retest-refresh-cycle-from-latest-state
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix>

Retest cycle scheduler mode is safe to call repeatedly. It does not write a
manifest before run_not_before_ms, and if an old WAIT status is past due it asks
for a fresh retest horizon status instead of triggering stale research:
  --run-retest-cycle-scheduler
  --retest-horizon-status-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json
  --input-manifest-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-input-manifest.json
  --focused-retest-manifest-output-file /tmp/nangman-crypto/research-focus/input-manifest.json

Focused retest manifests can be built without running research:
  --build-focused-retest-manifest
  --retest-horizon-status-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json
  --input-manifest-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-input-manifest.json
  --focused-retest-manifest-output-file /tmp/nangman-crypto/research-focus/input-manifest.json

Paper-watch live cycle can mark paper-only candidates from the MARKET_LIVE stream.
It does not create orders, does not approve live trading, and always writes
paper_watch_live_mark_v1 with live/order safety flags disabled:
  --run-paper-watch-live-cycle
  --paper-watch-candidate-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --paper-watch-candidate-s3-key paper-watch-candidate/schema=paper_watch_candidate_v1/...
  --market-live-nats-url nats://<private-nats-host>:4222
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --output-s3-prefix paper-watch-live-mark/schema=paper_watch_live_mark_v1

The focused manifest can also be written to S3 to wake the existing dispatcher:
  --build-focused-retest-manifest
  --retest-horizon-status-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --retest-horizon-status-s3-key retest-horizon-status/schema=research_horizon_status_checkpoint_v1/...
  --input-manifest-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --input-manifest-s3-key research-input-manifest/schema=research_input_manifest_v1/...
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --output-s3-prefix research-input-manifest/schema=research_input_manifest_v1

Shadow cycle scheduler decisions can also be built inside the app from
shadow_validation_run_v1 inputs:
  --build-shadow-cycle-decision
  --shadow-validation-run-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/shadow-validation-run.jsonl
  --shadow-cycle-latest-l1-as-of-ms 1779696900000
  --shadow-cycle-decision-output-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/shadow-cycle-decision.json

The same mode can run in ECS with S3 input/output:
  --build-shadow-cycle-decision
  --shadow-validation-run-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --shadow-validation-run-s3-key shadow-validation-run/schema=shadow_validation_run_v1/dt=.../part-000001.jsonl
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --output-s3-prefix shadow-cycle/

The ECS scheduler can also discover the latest shadow-validation-run artifacts
from the research bucket and write a fresh shadow-cycle-decision artifact:
  --run-shadow-cycle-from-latest-state
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix>

Market L1 replay input can be loaded from S3 in ECS:
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix>
  --market-feature-delta-s3-key market_feature_delta/run_id=l1_.../delta.json
  --market-regime-context-s3-key market_regime_context/run_id=l1_.../context.json
  The app can also discover later replay-window keys through direct run_id
  prefixes or the success-only l1_index -> manifest path.
  --historical-replay-run-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --historical-replay-run-s3-key replay-run/schema=replay_run_v1/dt=.../part-000001.jsonl
  --historical-replay-run-index-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --historical-replay-run-index-s3-key replay-run-index/schema=replay_run_index_v1/dt=.../part-000001.jsonl
  --shadow-validation-run-s3-bucket nangman-crypto-dev-research-<account-suffix>
  --shadow-validation-run-s3-key shadow-validation-run/schema=shadow_validation_run_v1/dt=.../part-000001.jsonl

ECS input and output can also come from environment:
  RESEARCH_INPUT_MANIFEST_S3_BUCKET
  RESEARCH_INPUT_MANIFEST_S3_KEY
  RESEARCH_INPUT_S3_BUCKET
  RESEARCH_INPUT_S3_KEY
  RESEARCH_MARKET_L1_S3_BUCKET
  RESEARCH_MARKET_FEATURE_DELTA_S3_KEYS
  RESEARCH_MARKET_REGIME_CONTEXT_S3_KEYS
  RESEARCH_HISTORICAL_REPLAY_RUN_S3_BUCKET
  RESEARCH_HISTORICAL_REPLAY_RUN_S3_KEYS
  RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET
  RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_KEYS
  RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX
  RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_READ_LIMIT
  RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_SCAN_LIMIT
  RESEARCH_SHADOW_VALIDATION_RUN_S3_BUCKET
  RESEARCH_SHADOW_VALIDATION_RUN_S3_KEYS
  RESEARCH_BUILD_SHADOW_CYCLE_DECISION
  RESEARCH_SHADOW_CYCLE_LATEST_L1_AS_OF_MS
	  RESEARCH_SHADOW_CYCLE_DECISION_OUTPUT_FILE
	  RESEARCH_RUN_PAPER_WATCH_LIVE_CYCLE
	  RESEARCH_PAPER_WATCH_CANDIDATE_FILE
	  RESEARCH_PAPER_WATCH_CANDIDATE_S3_BUCKET
	  RESEARCH_PAPER_WATCH_CANDIDATE_S3_KEY
	  RESEARCH_MARKET_LIVE_TICK_FILE
	  RESEARCH_MARKET_LIVE_NATS_URL
	  RESEARCH_MARKET_LIVE_NATS_STREAM
	  RESEARCH_MARKET_LIVE_NATS_SUBJECT
	  RESEARCH_MARKET_LIVE_NATS_CONSUMER
	  RESEARCH_MARKET_LIVE_NATS_DELIVER_POLICY
	  RESEARCH_MARKET_LIVE_NATS_BATCH_SIZE
	  RESEARCH_MARKET_LIVE_NATS_MAX_MESSAGES
	  RESEARCH_MARKET_LIVE_NATS_ACK_WAIT_SECS
	  RESEARCH_OUTPUT_S3_BUCKET
	  RESEARCH_OUTPUT_S3_PREFIX

Without --output-dir, the app prints research_run_report_v1 to stdout.
This app does not execute orders, does not approve live trading, and does not emit EXECUTION_APPROVED or LIVE_READY."#
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
