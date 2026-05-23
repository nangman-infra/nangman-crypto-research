# Research App

`research-app`은 `intel-candidate-app`이 만든 `intel_candidate_evidence_bundle_v1`을 읽어 research admission, native replay, deterministic report를 수행한다.

초기 버전은 외부 adapter를 실행하지 않는다.

```text
input candidate bundle or research_input_manifest_v1
  -> intake and admissibility
  -> native_replay
  -> replay_run_v1
  -> replay_run_index_v1
  -> historical replay-run merge
  -> deterministic aggregate gate
  -> research_run_report_v1
  -> research_aggregate_registry_record_v1
  -> optional shadow_validation_run_v1
```

## Local Run

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app
cargo run -- \
  --input-bundle-file /Volumes/WD/Developments/nangman-crypto/data/examples/candidate-bundles.jsonl \
  --output-dir /Volumes/WD/Developments/nangman-crypto/data/reports/research-local
```

Optional market inputs:

```bash
--market-feature-delta-file /Volumes/WD/Developments/nangman-crypto/data/examples/market-feature-delta.json
--market-regime-context-file /Volumes/WD/Developments/nangman-crypto/data/examples/market-regime-context.json
```

Optional historical replay input:

```bash
--historical-replay-run-file /Volumes/WD/Developments/nangman-crypto/data/reports/research-local/replay-run/schema=replay_run_v1/dt=2026-05-09/hour=00/research_run_report_id=.../part-000001.jsonl
--historical-replay-run-index-file /Volumes/WD/Developments/nangman-crypto/data/reports/research-local/replay-run-index/schema=replay_run_index_v1/dt=2026-05-09/hour=00/research_run_report_id=.../part-000001.jsonl
```

Batch manifest input:

```json
{
  "schema_version": "research_input_manifest_v1",
  "research_packet_id": "research_packet_2026_05_10_seed",
  "run_scope": "batch_seed_210d",
  "candidate_bundle_refs": [
    { "uri": "/Volumes/WD/Developments/nangman-crypto/data/research/bundle-a.jsonl" },
    { "uri": "s3://nangman-crypto-dev-intel-candidate-<account-suffix>/candidate-evidence-bundle/priority=p0/schema=intel_candidate_evidence_bundle_v1/dt=2026-05-10/hour=00/candidate_id=cand_001/part-000001.jsonl" }
  ],
  "market_feature_delta_refs": [
    { "uri": "s3://nangman-crypto-dev-market-ingest-l1-<account-suffix>/market_feature_delta/run_id=l1_001/delta.json" }
  ],
  "market_regime_context_refs": [
    { "uri": "s3://nangman-crypto-dev-market-ingest-l1-<account-suffix>/market_regime_context/run_id=l1_001/context.json" }
  ],
  "historical_replay_run_index_refs": [
    { "uri": "s3://nangman-crypto-dev-research-<account-suffix>/replay-run-index/schema=replay_run_index_v1/dt=2026-05-09/hour=00/research_run_report_id=.../part-000001.jsonl" }
  ],
  "runtime_budget_policy": {
    "max_candidate_bundle_count": 500,
    "max_market_artifact_ref_count": 2000,
    "max_historical_replay_run_ref_count": 10000,
    "max_replay_run_count": 20000
  }
}
```

Build a local batch manifest from recent S3 candidate bundles before enabling
the dispatcher:

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
scripts/build-research-batch-manifest.sh
```

The builder is local-only. It reads recent `candidate-evidence-bundle/` objects,
adds recent `replay-run-index/` references for historical evidence, writes a
local `research_input_manifest_v1`, and prints a local validation command. It
does not upload the manifest, start ECS, switch the dispatcher, or create S3
research/shadow/paper outputs.

By default the builder uses `RESEARCH_BATCH_UNIVERSE_MODE=current_approved`,
which means candidate bundles are selected only when their symbols are approved
in the latest Market-L1 universe snapshot. This prevents older bundles whose
embedded `approved_universe_symbol` was produced by an older universe policy
from looking like current promotion-safe input. For diagnostic replay only, use
`RESEARCH_BATCH_UNIVERSE_MODE=legacy_retest`; that mode must not be used as
promotion evidence.

In ECS, market replay inputs are loaded from Market-L1 S3. The app first uses
`selected_market_artifacts[].artifact_key` from the candidate bundle, then falls
back to the sibling keys derived from `market_data_quality_summary/run_id=...`.
It also discovers later Market-L1 15-minute replay windows from the candidate's
`forbidden_lookahead_boundary_ms` through the currently materialized horizon, so
native replay can progress after new post-decision market data lands without
manually wiring every delta/context key.

## Current-Approved Batch Driver

Use the current-approved batch driver when candidates exist but the dispatcher
is still in `dry_run`. It closes the local validation loop:

