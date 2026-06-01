use super::super::*;

#[test]
fn promote_to_shadow_alert_has_p2_priority() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::PromoteToShadowBias,
        reason_codes: vec!["deterministic_shadow_gate_passed".to_owned()],
    }]);
    let config = test_config(AlertPriority::P2);
    let event = research_report_alert_event(&report, &[], &config).expect("event is created");

    assert_eq!(event.priority, AlertPriority::P2);
    assert!(event.text("dev").contains("PROMOTE_TO_SHADOW"));
}

#[test]
fn promote_to_paper_alert_has_p1_priority() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::PromoteToPaperBias,
        reason_codes: vec!["shadow_validation_passed".to_owned()],
    }]);
    let event = research_report_alert_event(&report, &[], &test_config(AlertPriority::P1))
        .expect("paper event is created");

    assert_eq!(event.priority, AlertPriority::P1);
    assert!(
        event
            .text("prod")
            .contains("실제 주문과 live trading은 계속 꺼둡니다.")
    );
}

#[test]
fn non_zero_portfolio_notional_forces_p0_alert() {
    let mut report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec!["portfolio_notional_non_zero".to_owned()],
    }]);
    report.portfolio_allocation_snapshot = Some(test_portfolio_snapshot());

    let event = research_report_alert_event(&report, &[], &test_config(AlertPriority::P0))
        .expect("portfolio event is created");

    assert_eq!(event.priority, AlertPriority::P0);
    assert!(event.text("dev").contains("실제 투자 비중: 0.1000"));
}
