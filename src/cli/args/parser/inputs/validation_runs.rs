use super::super::*;

pub(super) fn apply_validation_run_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--oss-adapter-run-file" => {
            args.oss_adapter_run_files.push(absolute_path_arg(
                values.next(),
                "--oss-adapter-run-file requires an absolute path",
            )?);
        }
        "--shadow-validation-run-file" => {
            args.shadow_validation_run_files.push(absolute_path_arg(
                values.next(),
                "--shadow-validation-run-file requires an absolute path",
            )?);
        }
        "--oss-adapter-run-s3-bucket" => {
            args.oss_adapter_run_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--oss-adapter-run-s3-bucket requires a value",
            )?);
        }
        "--oss-adapter-run-s3-key" => {
            args.oss_adapter_run_s3_keys.push(non_empty_arg(
                values.next(),
                "--oss-adapter-run-s3-key requires a value",
            )?);
        }
        "--shadow-validation-run-s3-bucket" => {
            args.shadow_validation_run_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--shadow-validation-run-s3-bucket requires a value",
            )?);
        }
        "--shadow-validation-run-s3-key" => {
            args.shadow_validation_run_s3_keys.push(non_empty_arg(
                values.next(),
                "--shadow-validation-run-s3-key requires a value",
            )?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