```text
build current-approved manifest
  -> run local research replay
  -> summarize research report
  -> build retest horizon plan
  -> write one driver summary
```

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
scripts/run-current-approved-research-batch.sh
```

The driver discovers the Market-L1 bucket from the research ECS task definition
unless `RESEARCH_MARKET_L1_S3_BUCKET` is set. Outputs are written under
`/tmp/nangman-crypto/research-current-approved-batch/<run-id>/` by default.
Override with absolute paths only:

```bash
RESEARCH_BATCH_DRIVER_ROOT=/tmp/nangman-crypto/research-current-approved-batch \
RESEARCH_BATCH_DRIVER_RUN_ID=research_batch_YYYYMMDDTHHMMSSZ \
scripts/run-current-approved-research-batch.sh
```

The driver is local-output only. It does not upload reports, start ECS tasks,
switch the dispatcher, or create shadow/paper/live artifacts. It refuses
non-`current_approved` universe modes unless
`RESEARCH_BATCH_DRIVER_ALLOW_NON_APPROVED_UNIVERSE=true` is explicitly set for a
diagnostic run. When `AWS_PROFILE` is set and static environment credentials are
absent, it exports short-lived CLI-resolved credentials into the child process
environment so the Rust AWS SDK reads the same authenticated session as the AWS
CLI. The temporary credential file is removed before research replay starts.

ECS input/output environment:

```text
RESEARCH_INPUT_MANIFEST_S3_BUCKET=nangman-crypto-dev-research-<account-suffix>
RESEARCH_INPUT_MANIFEST_S3_KEY=research-input-manifest/schema=research_input_manifest_v1/...
RESEARCH_INPUT_S3_BUCKET=nangman-crypto-dev-intel-candidate-<account-suffix>
RESEARCH_INPUT_S3_KEY=candidate-evidence-bundle/priority=p0/...
RESEARCH_MARKET_L1_S3_BUCKET=nangman-crypto-dev-market-ingest-l1-<account-suffix>
RESEARCH_MARKET_FEATURE_DELTA_S3_KEYS optional comma-separated override
RESEARCH_MARKET_REGIME_CONTEXT_S3_KEYS optional comma-separated override
RESEARCH_HISTORICAL_REPLAY_RUN_S3_BUCKET=nangman-crypto-dev-research-<account-suffix>
RESEARCH_HISTORICAL_REPLAY_RUN_S3_KEYS optional comma-separated replay-run keys
RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET=nangman-crypto-dev-research-<account-suffix>
RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_KEYS optional comma-separated replay-run-index keys
RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX optional replay-run-index prefix discovery
RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_READ_LIMIT optional, default 20
RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_SCAN_LIMIT optional, default 1000
RESEARCH_OUTPUT_S3_BUCKET=nangman-crypto-dev-research-<account-suffix>
RESEARCH_OUTPUT_S3_PREFIX optional, default empty
```

When `RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX` is set, the app discovers
the latest `replay-run-index/.../part-000001.jsonl` objects under that prefix
and loads the referenced historical replay samples before running the aggregate
gate. This lets S3-triggered candidate runs accumulate replay evidence without a
separate manifest for every dispatch. The discovery path is read-only and is
bounded by `READ_LIMIT` and `SCAN_LIMIT`.

ECS placement:

```text
cluster = ecs-nangman-dev-invest-apn2
task definition = td-nangman-dev-research-apn2
capacity provider = FARGATE_SPOT only
log group = /aws/ecs/log-nangman-dev-research-apn2
```

## Activation Readiness

Before switching the S3 dispatcher from `dry_run` to `run_task`, run the
readiness check. It does not write research reports, does not switch the
dispatcher mode, and refuses to invoke the dispatcher unless the Lambda is still
in `dry_run`.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
RESEARCH_DRY_RUN_BUCKET=nangman-crypto-dev-intel-candidate-<account-suffix> \
RESEARCH_DRY_RUN_KEY=candidate-evidence-bundle/priority=p2/schema=intel_candidate_evidence_bundle_v1/dt=YYYY-MM-DD/hour=HH/candidate_id=<candidate-id>/part-000001.jsonl \
scripts/check-activation-readiness.sh
```

The check verifies:

```text
- dispatcher Lambda is Active and update status is Successful
- RESEARCH_DISPATCH_MODE is dry_run
- dispatcher points at td-nangman-dev-research-apn2 and research-app
- latest task definition is ACTIVE, ARM64, Linux, readonly root filesystem
- task definition has RESEARCH_OUTPUT_S3_BUCKET and RESEARCH_MARKET_L1_S3_BUCKET
- optional dry-run S3 event matches the dispatcher filter without starting ECS
- no RUNNING/PENDING ECS task exists with startedBy=research-s3-dispatcher
- latest research output prefixes are visible before activation
```

