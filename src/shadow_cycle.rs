use crate::error::{AppError, AppResult};
use crate::hash::stable_id;
use crate::model::{
    SHADOW_CYCLE_DECISION_SCHEMA_VERSION, ShadowCycleDecision, ShadowCycleDecisionSafety,
    ShadowCycleSampleState, ShadowCycleSchedulerAction, ShadowValidationRun,
    ShadowValidationStatus,
};
use chrono::{DateTime, SecondsFormat, Utc};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const MS_PER_HOUR: i64 = 60 * 60 * 1000;

#[derive(Debug, Clone)]
struct CandidateShadowState {
    symbol_set: BTreeSet<String>,
    observed_count: usize,
    target_materialized_count: usize,
    pending_target_count: usize,
    pending_count: usize,
    required_count: usize,
    next_pending_target_deadline_ms: Option<i64>,
}

impl CandidateShadowState {
    fn new() -> Self {
        Self {
            symbol_set: BTreeSet::new(),
            observed_count: 0,
            target_materialized_count: 0,
            pending_target_count: 0,
            pending_count: 0,
            required_count: 0,
            next_pending_target_deadline_ms: None,
        }
    }

    fn sample_deficit(&self) -> i64 {
        self.required_count
            .saturating_sub(self.target_materialized_count) as i64
    }

    fn sample_requirement_met(&self) -> bool {
        self.required_count > 0 && self.target_materialized_count >= self.required_count
    }
}

#[derive(Debug)]
struct ShadowCycleBuildSummary {
    candidates: BTreeMap<String, CandidateShadowState>,
    symbols: BTreeSet<String>,
    target_materialized_count: usize,
    run_identity_parts: Vec<String>,
}

pub fn build_shadow_cycle_decision(
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
    generated_at_ms: i64,
) -> ShadowCycleDecision {
    let mut summary = summarize_shadow_runs(shadow_runs, latest_l1_as_of_ms);
    summary.run_identity_parts.sort_unstable();
    let candidate_lifecycle_count = summary.candidates.len();
    let target_waiting_count = summary
        .candidates
        .values()
        .filter(|state| state.target_materialized_count == 0 && state.observed_count > 0)
        .count();
    let partially_materialized_count = summary
        .candidates
        .values()
        .filter(|state| {
            state.target_materialized_count > 0
                && state.target_materialized_count < state.observed_count
        })
        .count();
    let pending_target_window_candidate_count = summary
        .candidates
        .values()
        .filter(|state| state.pending_target_count > 0)
        .count();
    let sample_ready_count = summary
        .candidates
        .values()
        .filter(|state| state.sample_requirement_met())
        .count();
    let deficient_count = summary
        .candidates
        .values()
        .filter(|state| state.sample_deficit() > 0)
        .count();
    let pending_count = summary
        .candidates
        .values()
        .filter(|state| state.pending_count > 0)
        .count();
    let total_sample_deficit = summary
        .candidates
        .values()
        .map(CandidateShadowState::sample_deficit)
        .sum();
    let next_observation_not_before_ms =
        summary.candidates.values().fold(None, |current, state| {
            min_optional_ms(current, state.next_pending_target_deadline_ms)
        });

    let (source_verdict, scheduler_action) = select_scheduler_action(
        shadow_runs.is_empty(),
        latest_l1_as_of_ms.is_some(),
        target_waiting_count,
        partially_materialized_count,
        deficient_count,
        pending_count,
        sample_ready_count,
    );

    let run_not_before_ms = scheduler_action
        .is_wait_action()
        .then_some(next_observation_not_before_ms)
        .flatten();

    ShadowCycleDecision {
        schema_version: SHADOW_CYCLE_DECISION_SCHEMA_VERSION.to_owned(),
        generated_at: iso8601_ms(generated_at_ms),
        decision_id: stable_id(
            "shadow_cycle_decision",
            &[
                source_verdict,
                &latest_l1_as_of_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                &generated_at_ms.to_string(),
                &summary.run_identity_parts.join("|"),
            ],
        ),
        source_cycle_summary_file: None,
        run_dir: None,
        scheduler_action,
        source_verdict: source_verdict.to_owned(),
        run_not_before_ms,
        run_not_before_at: run_not_before_ms.map(iso8601_ms),
        run_not_before_source: run_not_before_ms
            .map(|_| "pending_shadow_target_exit_deadline_ms".to_owned()),
        focused_research_manifest_file: None,
        focused_research_summary_file: None,
        latest_l1_as_of_ms,
        shadow_sample_state: ShadowCycleSampleState {
            shadow_validation_count: shadow_runs.len(),
            target_window_materialized_count: summary.target_materialized_count,
            candidate_lifecycle_count,
            partially_materialized_candidate_count: partially_materialized_count,
            pending_target_window_candidate_count,
            total_sample_deficit,
            symbols: summary.symbols.into_iter().collect(),
        },
        safe_next_actions: safe_next_actions(source_verdict),
        blocked_actions: blocked_actions(source_verdict),
        safety: ShadowCycleDecisionSafety {
            s3_write: false,
            ecs_task_started: false,
            dispatcher_mode_changed: false,
            local_decision_only: true,
            shadow_status_mutated: false,
            paper_live_enabled: false,
            live_enabled: false,
            order_execution_enabled: false,
        },
    }
}

