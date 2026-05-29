mod missing;
mod read;
mod validation;
mod write;

pub(super) use missing::is_missing_market_artifact;
pub(super) use read::get_object_bytes;
pub(super) use validation::{validate_research_input_manifest_s3_key, validate_s3_location};
pub(super) use write::{
    PutIfAbsentResult, put_jsonl_object, put_object_bytes_if_absent, put_object_json,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    #[test]
    fn missing_market_artifact_errors_are_skippable() {
        let error = AppError::AwsNotFound("s3://bucket/missing.json".to_owned());

        assert!(is_missing_market_artifact(&error));
    }

    #[test]
    fn non_not_found_aws_errors_are_not_skippable() {
        let error = AppError::Aws("AccessDenied".to_owned());

        assert!(!is_missing_market_artifact(&error));
    }

    #[test]
    fn validates_s3_location_rejects_empty_values() {
        assert!(validate_s3_location("", "key.json", "test").is_err());
        assert!(validate_s3_location("bucket", "   ", "test").is_err());
    }

    #[test]
    fn validates_s3_location_rejects_period_only_key_segments() {
        let dot = validate_s3_location("bucket", "prefix/./object.json", "test")
            .expect_err("period-only segment must be rejected");
        let dotdot = validate_s3_location("bucket", "prefix/../object.json", "test")
            .expect_err("period-only parent segment must be rejected");

        assert!(dot.to_string().contains("period-only"));
        assert!(dotdot.to_string().contains("period-only"));
    }

    #[test]
    fn validates_s3_location_allows_safe_dot_in_key_names() {
        validate_s3_location("bucket", "prefix/file.name.json", "test")
            .expect("safe dot key is accepted");
    }

    #[test]
    fn validates_research_input_manifest_key_contract() {
        validate_research_input_manifest_s3_key("research-input-manifest/run/manifest.json")
            .expect("json manifest key is valid");
        validate_research_input_manifest_s3_key("/research-input-manifest/run/manifest.jsonl")
            .expect("leading slash is normalized for validation");

        assert!(
            validate_research_input_manifest_s3_key("other/run/manifest.json")
                .unwrap_err()
                .to_string()
                .contains("research-input-manifest/")
        );
    }
}