Only after this passes and research output upload is explicitly approved should
operators switch `RESEARCH_DISPATCH_MODE=run_task` or run an output-enabled
one-shot ECS task.

## Research Loop State Check

Use the loop-state check when deciding whether the system is merely alive or
actually moving candidates through the research factory. It is read-only: it
does not write reports, start ECS tasks, switch dispatcher mode, or create
shadow/paper artifacts.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
scripts/check-loop-state.sh
```

The output separates these states:

```text
- runtime_alive
- dispatcher_auto_research_enabled
- major50_universe_observed
- major50_universe_approved
- candidate_generated
- artifact_created
- research_replay_completed
- promotion_passed
- shadow_created
- paper_created
- live_enabled
```

The expected progression is:

```text
major-50 universe
  -> candidate evidence bundle
  -> research replay
  -> RETEST / PROMOTE / PRUNE
  -> shadow only after PROMOTE
  -> paper only after completed passed shadow
  -> live remains false in research-app
```

This check intentionally reports bottlenecks such as `dispatcher_not_run_task`,
`major50_approved_universe_incomplete`, `no_promoted_shadow_candidate`,
`shadow_output_absent`, and `paper_output_absent` instead of calling the system
"done" just because ECS or Lambda is healthy.

## Research Report Summary

Use the local report summary when a batch run completes but every candidate
stays in `RETEST_BIAS`. It reads local artifacts only and does not upload S3
outputs, start ECS tasks, switch the dispatcher, or create shadow/paper
artifacts.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RESEARCH_REPORT_FILE="/tmp/nangman-crypto/research-output/research-run-report/schema=research_run_report_v1/dt=YYYY-MM-DD/hour=HH/research_run_report_id=<report-id>/report.json" \
RESEARCH_AGGREGATE_REGISTRY_FILE="/tmp/nangman-crypto/research-output/research-aggregate-registry/schema=research_aggregate_registry_record_v1/dt=YYYY-MM-DD/hour=HH/research_run_report_id=<report-id>/part-000001.jsonl" \
scripts/summarize-research-report.sh
```

The summary separates:

```text
- source candidate / replay / partition counts
- RETEST / PRUNE / surviving counts
- reason-code histogram
- per-symbol aggregate sample counts
- strongest positive RETEST aggregates
- next research needs such as more native replay samples, unseen windows, or liquidity inputs
```

## Retest Horizon Plan

Use the retest horizon plan after a `current_approved` batch run to decide
whether each candidate horizon is waiting for Market-L1 coverage, ready for
another replay run, or blocked by sample accumulation. It fetches candidate
bundles referenced by a local `research_input_manifest_v1`, reads the local
report, and optionally discovers the latest Market-L1 universe as-of time. It
does not upload reports, start ECS tasks, switch the dispatcher, or create
shadow/paper artifacts.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
RESEARCH_MARKET_L1_S3_BUCKET=nangman-crypto-dev-market-ingest-l1-<account-suffix> \
scripts/build-retest-horizon-plan.sh \
  /tmp/nangman-crypto/research-input-manifest.json \
  /tmp/nangman-crypto/research-output/research-run-report/schema=research_run_report_v1/dt=YYYY-MM-DD/hour=HH/research_run_report_id=<report-id>/report.json
```

The output separates:

```text
- wait_for_market_l1_horizon
- run_research_replay_for_horizon
- materialize_completed_native_replay_sample
- accumulate_completed_native_replay_samples
- materialize_unseen_replay_windows
- materialize_train_validation_split
- materialize_liquidity_filter_inputs
```

## Retest Horizon Status

Use the retest horizon status summary to track every candidate by symbol and
1h/4h/24h horizon after a batch run. It reads the local retest horizon plan and
optional batch driver summary, then emits a compact checkpoint for repeated
operator review.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

scripts/summarize-retest-horizon-status.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-plan.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/batch-driver-summary.json
```

The checkpoint separates:

```text
- stage_state: candidate_generated, research_replay_completed, promotion_passed, shadow_created, paper_created, live_enabled
- batch_state: selected candidates, replay count, RETEST/PROMOTE surface
- by_symbol: per-symbol candidate and horizon status
- by_horizon: 1h/4h/24h action counts
- next_decision: safe next actions and blocked shadow/paper/live actions
```

The batch driver writes this checkpoint automatically as
`retest-horizon-status.json`. The summary is local-only: it does not upload S3
outputs, start ECS tasks, switch the dispatcher, or create shadow/paper/live
artifacts.

## Focused Retest Manifest

