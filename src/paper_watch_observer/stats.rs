use super::types::PaperWatchObserverReturnSummary;
use std::collections::BTreeMap;

pub(super) fn summarize_returns<I>(values: I) -> PaperWatchObserverReturnSummary
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

pub(super) fn count_by<'a, I>(values: I) -> BTreeMap<String, usize>
where
    I: Iterator<Item = &'a str>,
{
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_owned()).or_insert(0) += 1;
    }
    counts
}

pub(super) fn finite_min<I>(values: I) -> Option<f64>
where
    I: Iterator<Item = f64>,
{
    values
        .filter(|value| value.is_finite())
        .min_by(|left, right| left.total_cmp(right))
}

pub(super) fn finite_max<I>(values: I) -> Option<f64>
where
    I: Iterator<Item = f64>,
{
    values
        .filter(|value| value.is_finite())
        .max_by(|left, right| left.total_cmp(right))
}

pub(super) fn max_drawdown_bps(values: &[f64]) -> Option<f64> {
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
