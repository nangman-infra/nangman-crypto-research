use crate::model::{
    IntelCandidateEvidenceBundle, LiquidityFilterStatus, LiquidityFilterSummary, MarketFeatureDelta,
};

pub(super) fn liquidity_filter_summary(
    bundle: &IntelCandidateEvidenceBundle,
    matched: &[&MarketFeatureDelta],
) -> Option<LiquidityFilterSummary> {
    if !bundle.validation_requirements.include_liquidity_filter {
        return Some(LiquidityFilterSummary {
            status: LiquidityFilterStatus::NotRequired,
            reason_codes: Vec::new(),
            observed_metric_count: 0,
            positive_volume_metric_count: 0,
        });
    }

    let liquidity_metrics = matched
        .iter()
        .filter(|delta| is_liquidity_metric(delta))
        .collect::<Vec<_>>();
    let positive_volume_metric_count = liquidity_metrics
        .iter()
        .filter(|delta| liquidity_metric_is_positive(delta))
        .count();

    if liquidity_metrics.is_empty() {
        return Some(LiquidityFilterSummary {
            status: LiquidityFilterStatus::NotMaterialized,
            reason_codes: vec!["liquidity_filter_not_materialized".to_owned()],
            observed_metric_count: 0,
            positive_volume_metric_count,
        });
    }

    if positive_volume_metric_count == 0 {
        return Some(LiquidityFilterSummary {
            status: LiquidityFilterStatus::Failed,
            reason_codes: vec!["liquidity_filter_no_positive_volume_observed".to_owned()],
            observed_metric_count: liquidity_metrics.len(),
            positive_volume_metric_count,
        });
    }

    Some(LiquidityFilterSummary {
        status: LiquidityFilterStatus::Passed,
        reason_codes: vec!["liquidity_filter_positive_volume_observed".to_owned()],
        observed_metric_count: liquidity_metrics.len(),
        positive_volume_metric_count,
    })
}

fn is_liquidity_metric(delta: &MarketFeatureDelta) -> bool {
    delta.metric_name.eq_ignore_ascii_case("trade_volume")
        || delta.volume_change_same_window.is_some()
}

fn liquidity_metric_is_positive(delta: &MarketFeatureDelta) -> bool {
    is_liquidity_metric(delta) && delta.value_now.is_finite() && delta.value_now > 0.0
}