fn summarize_shadow_runs(
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
) -> ShadowCycleBuildSummary {
    let mut candidates = BTreeMap::<String, CandidateShadowState>::new();
    let mut symbols = BTreeSet::new();
    let mut target_materialized_count = 0usize;
    let mut run_identity_parts = Vec::new();

    for run in shadow_runs {
        run_identity_parts.push(run.shadow_validation_run_id.clone());
        symbols.insert(run.symbol_canonical.clone());
        let target_deadline_ms = target_exit_deadline_ms(run);
        let target_materialized = latest_l1_as_of_ms
            .zip(target_deadline_ms)
            .is_some_and(|(latest, target)| latest >= target);
        if target_materialized {
            target_materialized_count += 1;
        }

        let state = candidates
            .entry(run.candidate_lifecycle_key.clone())
            .or_insert_with(CandidateShadowState::new);
        state.symbol_set.insert(run.symbol_canonical.clone());
        state.observed_count += 1;
        state.required_count = state
            .required_count
            .max(run.watch_window_policy.min_shadow_samples);
        if run.status == ShadowValidationStatus::Pending {
            state.pending_count += 1;
        }
        if target_materialized {
            state.target_materialized_count += 1;
        } else if let Some(deadline) = target_deadline_ms {
            state.pending_target_count += 1;
            state.next_pending_target_deadline_ms =
                min_optional_ms(state.next_pending_target_deadline_ms, Some(deadline));
        }
    }

    ShadowCycleBuildSummary {
        candidates,
        symbols,
        target_materialized_count,
        run_identity_parts,
    }
}

fn select_scheduler_action(
    shadow_runs_empty: bool,
    latest_l1_watermark_known: bool,
    target_waiting_count: usize,
    partially_materialized_count: usize,
    deficient_count: usize,
    pending_count: usize,
    sample_ready_count: usize,
) -> (&'static str, ShadowCycleSchedulerAction) {
    if shadow_runs_empty {
        ("NO_SHADOW_CANDIDATES", ShadowCycleSchedulerAction::Noop)
    } else if !latest_l1_watermark_known {
        (
            "DISCOVER_LATEST_MARKET_L1_AS_OF",
            ShadowCycleSchedulerAction::DiscoverMarketL1Watermark,
        )
    } else if target_waiting_count > 0 {
        (
            "WAIT_FOR_TARGET_HOLDING_WINDOW",
            ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes,
        )
    } else if partially_materialized_count > 0 {
        (
            "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION",
            ShadowCycleSchedulerAction::WaitUntilPendingShadowTargetWindowMaterializes,
        )
    } else if deficient_count > 0 {
        (
            "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION",
            ShadowCycleSchedulerAction::HoldForOperatorReview,
        )
    } else if pending_count > 0 || sample_ready_count > 0 {
        (
            "REVIEW_SHADOW_COMPLETION_EVIDENCE",
            ShadowCycleSchedulerAction::ReviewShadowCompletionEvidence,
        )
    } else {
        (
            "NO_SHADOW_SAMPLE_GAP_DETECTED",
            ShadowCycleSchedulerAction::Noop,
        )
    }
}

