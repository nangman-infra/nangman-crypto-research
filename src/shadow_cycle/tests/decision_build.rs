use super::*;

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

#[test]
fn exposes_shadow_sample_deficit_lifecycle_keys_after_target_materializes() {
    let decision_available_ms = 1_780_000_000_000;
    let target_ms = decision_available_ms + i64::from(TARGET_HOURS) * MS_PER_HOUR;
    let pending_target_ms = target_ms - 1;
    let runs = vec![
        shadow_run("shadow_a", "cand_a", "XAUT", decision_available_ms, 30),
        shadow_run("shadow_b", "cand_b", "CHIP", decision_available_ms + 1, 30),
    ];

    assert!(shadow_sample_deficit_lifecycle_keys(&runs, Some(pending_target_ms)).is_empty());
    assert_eq!(
        shadow_sample_deficit_lifecycle_keys(&runs, Some(target_ms)),
        vec!["cand_a".to_owned()]
    );
}
