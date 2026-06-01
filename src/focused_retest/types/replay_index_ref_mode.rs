use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoricalReplayIndexRefMode {
    Auto,
    Always,
    Never,
}

impl HistoricalReplayIndexRefMode {
    pub fn parse(raw: &str) -> AppResult<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "true" | "always" => Ok(Self::Always),
            "false" | "never" => Ok(Self::Never),
            other => Err(AppError::config(format!(
                "focused retest historical replay index ref mode must be auto, true, or false; got {other}"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Always => "true",
            Self::Never => "false",
        }
    }

    pub(in crate::focused_retest) fn should_carry(self, actions: &[String]) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => actions
                .iter()
                .any(|action| action == "accumulate_completed_native_replay_samples"),
        }
    }
}