pub fn read_shadow_cycle_decision(path: &Path) -> AppResult<ShadowCycleDecision> {
    if !path.is_absolute() {
        return Err(AppError::config(
            "shadow cycle decision file must be an absolute path",
        ));
    }
    let raw = fs::read_to_string(path)?;
    let decision = serde_json::from_str(&raw)?;
    Ok(decision)
}

fn target_exit_deadline_ms(run: &ShadowValidationRun) -> Option<i64> {
    let absolute_deadline = run.holding_policy.absolute_exit_deadline_ms;
    let absolute_hours = i64::from(run.holding_policy.absolute_max_holding_hours);
    let target_hours = i64::from(run.holding_policy.target_max_holding_hours);
    if absolute_deadline <= 0 || absolute_hours <= 0 || target_hours <= 0 {
        return None;
    }
    Some(absolute_deadline - (absolute_hours * MS_PER_HOUR) + (target_hours * MS_PER_HOUR))
}

fn min_optional_ms(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn safe_next_actions(source_verdict: &str) -> Vec<String> {
    match source_verdict {
        "DISCOVER_LATEST_MARKET_L1_AS_OF" => vec!["discover_latest_market_l1_as_of"],
        "WAIT_FOR_TARGET_HOLDING_WINDOW" => vec![
            "wait_for_target_holding_window_materialization",
            "keep_shadow_status_pending_until_completion_evidence_exists",
        ],
        "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION" => vec![
            "wait_for_pending_shadow_target_window_materialization",
            "keep_shadow_status_pending_until_completion_evidence_exists",
        ],
        "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" => vec![
            "build_retest_horizon_status_before_focused_accumulation",
            "keep_shadow_status_pending_until_completion_evidence_exists",
        ],
        "REVIEW_SHADOW_COMPLETION_EVIDENCE" => {
            vec!["review_shadow_completion_evidence"]
        }
        _ => Vec::new(),
    }
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}

fn blocked_actions(source_verdict: &str) -> Vec<String> {
    let mut actions = vec![
        "do_not_mark_pending_shadow_passed_from_sample_counts_only",
        "do_not_create_paper_without_completed_passed_shadow",
        "do_not_enable_live_from_shadow_sample_gap_manifest",
    ];
    if source_verdict == "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION" {
        actions.push("do_not_enable_live_from_shadow_accumulation_manifest");
    }
    actions.into_iter().map(ToOwned::to_owned).collect()
}

fn iso8601_ms(value: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(value)
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn validate_shadow_cycle_decision(decision: &ShadowCycleDecision) -> AppResult<()> {
    if decision.schema_version != SHADOW_CYCLE_DECISION_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "shadow cycle decision schema_version must be {SHADOW_CYCLE_DECISION_SCHEMA_VERSION}; got {}",
            decision.schema_version
        )));
    }
    if decision.decision_id.trim().is_empty() {
        return Err(AppError::validation(
            "shadow cycle decision decision_id must be non-empty",
        ));
    }
    validate_scheduler_action(decision)?;
    validate_safety(decision)?;
    validate_blocked_actions(decision)?;
    Ok(())
}

fn validate_scheduler_action(decision: &ShadowCycleDecision) -> AppResult<()> {
    if decision.scheduler_action.is_wait_action() {
        if decision.run_not_before_ms.is_none() {
            return Err(AppError::validation(
                "wait shadow cycle decisions must include run_not_before_ms",
            ));
        }
        if decision.focused_research_manifest_file.is_some() {
            return Err(AppError::validation(
                "wait shadow cycle decisions must not include a focused research manifest",
            ));
        }
    }

    if decision
        .scheduler_action
        .requires_focused_research_manifest()
    {
        let Some(manifest_file) = &decision.focused_research_manifest_file else {
            return Err(AppError::validation(
                "focused shadow sample accumulation decisions must include focused_research_manifest_file",
            ));
        };
        if !manifest_file.starts_with('/') {
            return Err(AppError::validation(
                "focused_research_manifest_file must be an absolute local path",
            ));
        }
        if decision.run_not_before_ms.is_some() {
            return Err(AppError::validation(
                "focused shadow sample accumulation decisions must not include run_not_before_ms",
            ));
        }
    }

    if matches!(
        decision.scheduler_action,
        ShadowCycleSchedulerAction::Noop | ShadowCycleSchedulerAction::HoldForOperatorReview
    ) && (decision.run_not_before_ms.is_some()
        || decision.focused_research_manifest_file.is_some())
    {
        return Err(AppError::validation(
            "noop/operator-review shadow cycle decisions must not schedule work",
        ));
    }

    Ok(())
}

