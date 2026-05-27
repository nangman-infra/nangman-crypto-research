use crate::model::{MarketLiveTick, PaperWatchCandidate, PaperWatchLiveMark, PaperWatchSafety};
use crate::paper_live::{PaperWatchLiveEntryBook, build_paper_watch_live_marks_with_entry_book};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

pub const PAPER_WATCH_OBSERVER_SNAPSHOT_SCHEMA_VERSION: &str = "paper_watch_observer_snapshot_v1";

#[derive(Debug, Clone, Default)]
pub struct PaperWatchObserverState {
    entry_book: PaperWatchLiveEntryBook,
    seen_live_mark_ids: BTreeSet<String>,
    marks_by_candidate: BTreeMap<String, Vec<PaperWatchLiveMark>>,
}

impl PaperWatchObserverState {
    pub fn restore_marks(&mut self, marks: &[PaperWatchLiveMark]) {
        let mut ordered_marks = marks.to_vec();
        ordered_marks.sort_by(|left, right| {
            (
                left.exchange_timestamp_ms,
                left.ingest_timestamp_ms,
                left.paper_watch_live_mark_id.as_str(),
            )
                .cmp(&(
                    right.exchange_timestamp_ms,
                    right.ingest_timestamp_ms,
                    right.paper_watch_live_mark_id.as_str(),
                ))
        });
        for mark in ordered_marks {
            self.restore_entry_from_mark(&mark);
            if self
                .seen_live_mark_ids
                .insert(mark.paper_watch_live_mark_id.clone())
            {
                self.marks_by_candidate
                    .entry(mark.paper_watch_candidate_id.clone())
                    .or_default()
                    .push(mark);
            }
        }
    }

    pub fn ingest_ticks(
        &mut self,
        candidates: &[PaperWatchCandidate],
        ticks: &[MarketLiveTick],
    ) -> Vec<PaperWatchLiveMark> {
        let marks =
            build_paper_watch_live_marks_with_entry_book(candidates, ticks, &mut self.entry_book);
        let mut new_marks = Vec::new();
        for mark in marks {
            if !self
                .seen_live_mark_ids
                .insert(mark.paper_watch_live_mark_id.clone())
            {
                continue;
            }
            self.marks_by_candidate
                .entry(mark.paper_watch_candidate_id.clone())
                .or_default()
                .push(mark.clone());
            new_marks.push(mark);
        }
        new_marks
    }

