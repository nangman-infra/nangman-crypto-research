use super::super::*;

#[test]
fn paper_watch_alert_has_p2_priority_without_execution_language() {
    let mut report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec!["native_replay_positive_but_promotion_blocked".to_owned()],
    }]);
    let candidates = vec![test_paper_watch_candidate("TON")];
    report.paper_watch_candidates = candidates
        .iter()
        .map(|candidate| candidate.paper_watch_candidate_id.clone())
        .collect();
    let event = research_report_alert_event(&report, &candidates, &test_config(AlertPriority::P2))
        .expect("paper watch event is created");
    let text = event.text("dev");

    assert_eq!(event.priority, AlertPriority::P2);
    assert!(text.contains("모의 관찰 후보 1개 발생"));
    assert!(text.contains("관찰 코인: TON"));
    assert!(text.contains("완료 replay 샘플: 5/10"));
    assert!(text.contains("실제 주문: 꺼짐"));
    assert!(!text.contains("LIVE_READY: true"));
}
