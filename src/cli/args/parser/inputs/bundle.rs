use super::super::*;

pub(super) fn apply_bundle_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--input-bundle-file" => {
            args.input_bundle_file = Some(absolute_path_arg(
                values.next(),
                "--input-bundle-file requires an absolute path",
            )?);
        }
        "--input-bundle-s3-bucket" => {
            args.input_bundle_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--input-bundle-s3-bucket requires a value",
            )?);
        }
        "--input-bundle-s3-key" => {
            args.input_bundle_s3_key = Some(non_empty_arg(
                values.next(),
                "--input-bundle-s3-key requires a value",
            )?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
