use super::super::*;

pub(super) fn apply_candidate_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--paper-watch-candidate-file" => {
            args.paper_watch_candidate_file = Some(absolute_path_arg(
                values.next(),
                "--paper-watch-candidate-file requires an absolute path",
            )?);
        }
        "--paper-watch-candidate-s3-bucket" => {
            args.paper_watch_candidate_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--paper-watch-candidate-s3-bucket requires a value",
            )?);
        }
        "--paper-watch-candidate-s3-key" => {
            args.paper_watch_candidate_s3_key = Some(non_empty_arg(
                values.next(),
                "--paper-watch-candidate-s3-key requires a value",
            )?);
        }
        "--paper-watch-candidate-s3-prefix" => {
            args.paper_watch_candidate_s3_prefix = non_empty_arg(
                values.next(),
                "--paper-watch-candidate-s3-prefix requires a value",
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
