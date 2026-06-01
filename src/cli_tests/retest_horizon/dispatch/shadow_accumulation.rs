use super::fixtures::retest_cycle_source_state;
use super::*;

#[test]
fn shadow_accumulation_dispatch_filters_manifest_to_deficient_lifecycle_keys() {
    let args = default_args();
    let state = retest_cycle_source_state();
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();

    let dispatch = build_shadow_accumulation_manifest_dispatch(
        &args,
        &state,
        &status,
        &source_manifest,
        Some(7_201_300),
        7_400_000,
        vec!["cand_focus:v1".to_owned(), "missing:v1".to_owned()],
    )
    .expect("shadow accumulation dispatch builds")
    .expect("shadow accumulation dispatch is selected");

    assert!(dispatch.key.starts_with(
        "research-input-manifest/schema=research_input_manifest_v1/dedupe_key=research_shadow_accumulation_"
    ));
    assert_eq!(
        dispatch.manifest.run_scope.as_deref(),
        Some("shadow_sample_accumulation_local_validation")
    );
    assert_eq!(dispatch.manifest.candidate_bundle_refs.len(), 1);
    assert!(
        dispatch.manifest.candidate_bundle_refs[0]
            .uri
            .contains("candidate_id=cand_focus")
    );
    assert_eq!(dispatch.manifest.historical_replay_run_index_refs.len(), 1);
    assert_eq!(dispatch.focused_horizon_count, 1);
    assert_eq!(dispatch.focused_candidate_bundle_refs, 1);
    assert_eq!(
        dispatch.deficit_lifecycle_keys,
        vec!["cand_focus:v1".to_owned(), "missing:v1".to_owned()]
    );
}

#[test]
fn shadow_accumulation_dispatch_skips_empty_deficit_keys() {
    let args = default_args();
    let state = retest_cycle_source_state();
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();

    let dispatch = build_shadow_accumulation_manifest_dispatch(
        &args,
        &state,
        &status,
        &source_manifest,
        Some(7_201_300),
        7_400_000,
        Vec::new(),
    )
    .expect("empty deficit keys are valid");

    assert!(dispatch.is_none());
}
