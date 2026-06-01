use super::super::*;

pub(super) fn apply_market_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--market-feature-delta-file" => {
            args.market_feature_delta_file = Some(absolute_path_arg(
                values.next(),
                "--market-feature-delta-file requires an absolute path",
            )?);
        }
        "--market-regime-context-file" => {
            args.market_regime_context_file = Some(absolute_path_arg(
                values.next(),
                "--market-regime-context-file requires an absolute path",
            )?);
        }
        "--market-l1-s3-bucket" => {
            args.market_l1_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--market-l1-s3-bucket requires a value",
            )?);
        }
        "--market-feature-delta-s3-key" => {
            args.market_feature_delta_s3_keys.push(non_empty_arg(
                values.next(),
                "--market-feature-delta-s3-key requires a value",
            )?);
        }
        "--market-regime-context-s3-key" => {
            args.market_regime_context_s3_keys.push(non_empty_arg(
                values.next(),
                "--market-regime-context-s3-key requires a value",
            )?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
