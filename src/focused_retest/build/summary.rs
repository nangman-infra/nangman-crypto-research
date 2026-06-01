use super::super::types::{
    FocusedRetestBuildOptions, FocusedRetestManifestSafety, FocusedRetestManifestSourceSummary,
    FocusedRetestManifestSummary, FocusedRetestSelectionSummary,
};
use super::counts::{action_counts, horizon_counts, unique_sorted};
use super::selection::FocusedRetestSelection;
use crate::model::{FOCUSED_RETEST_MANIFEST_SUMMARY_SCHEMA_VERSION, ResearchInputManifest};

pub(super) fn build_summary(
    source_manifest: &ResearchInputManifest,
    options: &FocusedRetestBuildOptions,
    manifest: &ResearchInputManifest,
    selection: FocusedRetestSelection,
    carry_historical_index_refs: bool,
) -> FocusedRetestManifestSummary {
    FocusedRetestManifestSummary {
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
            focus_horizon_count: selection.rows.len(),
            focus_candidate_count: selection.focus_candidate_ids.len(),
            selected_candidate_bundle_ref_count: selection.selected_refs.len(),
            selected_historical_replay_run_index_ref_count: manifest
                .historical_replay_run_index_refs
                .len(),
            symbols: unique_sorted(selection.rows.iter().map(|row| row.symbol.as_str())),
            next_action_counts: action_counts(&selection.rows),
            horizons: horizon_counts(&selection.rows),
            selected_candidate_ids: selection.selected_candidate_ids,
            missing_candidate_ref_ids: selection.missing_candidate_ref_ids,
            rows: selection.rows,
        },
    }
}
