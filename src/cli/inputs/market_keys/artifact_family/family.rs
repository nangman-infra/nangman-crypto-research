use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MarketArtifactFamily {
    FeatureDelta,
    RegimeContext,
}

impl MarketArtifactFamily {
    pub(super) fn manual_keys(self, args: &Args) -> &[String] {
        match self {
            Self::FeatureDelta => &args.market_feature_delta_s3_keys,
            Self::RegimeContext => &args.market_regime_context_s3_keys,
        }
    }

    pub(super) fn key_from_run_id(self, run_id: &str) -> String {
        match self {
            Self::FeatureDelta => format!("market_feature_delta/run_id={run_id}/delta.json"),
            Self::RegimeContext => {
                format!("market_regime_context/run_id={run_id}/context.json")
            }
        }
    }
}
