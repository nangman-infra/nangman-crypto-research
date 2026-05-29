mod candidate_refs;
mod counts;
mod rows;

use self::candidate_refs::{selected_candidate_refs, source_candidate_refs};
use self::counts::{action_counts, horizon_counts, unique_sorted};
use self::rows::focus_rows;
use super::types::{
    FocusedRetestBuildOptions, FocusedRetestManifestBuild, FocusedRetestManifestSafety,
    FocusedRetestManifestSourceSummary, FocusedRetestManifestSummary,
    FocusedRetestSelectionSummary,
};
use crate::error::{AppError, AppResult};
use crate::model::{
    FOCUSED_RETEST_MANIFEST_SUMMARY_SCHEMA_VERSION, ResearchArtifactRef, ResearchInputManifest,
};
use crate::retest_cycle::validate_retest_horizon_status;
use serde_json::Value;

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

    let rows = focus_rows(
        status,
        &options.next_actions,
        &options.candidate_lifecycle_key_filter,
    )?;
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
