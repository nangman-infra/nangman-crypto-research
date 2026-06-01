use super::super::*;

pub(super) fn apply_manifest_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--input-manifest-file" => {
            args.input_manifest_file = Some(absolute_path_arg(
                values.next(),
                "--input-manifest-file requires an absolute path",
            )?);
        }
        "--input-manifest-s3-bucket" => {
            args.input_manifest_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--input-manifest-s3-bucket requires a value",
            )?);
        }
        "--input-manifest-s3-key" => {
            args.input_manifest_s3_key = Some(non_empty_arg(
                values.next(),
                "--input-manifest-s3-key requires a value",
            )?);
        }
        "--research-report-file" => {
            args.research_report_file = Some(absolute_path_arg(
                values.next(),
                "--research-report-file requires an absolute path",
            )?);
        }
        "--research-report-s3-bucket" => {
            args.research_report_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--research-report-s3-bucket requires a value",
            )?);
        }
        "--research-report-s3-key" => {
            args.research_report_s3_key = Some(non_empty_arg(
                values.next(),
                "--research-report-s3-key requires a value",
            )?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
