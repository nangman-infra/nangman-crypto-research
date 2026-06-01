use super::*;

pub(super) enum ArtifactLocation {
    Local(PathBuf),
    S3 { bucket: String, key: String },
}

pub(super) fn artifact_location(artifact_ref: &ResearchArtifactRef) -> AppResult<ArtifactLocation> {
    ArtifactLocation::from_uri(&artifact_ref.uri)
}

pub(super) fn validate_artifact_ref_location(artifact_ref: &ResearchArtifactRef) -> AppResult<()> {
    match artifact_location(artifact_ref)? {
        ArtifactLocation::Local(path) => {
            crate::path_validation::validate_config_absolute_path(&path, "manifest artifact uri")
                .map_err(|error| {
                    AppError::config(format!(
                        "manifest artifact uri must be an absolute path or s3 URI: {}; {error}",
                        artifact_ref.uri
                    ))
                })
        }
        ArtifactLocation::S3 { .. } => Ok(()),
    }
}

impl ArtifactLocation {
    fn from_uri(uri: &str) -> AppResult<Self> {
        let trimmed = uri.trim();
        if trimmed.is_empty() {
            return Err(AppError::config("manifest artifact uri must not be empty"));
        }
        if let Some((bucket, key)) = parse_s3_uri(trimmed) {
            return Ok(Self::S3 { bucket, key });
        }
        Ok(Self::Local(PathBuf::from(trimmed)))
    }
}