    pub fn snapshot(
        &self,
        observer_run_id: &str,
        iteration: usize,
        created_at_ms: i64,
        candidates: &[PaperWatchCandidate],
        new_marks: &[PaperWatchLiveMark],
    ) -> PaperWatchObserverSnapshot {
        let active_candidates = active_candidates(candidates, created_at_ms);
        let active_symbols = active_candidates
            .iter()
            .map(|candidate| candidate.symbol_canonical.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let all_marks = self
            .marks_by_candidate
            .values()
            .flat_map(|marks| marks.iter())
            .collect::<Vec<_>>();

        PaperWatchObserverSnapshot {
            schema_version: PAPER_WATCH_OBSERVER_SNAPSHOT_SCHEMA_VERSION.to_owned(),
            observer_run_id: observer_run_id.to_owned(),
            iteration,
            created_at_ms,
            active_candidate_count: active_candidates.len(),
            active_symbols,
            restored_live_mark_count: self
                .seen_live_mark_ids
                .len()
                .saturating_sub(new_marks.len()),
            new_live_mark_count: new_marks.len(),
            total_live_mark_count: self.seen_live_mark_ids.len(),
            lifecycle_counts: count_by(all_marks.iter().map(|mark| mark.lifecycle_state.as_str())),
            venue_counts: count_by(all_marks.iter().map(|mark| mark.venue.as_str())),
            net_return_bps: summarize_returns(all_marks.iter().map(|mark| mark.net_return_bps)),
            candidate_summaries: active_candidates
                .iter()
                .map(|candidate| self.candidate_summary(candidate, created_at_ms))
                .collect(),
            safety: PaperWatchObserverSafety {
                paper_only: true,
                live_enabled: false,
                order_execution_enabled: false,
                execution_approval_emitted: false,
            },
        }
    }

    fn restore_entry_from_mark(&mut self, mark: &PaperWatchLiveMark) {
        let quote_asset = mark
            .reason_codes
            .iter()
            .find_map(|reason| reason.strip_prefix("quote_asset="))
            .unwrap_or("");
        self.entry_book.restore_entry(
            &mark.paper_watch_candidate_id,
            &mark.venue,
            quote_asset,
            mark.entry_mark_price,
        );
    }

    fn candidate_summary(
        &self,
        candidate: &PaperWatchCandidate,
        now_ms: i64,
    ) -> PaperWatchObserverCandidateSummary {
        let marks = self
            .marks_by_candidate
            .get(&candidate.paper_watch_candidate_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let latest = marks.iter().max_by(|left, right| {
            (left.exchange_timestamp_ms, left.ingest_timestamp_ms)
                .cmp(&(right.exchange_timestamp_ms, right.ingest_timestamp_ms))
        });
        let returns = marks
            .iter()
            .map(|mark| mark.net_return_bps)
            .collect::<Vec<_>>();
        let max_return_bps = finite_max(returns.iter().copied());
        let min_return_bps = finite_min(returns.iter().copied());
        let latest_return_bps = latest.map(|mark| mark.net_return_bps);
        let lifecycle_state = latest
            .map(|mark| mark.lifecycle_state.clone())
            .unwrap_or_else(|| "waiting_for_live_tick".to_owned());
        let holding_elapsed_ms = now_ms.saturating_sub(candidate.created_at_ms).max(0);
        PaperWatchObserverCandidateSummary {
            paper_watch_candidate_id: candidate.paper_watch_candidate_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            candidate_lifecycle_key: candidate.candidate_lifecycle_key.clone(),
            symbol_canonical: candidate.symbol_canonical.clone(),
            source_research_run_id: candidate.source_research_run_id.clone(),
            target_max_holding_hours: candidate.target_max_holding_hours,
            absolute_max_holding_hours: candidate.absolute_max_holding_hours,
            holding_elapsed_ms,
            mark_count: marks.len(),
            venues: marks
                .iter()
                .map(|mark| mark.venue.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            latest_return_bps,
            min_return_bps,
            max_return_bps,
            max_drawdown_bps: max_drawdown_bps(&returns),
            lifecycle_state: lifecycle_state.clone(),
            observer_verdict: observer_verdict(
                marks.len(),
                latest_return_bps,
                min_return_bps,
                max_return_bps,
                &lifecycle_state,
            ),
            safety: PaperWatchSafety {
                paper_only: true,
                live_enabled: false,
                order_execution_enabled: false,
                execution_approval_emitted: false,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaperWatchObserverSnapshot {
    pub schema_version: String,
    pub observer_run_id: String,
    pub iteration: usize,
    pub created_at_ms: i64,
    pub active_candidate_count: usize,
    pub active_symbols: Vec<String>,
    pub restored_live_mark_count: usize,
    pub new_live_mark_count: usize,
    pub total_live_mark_count: usize,
    pub lifecycle_counts: BTreeMap<String, usize>,
    pub venue_counts: BTreeMap<String, usize>,
    pub net_return_bps: PaperWatchObserverReturnSummary,
    pub candidate_summaries: Vec<PaperWatchObserverCandidateSummary>,
    pub safety: PaperWatchObserverSafety,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaperWatchObserverCandidateSummary {
    pub paper_watch_candidate_id: String,
    pub candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub source_research_run_id: String,
    pub target_max_holding_hours: u32,
    pub absolute_max_holding_hours: u32,
    pub holding_elapsed_ms: i64,
    pub mark_count: usize,
    pub venues: Vec<String>,
    pub latest_return_bps: Option<f64>,
    pub min_return_bps: Option<f64>,
    pub max_return_bps: Option<f64>,
    pub max_drawdown_bps: Option<f64>,
    pub lifecycle_state: String,
    pub observer_verdict: String,
    pub safety: PaperWatchSafety,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaperWatchObserverReturnSummary {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub average: Option<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaperWatchObserverSafety {
    pub paper_only: bool,
    pub live_enabled: bool,
    pub order_execution_enabled: bool,
    pub execution_approval_emitted: bool,
}

pub fn active_candidates(
    candidates: &[PaperWatchCandidate],
    now_ms: i64,
) -> Vec<PaperWatchCandidate> {
    candidates
        .iter()
        .filter(|candidate| {
            candidate.safety.paper_only
                && !candidate.safety.live_enabled
                && !candidate.safety.order_execution_enabled
                && !candidate.safety.execution_approval_emitted
                && now_ms
                    < candidate.created_at_ms
                        + i64::from(candidate.absolute_max_holding_hours) * 60 * 60 * 1000
        })
        .cloned()
        .collect()
}

fn observer_verdict(
    mark_count: usize,
    latest_return_bps: Option<f64>,
    min_return_bps: Option<f64>,
    max_return_bps: Option<f64>,
    lifecycle_state: &str,
) -> String {
    if mark_count == 0 {
        return "WAIT_FOR_LIVE_TICK".to_owned();
    }
    if min_return_bps.is_some_and(|value| value <= -200.0) {
        return "RISK_REVIEW".to_owned();
    }
    if matches!(
        lifecycle_state,
        "target_holding_window_open" | "force_flat_due"
    ) && latest_return_bps.is_some_and(|value| value > 0.0)
        && min_return_bps.is_some_and(|value| value > -100.0)
    {
        return "SHADOW_REVIEW_CANDIDATE".to_owned();
    }
    if max_return_bps.is_some_and(|value| value > 0.0) {
        return "WATCHING_POSITIVE".to_owned();
    }
    "WATCHING".to_owned()
}

fn summarize_returns<I>(values: I) -> PaperWatchObserverReturnSummary
where
    I: Iterator<Item = f64>,
{
    let values = values.filter(|value| value.is_finite()).collect::<Vec<_>>();
    let average = if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    };
    PaperWatchObserverReturnSummary {
        min: finite_min(values.iter().copied()),
        max: finite_max(values.iter().copied()),
        average,
    }
}

fn count_by<'a, I>(values: I) -> BTreeMap<String, usize>
where
    I: Iterator<Item = &'a str>,
{
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_owned()).or_insert(0) += 1;
    }
    counts
}

fn finite_min<I>(values: I) -> Option<f64>
where
    I: Iterator<Item = f64>,
{
    values
        .filter(|value| value.is_finite())
        .min_by(|left, right| left.total_cmp(right))
}

fn finite_max<I>(values: I) -> Option<f64>
where
    I: Iterator<Item = f64>,
{
    values
        .filter(|value| value.is_finite())
        .max_by(|left, right| left.total_cmp(right))
}

fn max_drawdown_bps(values: &[f64]) -> Option<f64> {
    let mut peak: Option<f64> = None;
    let mut max_drawdown: Option<f64> = None;
    for value in values.iter().copied().filter(|value| value.is_finite()) {
        peak = Some(peak.map_or(value, |current| current.max(value)));
        if let Some(peak_value) = peak {
            let drawdown = peak_value - value;
            max_drawdown = Some(max_drawdown.map_or(drawdown, |current| current.max(drawdown)));
        }
    }
    max_drawdown
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        PaperExpectedCostProfile, PaperExpectedRiskProfile, PaperWatchReplaySampleSummary,
        ResearchBias, SurvivalBand,
    };

    #[test]
    fn observer_state_dedupes_marks_and_summarizes_candidates() {
        let candidate = candidate("watch_1", "XRP");
        let mark = mark("mark_1", &candidate, "binance", 100.0);
        let mut state = PaperWatchObserverState::default();

        state.restore_marks(&[mark.clone(), mark]);
        let snapshot = state.snapshot("observer_1", 1, 2_000, &[candidate], &[]);

        assert_eq!(snapshot.total_live_mark_count, 1);
        assert_eq!(snapshot.candidate_summaries.len(), 1);
        assert_eq!(
            snapshot.candidate_summaries[0].observer_verdict,
            "WATCHING_POSITIVE"
        );
        assert!(!snapshot.safety.order_execution_enabled);
    }

    #[test]
    fn observer_marks_target_window_as_shadow_review_candidate() {
        let mut candidate = candidate("watch_1", "XRP");
        candidate.created_at_ms = 0;
        candidate.target_max_holding_hours = 1;
        let mut mark = mark("mark_1", &candidate, "binance", 50.0);
        mark.lifecycle_state = "target_holding_window_open".to_owned();
        let mut state = PaperWatchObserverState::default();

        state.restore_marks(&[mark]);
        let snapshot = state.snapshot("observer_1", 1, 2 * 60 * 60 * 1000, &[candidate], &[]);

        assert_eq!(
            snapshot.candidate_summaries[0].observer_verdict,
            "SHADOW_REVIEW_CANDIDATE"
        );
    }

    #[test]
    fn active_candidates_exclude_expired_and_unsafe_candidates() {
        let safe = candidate("watch_safe", "DOGE");
        let mut expired = candidate("watch_expired", "XRP");
        expired.created_at_ms = 0;
        expired.absolute_max_holding_hours = 1;
        let mut live_enabled = candidate("watch_live", "TON");
        live_enabled.safety.live_enabled = true;
        let mut order_enabled = candidate("watch_order", "ZEC");
        order_enabled.safety.order_execution_enabled = true;

        let active = active_candidates(
            &[safe.clone(), expired, live_enabled, order_enabled],
            2 * 60 * 60 * 1000,
        );

        assert_eq!(active.len(), 1);
        assert_eq!(
            active[0].paper_watch_candidate_id,
            safe.paper_watch_candidate_id
        );
    }

    #[test]
    fn observer_snapshot_marks_no_tick_and_risk_review_states() {
        let waiting = candidate("watch_waiting", "PAXG");
        let risky = candidate("watch_risky", "PENGU");
        let mut risky_mark = mark("mark_risky", &risky, "upbit", -250.0);
        risky_mark.lifecycle_state = "watching".to_owned();
        let mut state = PaperWatchObserverState::default();

        state.restore_marks(&[risky_mark]);
        let snapshot = state.snapshot(
            "observer_1",
            1,
            2_000,
            &[waiting.clone(), risky.clone()],
            &[],
        );

        let by_symbol = snapshot
            .candidate_summaries
            .iter()
            .map(|summary| {
                (
                    summary.symbol_canonical.as_str(),
                    summary.observer_verdict.as_str(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(by_symbol["PAXG"], "WAIT_FOR_LIVE_TICK");
        assert_eq!(by_symbol["PENGU"], "RISK_REVIEW");
    }

    fn candidate(id: &str, symbol: &str) -> PaperWatchCandidate {
        PaperWatchCandidate {
            paper_watch_candidate_id: id.to_owned(),
            candidate_id: format!("cand_{id}"),
            candidate_lifecycle_key: format!("cand_{id}:v1"),
            symbol_canonical: symbol.to_owned(),
            source_research_run_id: "research_run_001".to_owned(),
            source_research_packet_id: "packet_001".to_owned(),
            source_research_bias: ResearchBias::RetestBias,
            historical_survival_band: SurvivalBand::Stable,
            admission_reason_codes: vec!["retest_positive_watch_admitted".to_owned()],
            blocked_promotion_reason_codes: vec!["needs_forward_observation".to_owned()],
            replay_sample_summary: PaperWatchReplaySampleSummary {
                research_aggregate_key: "agg_001".to_owned(),
                replay_run_count: 10,
                completed_count: 5,
                positive_net_count: 3,
                non_positive_net_count: 2,
                missing_market_replay_data_count: 0,
                insufficient_evidence_count: 0,
                effective_completed_sample_weight: 5.0,
                weighted_mean_net_after_cost_bps: Some(10.0),
                weighted_profit_factor_ppm: Some(1_100_000),
            },
            expected_cost_profile: PaperExpectedCostProfile {
                fee_model_version: "fee".to_owned(),
                slippage_model_version: "slippage".to_owned(),
                estimated_cost_bps: Some(8.0),
                cost_stressed_mean_net_after_cost_bps: Some(2.0),
            },
            expected_risk_profile: PaperExpectedRiskProfile {
                survival_band: SurvivalBand::Stable,
                max_drawdown_band: "low".to_owned(),
                positive_net_count: 3,
                non_positive_net_count: 2,
            },
            target_max_holding_hours: 24,
            absolute_max_holding_hours: 72,
            force_flat_policy: "paper_watch_only_no_order_execution".to_owned(),
            paper_start_recommendation: "start_forward_paper_watch".to_owned(),
            safety: PaperWatchSafety {
                paper_only: true,
                live_enabled: false,
                order_execution_enabled: false,
                execution_approval_emitted: false,
            },
            created_at_ms: 1_000,
            schema_version: "paper_watch_candidate_v1".to_owned(),
        }
    }

    fn mark(
        id: &str,
        candidate: &PaperWatchCandidate,
        venue: &str,
        net_return_bps: f64,
    ) -> PaperWatchLiveMark {
        PaperWatchLiveMark {
            paper_watch_live_mark_id: id.to_owned(),
            paper_watch_candidate_id: candidate.paper_watch_candidate_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            candidate_lifecycle_key: candidate.candidate_lifecycle_key.clone(),
            symbol_canonical: candidate.symbol_canonical.clone(),
            source_research_run_id: candidate.source_research_run_id.clone(),
            source_market_live_event_id: format!("event_{id}"),
            venue: venue.to_owned(),
            mark_source: "market_live_tick".to_owned(),
            marked_at_ms: 2_000,
            exchange_timestamp_ms: 2_000,
            ingest_timestamp_ms: 2_010,
            holding_elapsed_ms: 1_000,
            entry_mark_price: 1.0,
            current_mark_price: 1.0 + net_return_bps / 10_000.0,
            net_return_bps,
            target_max_holding_hours: candidate.target_max_holding_hours,
            absolute_max_holding_hours: candidate.absolute_max_holding_hours,
            lifecycle_state: "watching".to_owned(),
            reason_codes: vec![
                "paper_watch_live_mark".to_owned(),
                format!("venue={venue}"),
                "quote_asset=USDT".to_owned(),
            ],
            safety: PaperWatchSafety {
                paper_only: true,
                live_enabled: false,
                order_execution_enabled: false,
                execution_approval_emitted: false,
            },
            schema_version: "paper_watch_live_mark_v1".to_owned(),
        }
    }
}
