use crate::error::{AppError, AppResult};
use crate::model::{
    FOCUSED_RETEST_MANIFEST_SUMMARY_SCHEMA_VERSION, ResearchArtifactRef, ResearchInputManifest,
};
use crate::retest_cycle::validate_retest_horizon_status;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_FOCUSED_RETEST_ACTIONS: &[&str] = &[
    "run_research_replay_for_horizon",
    "accumulate_completed_native_replay_samples",
    "materialize_completed_native_replay_sample",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoricalReplayIndexRefMode {
    Auto,
    Always,
    Never,
}

impl HistoricalReplayIndexRefMode {
    pub fn parse(raw: &str) -> AppResult<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "true" | "always" => Ok(Self::Always),
            "false" | "never" => Ok(Self::Never),
            other => Err(AppError::config(format!(
                "focused retest historical replay index ref mode must be auto, true, or false; got {other}"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Always => "true",
            Self::Never => "false",
        }
    }

    fn should_carry(self, actions: &[String]) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => actions
                .iter()
                .any(|action| action == "accumulate_completed_native_replay_samples"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedRetestBuildOptions {
    pub generated_at_ms: i64,
    pub research_packet_id: String,
    pub run_scope: String,
    pub next_actions: Vec<String>,
    pub historical_replay_index_ref_mode: HistoricalReplayIndexRefMode,
    pub s3_write: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestManifestSummary {
    pub schema_version: String,
    pub generated_at_ms: i64,
    pub focus_next_actions: Vec<String>,
    pub safety: FocusedRetestManifestSafety,
    pub source: FocusedRetestManifestSourceSummary,
    pub focused: FocusedRetestSelectionSummary,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestManifestSafety {
    pub s3_write: bool,
    pub ecs_task_started: bool,
    pub dispatcher_mode_changed: bool,
    pub shadow_paper_live_enabled: bool,
    pub selected_from_existing_retest_status: bool,
    pub historical_replay_run_index_ref_mode: String,
    pub historical_replay_run_index_refs_carried: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestManifestSourceSummary {
    pub research_packet_id: Option<String>,
    pub run_scope: Option<String>,
    pub candidate_bundle_ref_count: usize,
    pub historical_replay_run_index_ref_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestSelectionSummary {
    pub focus_horizon_count: usize,
    pub focus_candidate_count: usize,
    pub selected_candidate_bundle_ref_count: usize,
    pub selected_historical_replay_run_index_ref_count: usize,
    pub symbols: Vec<String>,
    pub next_action_counts: Vec<FocusedRetestActionCount>,
    pub horizons: Vec<FocusedRetestHorizonCount>,
    pub selected_candidate_ids: Vec<String>,
    pub missing_candidate_ref_ids: Vec<String>,
    pub rows: Vec<FocusedRetestRow>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestActionCount {
    pub next_action: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestHorizonCount {
    pub horizon: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestRow {
    pub candidate_id: String,
    pub candidate_lifecycle_key: Option<String>,
    pub symbol: String,
    pub symbols: Vec<String>,
    pub hypothesis_type: Option<String>,
    pub research_priority: Option<String>,
    pub horizon: String,
    pub next_action: String,
    pub replay_run_count: Option<i64>,
    pub completed_count: Option<i64>,
    pub completed_sample_deficit: Option<i64>,
    pub inferred_unseen_window_count: Option<i64>,
    pub unseen_window_deficit: Option<i64>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug)]
pub struct FocusedRetestManifestBuild {
    pub manifest: ResearchInputManifest,
    pub summary: FocusedRetestManifestSummary,
}

pub fn default_focused_retest_actions() -> Vec<String> {
    DEFAULT_FOCUSED_RETEST_ACTIONS
        .iter()
        .map(|action| (*action).to_owned())
        .collect()
}

pub fn parse_focused_retest_actions(raw: &str) -> Vec<String> {
    let mut actions = raw
        .split(',')
        .map(str::trim)
        .filter(|action| !action.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    actions.sort();
    actions.dedup();
    actions
}

pub fn build_focused_retest_manifest(
    status: &Value,
    source_manifest: &ResearchInputManifest,
    options: &FocusedRetestBuildOptions,
) -> AppResult<FocusedRetestManifestBuild> {
    validate_retest_horizon_status(status)?;
    if options.next_actions.is_empty() {
        return Err(AppError::config(
            "focused retest next action list must not be empty",
        ));
    }
    if options.research_packet_id.trim().is_empty() {
        return Err(AppError::config(
            "focused retest research_packet_id must not be empty",
        ));
    }
    if options.run_scope.trim().is_empty() {
        return Err(AppError::config(
            "focused retest run_scope must not be empty",
        ));
    }

    let rows = focus_rows(status, &options.next_actions)?;
    let focus_candidate_ids = unique_sorted(rows.iter().map(|row| row.candidate_id.as_str()));
    let source_refs = source_candidate_refs(source_manifest);
    let selected_refs = selected_candidate_refs(&source_refs, &focus_candidate_ids);
    let selected_candidate_ids = unique_sorted(
        selected_refs
            .iter()
            .filter_map(|candidate_ref| candidate_ref.candidate_id.as_deref()),
    );
    let missing_candidate_ref_ids = focus_candidate_ids
        .iter()
        .filter(|candidate_id| !selected_candidate_ids.contains(candidate_id))
        .cloned()
        .collect::<Vec<_>>();
    if selected_refs.is_empty() {
        return Err(AppError::validation(format!(
            "focused retest selected zero candidate bundle refs; focus_candidate_count={}, missing_candidate_ref_ids={}",
            focus_candidate_ids.len(),
            missing_candidate_ref_ids.join(",")
        )));
    }

    let carry_historical_index_refs = options
        .historical_replay_index_ref_mode
        .should_carry(&options.next_actions);
    let historical_replay_run_index_refs = if carry_historical_index_refs {
        source_manifest.historical_replay_run_index_refs.clone()
    } else {
        Vec::new()
    };

    let mut runtime_budget_policy = source_manifest.runtime_budget_policy.clone();
    runtime_budget_policy.max_candidate_bundle_count = selected_refs.len().max(1);

    let manifest = ResearchInputManifest {
        schema_version: source_manifest.schema_version.clone(),
        research_packet_id: Some(options.research_packet_id.clone()),
        run_scope: Some(options.run_scope.clone()),
        candidate_bundle_refs: selected_refs
            .iter()
            .map(|candidate_ref| ResearchArtifactRef {
                uri: candidate_ref.uri.clone(),
            })
            .collect(),
        market_feature_delta_refs: Vec::new(),
        market_regime_context_refs: Vec::new(),
        shadow_validation_run_refs: Vec::new(),
        hypothesis_harness_result_refs: Vec::new(),
        oss_adapter_run_refs: Vec::new(),
        historical_replay_run_refs: Vec::new(),
        historical_replay_run_index_refs,
        runtime_budget_policy,
    };

    let summary = FocusedRetestManifestSummary {
        schema_version: FOCUSED_RETEST_MANIFEST_SUMMARY_SCHEMA_VERSION.to_owned(),
        generated_at_ms: options.generated_at_ms,
        focus_next_actions: options.next_actions.clone(),
        safety: FocusedRetestManifestSafety {
            s3_write: options.s3_write,
            ecs_task_started: false,
            dispatcher_mode_changed: false,
            shadow_paper_live_enabled: false,
            selected_from_existing_retest_status: true,
            historical_replay_run_index_ref_mode: options
                .historical_replay_index_ref_mode
                .as_str()
                .to_owned(),
            historical_replay_run_index_refs_carried: carry_historical_index_refs,
        },
        source: FocusedRetestManifestSourceSummary {
            research_packet_id: source_manifest.research_packet_id.clone(),
            run_scope: source_manifest.run_scope.clone(),
            candidate_bundle_ref_count: source_manifest.candidate_bundle_refs.len(),
            historical_replay_run_index_ref_count: source_manifest
                .historical_replay_run_index_refs
                .len(),
        },
        focused: FocusedRetestSelectionSummary {
            focus_horizon_count: rows.len(),
            focus_candidate_count: focus_candidate_ids.len(),
            selected_candidate_bundle_ref_count: selected_refs.len(),
            selected_historical_replay_run_index_ref_count: manifest
                .historical_replay_run_index_refs
                .len(),
            symbols: unique_sorted(rows.iter().map(|row| row.symbol.as_str())),
            next_action_counts: action_counts(&rows),
            horizons: horizon_counts(&rows),
            selected_candidate_ids,
            missing_candidate_ref_ids,
            rows,
        },
    };

    Ok(FocusedRetestManifestBuild { manifest, summary })
}

fn focus_rows(status: &Value, actions: &[String]) -> AppResult<Vec<FocusedRetestRow>> {
    let action_set = actions.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    let Some(symbols) = status.get("by_symbol").and_then(Value::as_array) else {
        return Ok(rows);
    };
    for symbol_doc in symbols {
        let symbol = string_field(symbol_doc, "symbol").unwrap_or("UNKNOWN");
        let Some(candidates) = symbol_doc.get("candidates").and_then(Value::as_array) else {
            continue;
        };
        for candidate in candidates {
            let Some(candidate_id) = string_field(candidate, "candidate_id") else {
                continue;
            };
            let Some(horizons) = candidate.get("horizons").and_then(Value::as_array) else {
                continue;
            };
            for horizon in horizons {
                let Some(next_action) = string_field(horizon, "next_action") else {
                    continue;
                };
                if !action_set.contains(next_action) {
                    continue;
                }
                rows.push(FocusedRetestRow {
                    candidate_id: candidate_id.to_owned(),
                    candidate_lifecycle_key: string_field(candidate, "candidate_lifecycle_key")
                        .map(ToOwned::to_owned),
                    symbol: symbol.to_owned(),
                    symbols: string_array_field(horizon, "symbols"),
                    hypothesis_type: string_field(candidate, "hypothesis_type")
                        .map(ToOwned::to_owned),
                    research_priority: string_field(candidate, "research_priority")
                        .map(ToOwned::to_owned),
                    horizon: string_field(horizon, "horizon")
                        .unwrap_or("unknown")
                        .to_owned(),
                    next_action: next_action.to_owned(),
                    replay_run_count: integer_field(horizon, "replay_run_count"),
                    completed_count: integer_field(horizon, "completed_count"),
                    completed_sample_deficit: integer_field(horizon, "completed_sample_deficit"),
                    inferred_unseen_window_count: integer_field(
                        horizon,
                        "inferred_unseen_window_count",
                    ),
                    unseen_window_deficit: integer_field(horizon, "unseen_window_deficit"),
                    reason_codes: string_array_field(horizon, "reason_codes"),
                });
            }
        }
    }
    rows.sort_by(|left, right| {
        (
            left.symbol.as_str(),
            left.candidate_id.as_str(),
            horizon_order(&left.horizon),
        )
            .cmp(&(
                right.symbol.as_str(),
                right.candidate_id.as_str(),
                horizon_order(&right.horizon),
            ))
    });
    Ok(rows)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceCandidateRef {
    uri: String,
    candidate_id: Option<String>,
}

fn source_candidate_refs(source_manifest: &ResearchInputManifest) -> Vec<SourceCandidateRef> {
    source_manifest
        .candidate_bundle_refs
        .iter()
        .map(|artifact_ref| SourceCandidateRef {
            uri: artifact_ref.uri.clone(),
            candidate_id: candidate_id_from_uri(&artifact_ref.uri),
        })
        .collect()
}

fn selected_candidate_refs(
    source_refs: &[SourceCandidateRef],
    focus_candidate_ids: &[String],
) -> Vec<SourceCandidateRef> {
    let focus_candidate_ids = focus_candidate_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut seen_uris = BTreeSet::new();
    let mut selected = Vec::new();
    for source_ref in source_refs {
        let Some(candidate_id) = source_ref.candidate_id.as_deref() else {
            continue;
        };
        if !focus_candidate_ids.contains(candidate_id) || !seen_uris.insert(source_ref.uri.as_str())
        {
            continue;
        }
        selected.push(source_ref.clone());
    }
    selected
}

fn candidate_id_from_uri(uri: &str) -> Option<String> {
    let (_, rest) = uri.split_once("candidate_id=")?;
    let candidate_id = rest.split('/').next()?.trim();
    (!candidate_id.is_empty()).then(|| candidate_id.to_owned())
}

fn action_counts(rows: &[FocusedRetestRow]) -> Vec<FocusedRetestActionCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        *counts.entry(row.next_action.clone()).or_default() += 1;
    }
    let mut counts = counts
        .into_iter()
        .map(|(next_action, count)| FocusedRetestActionCount { next_action, count })
        .collect::<Vec<_>>();
    counts.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.next_action.cmp(&right.next_action))
    });
    counts
}

fn horizon_counts(rows: &[FocusedRetestRow]) -> Vec<FocusedRetestHorizonCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        *counts.entry(row.horizon.clone()).or_default() += 1;
    }
    let mut counts = counts
        .into_iter()
        .map(|(horizon, count)| FocusedRetestHorizonCount { horizon, count })
        .collect::<Vec<_>>();
    counts.sort_by_key(|count| horizon_order(&count.horizon));
    counts
}

fn unique_sorted<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    values
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn horizon_order(horizon: &str) -> u8 {
    match horizon {
        "1h" => 1,
        "4h" => 2,
        "24h" | "1d" => 3,
        "72h" => 4,
        "7d" => 5,
        _ => 99,
    }
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn integer_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}

fn string_array_field(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResearchRuntimeBudgetPolicy;
    use serde_json::json;

    #[test]
    fn builds_focused_manifest_for_ready_actions() {
        let source_manifest = source_manifest();
        let status = status_with_focus_rows();
        let build = build_focused_retest_manifest(
            &status,
            &source_manifest,
            &FocusedRetestBuildOptions {
                generated_at_ms: 1_779_719_361_452,
                research_packet_id: "research_focus_test".to_owned(),
                run_scope: "focused_retest_local_validation".to_owned(),
                next_actions: vec!["accumulate_completed_native_replay_samples".to_owned()],
                historical_replay_index_ref_mode: HistoricalReplayIndexRefMode::Auto,
                s3_write: false,
            },
        )
        .expect("focused manifest builds");

        assert_eq!(
            build.manifest.research_packet_id.as_deref(),
            Some("research_focus_test")
        );
        assert_eq!(build.manifest.candidate_bundle_refs.len(), 1);
        assert_eq!(
            build.manifest.candidate_bundle_refs[0].uri,
            "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_a/part-000001.jsonl"
        );
        assert_eq!(build.manifest.historical_replay_run_index_refs.len(), 1);
        assert_eq!(
            build
                .manifest
                .runtime_budget_policy
                .max_candidate_bundle_count,
            1
        );
        assert_eq!(build.summary.focused.focus_horizon_count, 1);
        assert_eq!(build.summary.focused.selected_candidate_bundle_ref_count, 1);
    }

    #[test]
    fn rejects_empty_focused_manifest() {
        let source_manifest = source_manifest();
        let status = status_with_focus_rows();
        let error = build_focused_retest_manifest(
            &status,
            &source_manifest,
            &FocusedRetestBuildOptions {
                generated_at_ms: 1_779_719_361_452,
                research_packet_id: "research_focus_test".to_owned(),
                run_scope: "focused_retest_local_validation".to_owned(),
                next_actions: vec!["run_research_replay_for_horizon".to_owned()],
                historical_replay_index_ref_mode: HistoricalReplayIndexRefMode::Auto,
                s3_write: false,
            },
        )
        .expect_err("empty selected refs are rejected");

        assert!(
            error
                .to_string()
                .contains("selected zero candidate bundle refs")
        );
    }

    fn source_manifest() -> ResearchInputManifest {
        ResearchInputManifest {
            schema_version: "research_input_manifest_v1".to_owned(),
            research_packet_id: Some("source_packet".to_owned()),
            run_scope: Some("current_approved".to_owned()),
            candidate_bundle_refs: vec![
                ResearchArtifactRef {
                    uri: "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_a/part-000001.jsonl".to_owned(),
                },
                ResearchArtifactRef {
                    uri: "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_b/part-000001.jsonl".to_owned(),
                },
            ],
            market_feature_delta_refs: Vec::new(),
            market_regime_context_refs: Vec::new(),
            shadow_validation_run_refs: Vec::new(),
            hypothesis_harness_result_refs: Vec::new(),
            oss_adapter_run_refs: Vec::new(),
            historical_replay_run_refs: Vec::new(),
            historical_replay_run_index_refs: vec![ResearchArtifactRef {
                uri: "s3://research/replay-run-index/part-000001.jsonl".to_owned(),
            }],
            runtime_budget_policy: ResearchRuntimeBudgetPolicy::default(),
        }
    }

    fn status_with_focus_rows() -> Value {
        json!({
            "schema_version": "research_horizon_status_checkpoint_v1",
            "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_summary_only": true,
                "shadow_paper_live_enabled": false
            },
            "stage_state": {
                "candidate_generated": true,
                "research_replay_completed": true,
                "promotion_passed": false,
                "shadow_created": false,
                "paper_created": false,
                "live_enabled": false
            },
            "next_decision": {
                "verdict": "WAIT_FOR_MARKET_L1_HORIZON",
                "scheduler_hint": {
                    "latest_l1_as_of_ms": 1_779_710_400_000_i64,
                    "run_research_after_l1_as_of_ms": 1_779_719_361_452_i64,
                    "run_now_replay_ready": false,
                    "promotion_ready_for_review": false
                },
                "blocked_actions": [
                    "do_not_create_shadow_without_promotion",
                    "do_not_create_paper_without_passed_shadow",
                    "do_not_enable_live_from_research_batch"
                ]
            },
            "by_symbol": [
                {
                    "symbol": "AAVE",
                    "candidates": [
                        {
                            "candidate_id": "cand_a",
                            "candidate_lifecycle_key": "cand_a:v1",
                            "hypothesis_type": "event_reaction",
                            "research_priority": "p0",
                            "horizons": [
                                {
                                    "horizon": "1h",
                                    "next_action": "accumulate_completed_native_replay_samples",
                                    "symbols": ["AAVE"],
                                    "replay_run_count": 3,
                                    "completed_count": 1,
                                    "completed_sample_deficit": 2,
                                    "reason_codes": ["sample_deficit"]
                                }
                            ]
                        }
                    ]
                }
            ]
        })
    }
}
