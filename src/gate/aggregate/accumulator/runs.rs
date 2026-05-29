use super::state::AggregateAccumulator;
use crate::gate::sample::{DecayBand, ReplaySampleWeight, replay_sample_weight};
use crate::model::{IntelCandidateEvidenceBundle, ReplayRun, ReplayRunStatus, ResearchGatePolicy};

impl AggregateAccumulator {
    pub(in crate::gate::aggregate) fn add_run(
        &mut self,
        run: &ReplayRun,
        bundle: Option<&IntelCandidateEvidenceBundle>,
        policy: &ResearchGatePolicy,
        gate_as_of_ms: i64,
    ) {
        self.replay_run_count += 1;
        self.research_partition_keys
            .insert(run.research_partition_key.clone());
        self.source_candidate_ids
            .insert(run.source_candidate_id.clone());
        self.source_candidate_lifecycle_keys
            .insert(run.source_candidate_lifecycle_key.clone());
        self.apply_bundle_requirements(bundle);

        let sample_weight = replay_sample_weight(gate_as_of_ms, run.window_end_ms, policy);
        if sample_weight.band == DecayBand::Expired {
            self.expired_replay_run_count += 1;
            if run.result_summary.status == ReplayRunStatus::Completed
                && run.result_summary.net_after_cost_bps.is_some()
            {
                self.expired_completed_count += 1;
            }
            return;
        }

        self.active_replay_run_count += 1;
        self.active_replay_windows
            .insert(format!("{}-{}", run.window_start_ms, run.window_end_ms));
        self.cost_estimates
            .push(run.result_summary.estimated_cost_bps);
        self.market_regime_labels
            .extend(run.result_summary.market_regime_labels.iter().cloned());

        match run.result_summary.status {
            ReplayRunStatus::Completed => self.add_completed_run(run, sample_weight),
            ReplayRunStatus::InvalidInput => {
                self.invalid_input_count += 1;
            }
            ReplayRunStatus::MissingMarketReplayData => {
                self.missing_market_replay_data_count += 1;
            }
            ReplayRunStatus::InsufficientEvidence => {
                self.insufficient_evidence_count += 1;
            }
        }
    }

    fn add_completed_run(&mut self, run: &ReplayRun, sample_weight: ReplaySampleWeight) {
        self.completed_count += 1;
        if self.liquidity_filter_required {
            self.add_liquidity_filter_summary(run);
        }
        if let Some(value) = run.result_summary.raw_return_bps {
            self.raw_returns.push(value);
        }
        if let Some(value) = run.result_summary.btc_adjusted_return_bps {
            self.btc_adjusted_returns.push(value);
        }
        if let Some(value) = run.result_summary.net_after_cost_bps {
            self.add_completed_net_sample(run, value, sample_weight.weight);
            if sample_weight.band != DecayBand::Fresh {
                self.decayed_completed_count += 1;
            }
        }
    }
}
