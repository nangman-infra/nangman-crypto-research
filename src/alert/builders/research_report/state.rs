use super::metrics::ResearchAlertMetrics;
use crate::alert::builders::formatting::unique_join;
use crate::model::{PaperWatchCandidate, ResearchRunReport};

pub(super) fn current_state_lines(
    report: &ResearchRunReport,
    paper_watch_candidates: &[PaperWatchCandidate],
    metrics: &ResearchAlertMetrics,
) -> Vec<String> {
    let mut lines = vec![
        format!("report_id: {}", report.research_run_report_id),
        format!("실행 범위: {}", report.run_scope),
        format!("전체 후보: {}개", report.summary_findings.len()),
        format!(
            "판정: RETEST {} / PRUNE {} / SHADOW 승급 {} / PAPER 승급 {}",
            metrics.retest_count,
            metrics.prune_count,
            metrics.promote_shadow_count,
            metrics.promote_paper_count
        ),
        format!("shadow 관찰 생성: {}개", metrics.shadow_count),
        format!("모의 관찰 후보: {}개", metrics.paper_watch_count),
        format!("paper 후보: {}개", metrics.paper_count),
        format!("실제 투자 비중: {:.4}", metrics.max_total_notional_pct),
    ];
    lines.extend(paper_watch_candidate_summary_lines(paper_watch_candidates));
    lines
}

fn paper_watch_candidate_summary_lines(candidates: &[PaperWatchCandidate]) -> Vec<String> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let symbols = unique_join(
        candidates
            .iter()
            .map(|candidate| candidate.symbol_canonical.as_str()),
    );
    let completed = candidates
        .iter()
        .map(|candidate| candidate.replay_sample_summary.completed_count)
        .sum::<usize>();
    let replay_runs = candidates
        .iter()
        .map(|candidate| candidate.replay_sample_summary.replay_run_count)
        .sum::<usize>();
    let positive = candidates
        .iter()
        .map(|candidate| candidate.replay_sample_summary.positive_net_count)
        .sum::<usize>();
    let non_positive = candidates
        .iter()
        .map(|candidate| candidate.replay_sample_summary.non_positive_net_count)
        .sum::<usize>();
    let net_values = candidates
        .iter()
        .filter_map(|candidate| {
            candidate
                .replay_sample_summary
                .weighted_mean_net_after_cost_bps
        })
        .collect::<Vec<_>>();
    let mut lines = vec![
        format!("관찰 코인: {symbols}"),
        format!("완료 replay 샘플: {completed}/{replay_runs}"),
        format!("positive/non-positive 샘플: {positive}/{non_positive}"),
    ];
    if !net_values.is_empty() {
        let min_net = net_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_net = net_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        lines.push(format!(
            "과거 검증 net_after_cost 범위: {min_net:.2} ~ {max_net:.2} bps"
        ));
    }
    lines
}