fn validate_safety(decision: &ShadowCycleDecision) -> AppResult<()> {
    let safety = &decision.safety;
    if safety.s3_write
        || safety.ecs_task_started
        || safety.dispatcher_mode_changed
        || safety.shadow_status_mutated
        || safety.paper_live_enabled
        || safety.live_enabled
        || safety.order_execution_enabled
    {
        return Err(AppError::validation(
            "shadow cycle decision must be local-only and must not enable paper/live/order execution",
        ));
    }
    if !safety.local_decision_only {
        return Err(AppError::validation(
            "shadow cycle decision must set local_decision_only=true",
        ));
    }
    Ok(())
}

fn validate_blocked_actions(decision: &ShadowCycleDecision) -> AppResult<()> {
    let required_actions = [
        "do_not_create_paper_without_completed_passed_shadow",
        "do_not_enable_live_from_shadow_sample_gap_manifest",
    ];
    for required_action in required_actions {
        if !decision
            .blocked_actions
            .iter()
            .any(|action| action == required_action)
        {
            return Err(AppError::validation(format!(
                "shadow cycle decision missing blocked action: {required_action}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        HOLDING_POLICY_VERSION, HoldingPolicy, ShadowCycleDecision, ShadowStartConditionSummary,
        ShadowTerminationPolicy, ShadowWatchWindowPolicy, SurvivalBand,
    };

    const TARGET_HOURS: u32 = 24;
    const ABSOLUTE_HOURS: u32 = 72;

    #[test]
    fn validates_wait_decision_contract() {
        let decision: ShadowCycleDecision = serde_json::from_str(
            r#"{
              "schema_version": "research_shadow_cycle_decision_v1",
              "generated_at": "2026-05-24T12:16:00Z",
              "decision_id": "shadow_cycle_decision:run:WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION:1779670979756",
              "source_cycle_summary_file": "/tmp/run/shadow-sample-accumulation-cycle-summary.json",
              "run_dir": "/tmp/run",
              "scheduler_action": "WAIT_UNTIL_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZES",
              "source_verdict": "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION",
              "run_not_before_ms": 1779670979756,
              "run_not_before_at": "2026-05-25T01:02:59Z",
              "run_not_before_source": "pending_shadow_target_exit_deadline_ms",
              "focused_research_manifest_file": null,
              "focused_research_summary_file": null,
              "latest_l1_as_of_ms": null,
              "shadow_sample_state": {
                "shadow_validation_count": 24,
                "target_window_materialized_count": 12,
                "candidate_lifecycle_count": 6,
                "partially_materialized_candidate_count": 6,
                "pending_target_window_candidate_count": 6,
                "total_sample_deficit": 168,
                "symbols": ["BTC", "DOGE", "ETH", "SOL", "TON", "ZEC"]
              },
              "safe_next_actions": ["wait_for_pending_shadow_target_window_materialization"],
              "blocked_actions": [
                "do_not_mark_pending_shadow_passed_from_sample_counts_only",
                "do_not_create_paper_without_completed_passed_shadow",
                "do_not_enable_live_from_shadow_sample_gap_manifest"
              ],
              "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_decision_only": true,
                "shadow_status_mutated": false,
                "paper_live_enabled": false,
                "live_enabled": false,
                "order_execution_enabled": false
              }
            }"#,
        )
        .expect("wait decision parses");

        validate_shadow_cycle_decision(&decision).expect("wait decision validates");
    }

    #[test]
    fn validates_focused_accumulation_decision_contract() {
        let decision: ShadowCycleDecision = serde_json::from_str(
            r#"{
              "schema_version": "research_shadow_cycle_decision_v1",
              "generated_at": "2026-05-24T12:16:00Z",
              "decision_id": "shadow_cycle_decision:run:ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION:1779700000000",
              "source_cycle_summary_file": "/tmp/run/shadow-sample-accumulation-cycle-summary.json",
              "run_dir": "/tmp/run",
              "scheduler_action": "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH",
              "source_verdict": "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION",
              "run_not_before_ms": null,
              "run_not_before_at": null,
              "run_not_before_source": null,
              "focused_research_manifest_file": "/tmp/run/shadow-accumulation-input-manifest.next.json",
              "focused_research_summary_file": "/tmp/run/shadow-accumulation-input-manifest.next.summary.json",
              "latest_l1_as_of_ms": 1779700000000,
              "shadow_sample_state": {
                "shadow_validation_count": 24,
                "target_window_materialized_count": 24,
                "candidate_lifecycle_count": 6,
                "partially_materialized_candidate_count": 0,
                "pending_target_window_candidate_count": 0,
                "total_sample_deficit": 156,
                "symbols": ["BTC", "DOGE", "ETH", "SOL", "TON", "ZEC"]
              },
              "safe_next_actions": ["accumulate_shadow_observation_samples"],
              "blocked_actions": [
                "do_not_mark_pending_shadow_passed_from_sample_counts_only",
                "do_not_create_paper_without_completed_passed_shadow",
                "do_not_enable_live_from_shadow_accumulation_manifest",
                "do_not_enable_live_from_shadow_sample_gap_manifest"
              ],
              "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_decision_only": true,
                "shadow_status_mutated": false,
                "paper_live_enabled": false,
                "live_enabled": false,
                "order_execution_enabled": false
              }
            }"#,
        )
        .expect("focused decision parses");

        validate_shadow_cycle_decision(&decision).expect("focused decision validates");
    }

    #[test]
    fn rejects_wait_decision_without_not_before_time() {
        let decision: ShadowCycleDecision = serde_json::from_str(
            r#"{
              "schema_version": "research_shadow_cycle_decision_v1",
              "generated_at": "2026-05-24T12:16:00Z",
              "decision_id": "shadow_cycle_decision:run:wait:none",
              "scheduler_action": "WAIT_UNTIL_TARGET_WINDOW_MATERIALIZES",
              "source_verdict": "WAIT_FOR_TARGET_HOLDING_WINDOW",
              "shadow_sample_state": {
                "shadow_validation_count": 1,
                "target_window_materialized_count": 0,
                "candidate_lifecycle_count": 1,
                "partially_materialized_candidate_count": 0,
                "pending_target_window_candidate_count": 1,
                "total_sample_deficit": 30,
                "symbols": ["BTC"]
              },
              "blocked_actions": [
                "do_not_create_paper_without_completed_passed_shadow",
                "do_not_enable_live_from_shadow_sample_gap_manifest"
              ],
              "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_decision_only": true,
                "shadow_status_mutated": false,
                "paper_live_enabled": false,
                "live_enabled": false,
                "order_execution_enabled": false
              }
            }"#,
        )
        .expect("invalid wait decision parses");

        let error = validate_shadow_cycle_decision(&decision).expect_err("wait time is required");
        assert!(error.to_string().contains("run_not_before_ms"));
    }

    #[test]
    fn rejects_decision_that_enables_order_execution() {
        let decision: ShadowCycleDecision = serde_json::from_str(
            r#"{
              "schema_version": "research_shadow_cycle_decision_v1",
              "generated_at": "2026-05-24T12:16:00Z",
              "decision_id": "shadow_cycle_decision:run:unsafe",
              "scheduler_action": "NOOP",
              "source_verdict": "NO_SHADOW_SAMPLE_GAP_DETECTED",
              "shadow_sample_state": {
                "shadow_validation_count": 0,
                "target_window_materialized_count": 0,
                "candidate_lifecycle_count": 0,
                "partially_materialized_candidate_count": 0,
                "pending_target_window_candidate_count": 0,
                "total_sample_deficit": 0,
                "symbols": []
              },
              "blocked_actions": [
                "do_not_create_paper_without_completed_passed_shadow",
                "do_not_enable_live_from_shadow_sample_gap_manifest"
              ],
              "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_decision_only": true,
                "shadow_status_mutated": false,
                "paper_live_enabled": false,
                "live_enabled": false,
                "order_execution_enabled": true
              }
            }"#,
        )
        .expect("unsafe decision parses");

        let error =
            validate_shadow_cycle_decision(&decision).expect_err("order execution is rejected");
        assert!(error.to_string().contains("paper/live/order execution"));
    }

    #[test]
    fn builds_wait_decision_until_target_windows_materialize() {
        let decision_available_ms = 1_780_000_000_000;
        let materialized_target_ms = decision_available_ms + i64::from(TARGET_HOURS) * MS_PER_HOUR;
        let later_decision_ms = decision_available_ms + 2 * MS_PER_HOUR;
        let runs = vec![
            shadow_run("shadow_a", "cand_a", "XAUT", decision_available_ms, 30),
            shadow_run("shadow_b", "cand_b", "CHIP", later_decision_ms, 30),
        ];

        let decision =
            build_shadow_cycle_decision(&runs, Some(materialized_target_ms), 1_780_100_000_000);

        assert_eq!(
            decision.scheduler_action,
            ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes
        );
        assert_eq!(decision.source_verdict, "WAIT_FOR_TARGET_HOLDING_WINDOW");
        assert_eq!(
            decision.run_not_before_ms,
            Some(later_decision_ms + i64::from(TARGET_HOURS) * MS_PER_HOUR)
        );
        assert_eq!(decision.shadow_sample_state.shadow_validation_count, 2);
        assert_eq!(
            decision
                .shadow_sample_state
                .target_window_materialized_count,
            1
        );
        assert_eq!(
            decision
                .shadow_sample_state
                .pending_target_window_candidate_count,
            1
        );
        validate_shadow_cycle_decision(&decision).expect("generated wait decision validates");
    }

    #[test]
    fn builds_operator_review_decision_when_samples_are_deficient() {
        let decision_available_ms = 1_780_000_000_000;
        let target_ms = decision_available_ms + i64::from(TARGET_HOURS) * MS_PER_HOUR;
        let runs = vec![shadow_run(
            "shadow_a",
            "cand_a",
            "XAUT",
            decision_available_ms,
            30,
        )];

        let decision = build_shadow_cycle_decision(&runs, Some(target_ms), 1_780_100_000_000);

        assert_eq!(
            decision.scheduler_action,
            ShadowCycleSchedulerAction::HoldForOperatorReview
        );
        assert_eq!(
            decision.source_verdict,
            "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION"
        );
        assert_eq!(decision.run_not_before_ms, None);
        assert_eq!(decision.shadow_sample_state.total_sample_deficit, 29);
        validate_shadow_cycle_decision(&decision).expect("generated hold decision validates");
    }

    fn shadow_run(
        shadow_validation_run_id: &str,
        candidate_lifecycle_key: &str,
        symbol_canonical: &str,
        decision_available_ms: i64,
        min_shadow_samples: usize,
    ) -> ShadowValidationRun {
        ShadowValidationRun {
            shadow_validation_run_id: shadow_validation_run_id.to_owned(),
            candidate_lifecycle_key: candidate_lifecycle_key.to_owned(),
            symbol_canonical: symbol_canonical.to_owned(),
            trigger_research_run_id: "research_report_test".to_owned(),
            start_condition_summary: ShadowStartConditionSummary {
                research_aggregate_key: "aggregate_test".to_owned(),
                gate_policy_version: "test_gate_policy".to_owned(),
                completed_count: 30,
                mean_net_after_cost_bps: Some(12.0),
                win_rate_ppm: Some(600_000),
                profit_factor_ppm: Some(1_200_000),
                gate_reason_codes: vec!["deterministic_shadow_gate_passed".to_owned()],
            },
            expected_survival_band: SurvivalBand::Stable,
            watch_window_policy: ShadowWatchWindowPolicy {
                mode: "forward_observation_only".to_owned(),
                min_shadow_samples,
                max_shadow_age_days: 30,
            },
            termination_policy: ShadowTerminationPolicy {
                prune_on_non_positive_mean_net: true,
                prune_on_max_age_without_samples: true,
                no_order_execution: true,
            },
            holding_policy: HoldingPolicy {
                target_max_holding_hours: TARGET_HOURS,
                absolute_max_holding_hours: ABSOLUTE_HOURS,
                absolute_exit_deadline_ms: decision_available_ms
                    + i64::from(ABSOLUTE_HOURS) * MS_PER_HOUR,
                force_flat_policy: "daily_or_ttl_exit".to_owned(),
                overnight_risk_exception: false,
                holding_policy_version: HOLDING_POLICY_VERSION.to_owned(),
            },
            status: ShadowValidationStatus::Pending,
            passed: false,
            paper_trade_candidate_contract_version: "paper_trade_candidate_v1".to_owned(),
            schema_version: "shadow_validation_run_v1".to_owned(),
        }
    }
}
