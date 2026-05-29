use super::formatting::{count_join, unique_join};
use crate::alert::config::{AlertConfig, AlertPriority};
use crate::alert::event::AlertEvent;
use crate::model::PaperWatchLiveMark;

pub(in crate::alert) fn paper_watch_live_mark_alert_event(
    marks: &[PaperWatchLiveMark],
    config: &AlertConfig,
) -> Option<AlertEvent> {
    if marks.is_empty() {
        return None;
    }
    let unsafe_mark = marks.iter().any(|mark| {
        !mark.safety.paper_only
            || mark.safety.live_enabled
            || mark.safety.order_execution_enabled
            || mark.safety.execution_approval_emitted
    });
    if !unsafe_mark {
        return None;
    }
    let priority = AlertPriority::P0;
    if !config.allows(priority) {
        return None;
    }

    let symbols = unique_join(marks.iter().map(|mark| mark.symbol_canonical.as_str()));
    let venue_counts = count_join(marks.iter().map(|mark| mark.venue.as_str()));
    let state_counts = count_join(marks.iter().map(|mark| mark.lifecycle_state.as_str()));
    let net_returns = marks
        .iter()
        .map(|mark| mark.net_return_bps)
        .collect::<Vec<_>>();
    let min_net = net_returns.iter().copied().fold(f64::INFINITY, f64::min);
    let max_net = net_returns
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let avg_net = net_returns.iter().sum::<f64>() / net_returns.len() as f64;

    Some(AlertEvent {
        priority,
        title: format!("paper-watch live safety boundary changed: {} marks", marks.len()),
        conclusion: "paper-watch live mark 안에서 paper-only 안전 경계가 깨진 항목이 감지됐습니다. 실제 주문 설정이 열린 것인지 즉시 확인해야 합니다."
            .to_owned(),
        current_state: vec![
            format!("관찰 코인: {symbols}"),
            format!("생성된 live mark: {}개", marks.len()),
            format!("거래소별 mark: {venue_counts}"),
            format!("후보 상태: {state_counts}"),
            format!("모의 수익률 범위: {min_net:.2} ~ {max_net:.2} bps"),
            format!("모의 평균 수익률: {avg_net:.2} bps"),
        ],
        reasons: Vec::new(),
        next_actions: vec![
            "live/order 설정이 의도적으로 열린 것인지 즉시 확인합니다.".to_owned(),
            "의도한 변경이 아니면 paper-watch observer와 downstream 승급 흐름을 멈춥니다."
                .to_owned(),
        ],
        safety: vec![
            "실제 주문: 꺼짐".to_owned(),
            "실제 돈 사용: 없음".to_owned(),
            "paper-watch live mark만 기록".to_owned(),
        ],
    })
}
