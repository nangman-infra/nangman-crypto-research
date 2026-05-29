use crate::alert::config::AlertPriority;
use crate::model::{ResearchBias, ResearchRunReport};
use std::collections::BTreeMap;

pub(in crate::alert) fn top_reason_lines(
    report: &ResearchRunReport,
    priority: AlertPriority,
) -> Vec<String> {
    let mut reason_counts = BTreeMap::<String, usize>::new();
    for finding in &report.summary_findings {
        if priority == AlertPriority::P3 && finding.bias != ResearchBias::RetestBias {
            continue;
        }
        for reason in &finding.reason_codes {
            *reason_counts.entry(reason.clone()).or_default() += 1;
        }
    }
    let mut rows = reason_counts.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    rows.into_iter()
        .take(5)
        .map(|(reason, count)| format!("{}: {}개", human_reason(&reason), count))
        .collect()
}

fn human_reason(reason: &str) -> &'static str {
    match reason {
        "native_replay_horizon_not_materialized" => "목표 보유시간만큼의 시장 데이터가 아직 부족함",
        "native_replay_positive_but_promotion_blocked" => {
            "과거 검증은 긍정적이지만 승급 조건이 아직 부족함"
        }
        "native_replay_positive_but_survival_not_proven" => {
            "시간이 지나도 신호가 유지되는지 아직 증명되지 않음"
        }
        "needs_unseen_window_validation" | "unseen_window_validation_not_proven" => {
            "아직 보지 않은 구간 검증이 부족함"
        }
        "no_completed_native_replay_samples" => "완료된 replay 샘플이 아직 없음",
        "promotion_sample_count_below_minimum" => "승급에 필요한 샘플 수가 부족함",
        "liquidity_filter_not_materialized" => "유동성 검증 데이터가 아직 준비되지 않음",
        "liquidity_filter_no_positive_volume_observed" => {
            "거래량 검증에서 충분한 유동성이 확인되지 않음"
        }
        "deterministic_shadow_gate_passed" => "shadow 관찰로 올릴 조건을 통과함",
        "shadow_validation_passed" => "shadow 검증을 통과함",
        "portfolio_notional_non_zero" => "실제 투자 비중이 0보다 큼",
        _ => "기타 검증 조건 미충족",
    }
}
