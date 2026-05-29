use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchRunStatus {
    Completed,
    Partial,
    InvalidInput,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchBias {
    PruneBias,
    RetestBias,
    PromoteToShadowBias,
    PromoteToPaperBias,
}

impl ResearchBias {
    pub fn report_key(&self) -> &'static str {
        match self {
            Self::PruneBias => "PRUNE_BIAS",
            Self::RetestBias => "RETEST_BIAS",
            Self::PromoteToShadowBias => "PROMOTE_TO_SHADOW_BIAS",
            Self::PromoteToPaperBias => "PROMOTE_TO_PAPER_BIAS",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplayRunStatus {
    Completed,
    InvalidInput,
    MissingMarketReplayData,
    InsufficientEvidence,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityFilterStatus {
    NotRequired,
    Passed,
    Failed,
    NotMaterialized,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurvivalBand {
    Fragile,
    Conditional,
    Stable,
    Exceptional,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchAggregateRegistryStage {
    Pruned,
    Retest,
    ShadowCandidate,
    PaperCandidateBias,
}
