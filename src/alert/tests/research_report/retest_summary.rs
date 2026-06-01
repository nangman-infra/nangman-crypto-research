use super::super::*;

#[test]
fn retest_summary_message_is_human_readable() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec![
            "promotion_sample_count_below_minimum".to_owned(),
            "unseen_window_validation_not_proven".to_owned(),
        ],
    }]);
    let mut config = test_config(AlertPriority::P3);
    config.include_retest_summary = true;
    let event = research_report_alert_event(&report, &[], &config).expect("event is created");
    let text = event.text("dev");

    assert!(text.contains("[P3][research-app] RETEST"));
    assert!(text.contains("결론:"));
    assert!(text.contains("현재 상태:"));
    assert!(text.contains("주요 원인:"));
    assert!(text.contains("다음 행동:"));
    assert!(text.contains("안전 상태:"));
    assert!(text.contains("승급에 필요한 샘플 수가 부족함: 1개"));
}

#[test]
fn retest_summary_is_quiet_by_default() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec!["promotion_sample_count_below_minimum".to_owned()],
    }]);

    assert!(research_report_alert_event(&report, &[], &test_config(AlertPriority::P3)).is_none());
}

#[test]
fn retest_summary_is_filtered_when_min_priority_is_too_high() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec!["promotion_sample_count_below_minimum".to_owned()],
    }]);
    let mut config = test_config(AlertPriority::P2);
    config.include_retest_summary = true;

    assert!(research_report_alert_event(&report, &[], &config).is_none());
}

#[test]
fn reason_summary_limits_to_top_five_reasons() {
    let report = test_report(vec![
        finding(
            "cand_001",
            ResearchBias::PromoteToShadowBias,
            &[
                "native_replay_horizon_not_materialized",
                "needs_unseen_window_validation",
            ],
        ),
        finding(
            "cand_002",
            ResearchBias::PromoteToShadowBias,
            &[
                "native_replay_horizon_not_materialized",
                "promotion_sample_count_below_minimum",
            ],
        ),
        finding(
            "cand_003",
            ResearchBias::PromoteToShadowBias,
            &[
                "native_replay_horizon_not_materialized",
                "promotion_sample_count_below_minimum",
            ],
        ),
        finding(
            "cand_004",
            ResearchBias::PromoteToShadowBias,
            &["liquidity_filter_not_materialized"],
        ),
        finding(
            "cand_005",
            ResearchBias::PromoteToShadowBias,
            &["liquidity_filter_no_positive_volume_observed"],
        ),
        finding(
            "cand_006",
            ResearchBias::PromoteToShadowBias,
            &["native_replay_positive_but_survival_not_proven"],
        ),
    ]);

    let reasons = top_reason_lines(&report, AlertPriority::P2);

    assert_eq!(reasons.len(), 5);
    assert_eq!(
        reasons[0],
        "목표 보유시간만큼의 시장 데이터가 아직 부족함: 3개"
    );
    assert!(reasons.contains(&"승급에 필요한 샘플 수가 부족함: 2개".to_owned()));
    assert!(!reasons.iter().any(|line| line.contains("positive_volume")));
}
