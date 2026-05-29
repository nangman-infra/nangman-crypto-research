mod actions;
mod metrics;
mod priority;
mod state;

use super::reasons::top_reason_lines;
use crate::alert::config::AlertConfig;
use crate::alert::event::AlertEvent;
use crate::model::{PaperWatchCandidate, ResearchRunReport};
use actions::research_next_actions;
use metrics::ResearchAlertMetrics;
use priority::research_alert_context;
use state::current_state_lines;

pub(in crate::alert) fn research_report_alert_event(
    report: &ResearchRunReport,
    paper_watch_candidates: &[PaperWatchCandidate],
    config: &AlertConfig,
) -> Option<AlertEvent> {
    let metrics = ResearchAlertMetrics::from_report(report, paper_watch_candidates);
    let alert_context = research_alert_context(&metrics, config)?;
    if !config.allows(alert_context.priority) {
        return None;
    }

    Some(AlertEvent {
        priority: alert_context.priority,
        title: alert_context.title,
        conclusion: alert_context.conclusion,
        current_state: current_state_lines(report, paper_watch_candidates, &metrics),
        reasons: top_reason_lines(report, alert_context.priority),
        next_actions: research_next_actions(alert_context.priority),
        safety: vec![
            "실제 주문: 꺼짐".to_owned(),
            "실제 돈 사용: 없음".to_owned(),
            "EXECUTION_APPROVED/LIVE_READY: research-app에서 발행하지 않음".to_owned(),
        ],
    })
}
