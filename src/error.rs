use std::fmt::{Display, Formatter};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Aws(String),
    AwsNotFound(String),
    Config(String),
    Io(String),
    Json(String),
    Validation(String),
}

impl AppError {
    pub fn aws(message: impl Into<String>) -> Self {
        Self::Aws(message.into())
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Aws(message) => write!(formatter, "aws error: {message}"),
            Self::AwsNotFound(message) => write!(formatter, "aws not found: {message}"),
            Self::Config(message) => write!(formatter, "config error: {message}"),
            Self::Io(message) => write!(formatter, "io error: {message}"),
            Self::Json(message) => write!(formatter, "json error: {message}"),
            Self::Validation(message) => write!(formatter, "validation error: {message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aws_not_found_display_is_specific() {
        let error = AppError::AwsNotFound("s3://bucket/key.json".to_owned());

        assert_eq!(error.to_string(), "aws not found: s3://bucket/key.json");
    }
}
