include "summarize-retest-horizon-status-coverage-state";
include "summarize-retest-horizon-status-coverage-major50";
include "summarize-retest-horizon-status-coverage-gaps";

def with_research_factory_coverage($rows; $driver; $latest_universe):
  research_factory_coverage_state($rows; $driver; $latest_universe) as $coverage
  | . + {
      verdict:.next_decision.verdict,
      selected_symbols:$coverage.candidate_symbols,
      next_action_counts:.horizon_summary.next_action_counts,
      major50_state:research_factory_major50_state($coverage; $driver; $latest_universe),
      research_factory_progression:research_factory_progression($coverage; $latest_universe; .stage_state),
      coverage_gaps:research_factory_coverage_gaps($coverage; .stage_state),
      research_factory_gap_summary:research_factory_gap_summary($coverage; $driver; $latest_universe; .next_decision)
    };
