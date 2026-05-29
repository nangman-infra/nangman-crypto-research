use super::state::AggregateAccumulator;
use crate::gate::sample::CompletedSample;
use crate::model::{LiquidityFilterStatus, ReplayRun};

impl AggregateAccumulator {
    pub(in crate::gate::aggregate) fn add_completed_net_sample(
        &mut self,
        run: &ReplayRun,
        value: f64,
        sample_weight: f64,
    ) {
        self.net_after_costs.push(value);
        self.effective_completed_sample_weight += sample_weight;
        self.weighted_net_after_cost_sum += value * sample_weight;
        self.completed_samples.push(CompletedSample {
            window_start_ms: run.window_start_ms,
            net_after_cost_bps: value,
            estimated_cost_bps: run.result_summary.estimated_cost_bps,
            market_regime_labels: run.result_summary.market_regime_labels.clone(),
        });
        if value > 0.0 {
            self.positive_net_count += 1;
            self.gross_positive_net_bps += value;
            self.weighted_positive_net_bps += value * sample_weight;
            self.weighted_positive_sample_weight += sample_weight;
        } else {
            self.non_positive_net_count += 1;
            self.gross_negative_net_bps_abs += value.abs();
            self.weighted_negative_net_bps_abs += value.abs() * sample_weight;
        }
    }

    pub(in crate::gate::aggregate) fn add_liquidity_filter_summary(&mut self, run: &ReplayRun) {
        let Some(summary) = run.result_summary.liquidity_filter_summary.as_ref() else {
            return;
        };
        match summary.status {
            LiquidityFilterStatus::Passed => {
                self.liquidity_filter_materialized_count += 1;
                self.liquidity_filter_passed_count += 1;
            }
            LiquidityFilterStatus::Failed => {
                self.liquidity_filter_materialized_count += 1;
                self.liquidity_filter_failed_count += 1;
            }
            LiquidityFilterStatus::NotMaterialized | LiquidityFilterStatus::NotRequired => {}
        }
    }
}
