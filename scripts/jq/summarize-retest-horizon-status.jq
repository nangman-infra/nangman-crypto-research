include "summarize-retest-horizon-status-checkpoint";
include "summarize-retest-horizon-status-coverage";

($driver_summary_input[0] // null) as $driver_summary
| ($driver_manifest_summary_input[0] // null) as $driver_manifest_summary
| (.horizon_rows // []) as $rows
| ($driver_summary // {}) as $driver
| ($driver_manifest_summary // {}) as $manifest_summary
| ($driver.manifest.latest_universe // $manifest_summary.latest_universe // {}) as $latest_universe
| (.latest_l1_as_of_ms // null) as $latest_l1_as_of_ms
| retest_horizon_status_checkpoint(
    $rows;
    $driver;
    $generated_at;
    $plan_file;
    $driver_summary_file;
    $latest_l1_as_of_ms
  )
| with_research_factory_coverage($rows; $driver; $latest_universe)
