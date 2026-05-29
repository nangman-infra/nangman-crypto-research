mod bundle;
mod classification;
mod evidence;
mod market;

pub use bundle::IntelCandidateEvidenceBundle;
pub use classification::{CandidateClass, ConfidenceBand};
pub use evidence::{
    DataQualitySummaryRef, MetricEvidence, SelectedMarketArtifactTrace, SourceIndependenceSummary,
    SymbolResolutionTrace, ValidationRequirements,
};
pub use market::{MarketFeatureDelta, MarketRegimeContext};