Use the focused retest manifest when a status checkpoint shows only a small set
of horizons are ready for another local replay. It reads the local
`retest-horizon-status.json` plus the source `research_input_manifest_v1`, then
creates a smaller manifest containing only candidate bundles whose horizons
match the requested `next_action`.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RESEARCH_FOCUS_MANIFEST_OUTPUT=/tmp/nangman-crypto/research-focus/input-manifest.json \
RESEARCH_FOCUS_SUMMARY_OUTPUT=/tmp/nangman-crypto/research-focus/input-manifest.summary.json \
scripts/build-focused-retest-manifest.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-input-manifest.json
```

By default the focused manifest selects horizons with:

```text
run_research_replay_for_horizon
materialize_completed_native_replay_sample
```

Override with `RESEARCH_FOCUS_NEXT_ACTIONS=action_a,action_b`. The script is
local-manifest only. By default it excludes historical replay index refs so a
small focused run does not pull unrelated historical aggregates into the
checkpoint. Set `RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS=true` only when
the focused manifest is meant to reuse the source manifest's full historical
evidence surface. The script does not fetch candidate bundles, upload reports,
start ECS tasks, switch the dispatcher, or create shadow/paper/live artifacts.

## Post-Activation Runtime Check

After an approved output-enabled run or dispatcher activation, verify the
runtime artifacts with:

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
RESEARCH_EXPECTED_DISPATCH_MODE=run_task \
RESEARCH_OUTPUT_MIN_LAST_MODIFIED=YYYY-MM-DDTHH:MM:SS+00:00 \
scripts/check-post-activation-runtime.sh
```

The post-activation check verifies:

```text
- dispatcher Lambda is Active and in the expected mode
- latest task definition is ACTIVE, ARM64, Linux, readonly root filesystem
- task definition has research output bucket and historical replay index prefix
- latest research-run-report, replay-run, and replay-run-index are present
- optional shadow-validation-run and paper-trade-run freshness is reported
- research_run_report_v1 sample has candidate ids, replay ids, aggregate fields, and gate policy
```

For pre-activation dry checks, keep freshness disabled:

```bash
AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
RESEARCH_EXPECTED_DISPATCH_MODE=dry_run \
RESEARCH_VERIFY_FRESH_OUTPUT=false \
scripts/check-post-activation-runtime.sh
```

## V0 Boundaries

```text
native_replay only
no Freqtrade execution
no LEAN execution
no Hummingbot execution
no paper/live order
no EXECUTION_APPROVED
no LIVE_READY
```

If deterministic market replay data is missing, the app emits `RETEST_BIAS` with explicit reasons instead of pretending profitability was verified.

The report includes `partition_aggregates` with sample counts, win rate, net edge, profit factor, inferred unseen windows, regime labels, and deterministic gate reasons. Positive replay is never enough by itself: promotion is blocked until sample, unseen, split, liquidity, cost, and regime evidence clear the research gate.

Each `replay_run_v1.result_summary` includes `liquidity_filter_summary` when the candidate bundle requests a liquidity filter. Native replay marks it `passed` only when matched Market-L1 replay data contains a liquidity metric such as `trade_volume` or `volume_change_same_window` with positive current volume. If no liquidity metric is matched, the aggregate gate keeps `liquidity_filter_not_materialized`; if liquidity data exists but has no positive current volume, it emits `liquidity_filter_failed`.

Historical replay-runs are merged into the aggregate gate, but only the current invocation's replay-runs are written to the new replay-run output. Each replay-run output also gets a `replay-run-index/schema=replay_run_index_v1/.../part-000001.jsonl` artifact so later research runs can discover historical samples by `research_aggregate_key`.

Every report writes `research-aggregate-registry/schema=research_aggregate_registry_record_v1/.../part-000001.jsonl`. This is a research-owned projection, not the canonical `memory-app` candidate registry. It can say `pruned`, `retest`, `shadow_candidate`, or `paper_candidate_bias`, but never `EXECUTION_APPROVED` or `LIVE_READY`.

If the deterministic gate promotes a candidate to shadow, the app writes `shadow-validation-run/schema=shadow_validation_run_v1/.../part-000001.jsonl` with `status=pending`, `passed=false`, and `no_order_execution=true`. Old samples are decay-aware:

```text
0-30 days   = full sample weight
31-60 days  = 0.7 sample weight
61-90 days  = 0.4 sample weight
>90 days    = expired, excluded from promotion gate
```

The app can emit `PROMOTE_TO_SHADOW_BIAS` from replay evidence alone. It emits `PROMOTE_TO_PAPER_BIAS` and writes `paper-trade-candidate`, `paper-trade-run`, `paper-trade-summary`, and `paper-trade-mark` only when a completed, passed `shadow_validation_run_v1` is supplied through `--shadow-validation-run-file`, `--shadow-validation-run-s3-key`, or `research_input_manifest_v1.shadow_validation_run_refs[]`.

Paper output still does not approve execution. `paper_trade_summary.promote_recommendation` is a review signal only; the app never emits `EXECUTION_APPROVED` or `LIVE_READY`.
