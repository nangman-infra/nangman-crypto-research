const PPM_DENOMINATOR: f64 = 1_000_000.0;
const MAX_REPORTED_PROFIT_FACTOR_PPM: u64 = 9_999_999_999;

pub(super) fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

pub(super) fn weighted_mean(weighted_sum: f64, weight_sum: f64) -> Option<f64> {
    (weight_sum > 0.0).then(|| weighted_sum / weight_sum)
}

pub(super) fn ratio_ppm(numerator: usize, denominator: usize) -> Option<u64> {
    if denominator == 0 {
        return None;
    }
    Some(((numerator as f64 / denominator as f64) * PPM_DENOMINATOR).round() as u64)
}

pub(super) fn weighted_ratio_ppm(numerator_weight: f64, denominator_weight: f64) -> Option<u64> {
    if denominator_weight <= 0.0 {
        return None;
    }
    Some(((numerator_weight / denominator_weight) * PPM_DENOMINATOR).round() as u64)
}

pub(super) fn profit_factor_ppm(
    gross_positive_net_bps: f64,
    gross_negative_net_bps_abs: f64,
) -> Option<u64> {
    if gross_negative_net_bps_abs > 0.0 {
        return Some(
            ((gross_positive_net_bps / gross_negative_net_bps_abs) * PPM_DENOMINATOR).round()
                as u64,
        );
    }
    (gross_positive_net_bps > 0.0).then_some(MAX_REPORTED_PROFIT_FACTOR_PPM)
}
