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
  -> optional paper_watch_candidate_v1
  -> optional paper_watch_live_mark_v1 from MARKET_LIVE
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

## Paper-Watch Live Cycle

`paper_watch_candidate_v1`은 실거래 후보가 아니라 forward 모의 관측 후보입니다.
`research-app`은 `MARKET_LIVE` NATS stream 또는 로컬 tick 파일을 읽어서
`paper_watch_live_mark_v1`을 기록할 수 있습니다. 이 경로는 주문을 만들지 않고,
`live_enabled=false`, `order_execution_enabled=false`,
`execution_approval_emitted=false`를 유지합니다.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

cargo run -- \
  --run-paper-watch-live-cycle \
  --paper-watch-candidate-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --paper-watch-candidate-s3-key paper-watch-candidate/schema=paper_watch_candidate_v1/.../part-000001.jsonl \
  --market-live-nats-url nats://<private-nats-host>:4222 \
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --output-s3-prefix paper-watch-live-mark/schema=paper_watch_live_mark_v1
```

Default NATS settings:

```text
stream: MARKET_LIVE
subject: market_live_tick.created.>
consumer: research-paper-watch-live
deliver policy: last_per_subject
batch size: 100
max messages per cycle: 500
```

## Paper-Watch Observer Service

`--run-paper-watch-live-cycle`은 한 번 실행하고 끝나는 검증용 task입니다.
상시 모의 관찰은 `--run-paper-watch-observer`로 실행합니다. 이 모드는 최근
`paper_watch_candidate_v1` 후보를 S3에서 반복해서 읽고, `MARKET_LIVE` NATS
stream에서 Binance/Upbit live tick을 계속 받아 `paper_watch_live_mark_v1`과
`paper_watch_observer_snapshot_v1`을 누적합니다.

이 모드도 실제 주문을 만들지 않습니다. 안전 플래그는 계속
`live_enabled=false`, `order_execution_enabled=false`,
`execution_approval_emitted=false`입니다.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

cargo run -- \
  --run-paper-watch-observer \
  --paper-watch-candidate-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --paper-watch-candidate-s3-prefix paper-watch-candidate/schema=paper_watch_candidate_v1 \
  --market-live-nats-url nats://<private-nats-host>:4222 \
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --output-s3-prefix paper-watch-observer-state/schema=paper_watch_observer_snapshot_v1 \
  --paper-watch-live-mark-s3-prefix paper-watch-live-mark/schema=paper_watch_live_mark_v1 \
  --paper-watch-observer-poll-secs 5
```

Smoke test처럼 한 번만 돌릴 때는 다음 옵션을 붙입니다.

```bash
--paper-watch-observer-max-iterations 1
```

CodeBuild가 observer task definition을 등록할 때는
`RESEARCH_MARKET_LIVE_NATS_URL`을 CodeBuild 환경변수나 기존 observer task
definition 환경변수로 제공해야 합니다. repo에는 private NATS host 값을
커밋하지 않습니다. 배포 buildspec은 기존 observer task definition의
paper-watch/NATS 환경변수를 먼저 보존한 뒤 새 image revision만 적용합니다.

운영 판단 기준:

```text
WAIT_FOR_LIVE_TICK: 아직 해당 후보의 실시간 tick이 없음
WATCHING: 관찰 중이지만 아직 뚜렷한 양수 성과 없음
WATCHING_POSITIVE: 관찰 중 양수 성과가 한 번 이상 나옴
RISK_REVIEW: paper 관찰 중 -200 bps 이하 손실 구간 발생
SHADOW_REVIEW_CANDIDATE: 목표 보유시간 구간에 도달했고 수익/손실 조건이 양호함
```

## Pipeline Alerts

`research-app` emits operator alerts as `pipeline_alert_event_v1` JSON objects
under S3. It does not call the Mattermost webhook directly from ECS. The central
`lmbd-nangman-dev-pipeline-alert-apn2` Lambda reads those objects, applies
priority filtering and dedupe, then sends the human-readable Mattermost message.

Alert delivery is best-effort: an alert write failure is logged to stderr and
does not fail the research run.

```bash
NANGMAN_PIPELINE_ALERT_S3_BUCKET=nangman-crypto-dev-research-<account-suffix>
NANGMAN_PIPELINE_ALERT_S3_PREFIX=pipeline-alert-event/schema=pipeline_alert_event_v1
NANGMAN_ALERT_ENV=dev
NANGMAN_ALERT_MIN_PRIORITY=P2
```

Priority behavior:

```text
P0: portfolio notional, live/order safety boundary, or unsafe paper-watch mark changed
P1: PROMOTE_TO_PAPER or paper candidate created
P2: PAPER_WATCH candidate creation, PROMOTE_TO_SHADOW, or shadow validation created
P3: RETEST blocker summary, only when explicitly enabled
```

`paper-watch-live-mark` batches are intentionally quiet when they are paper-only
and safe. They are high-frequency observation data, not operator decisions. A
P0 alert is emitted only if a mark reports non-paper-only, live-enabled,
order-execution-enabled, or execution-approved state. Routine live mark progress
belongs in the pipeline digest/state-change alert layer.

Optional lower-signal summaries:

```bash
NANGMAN_ALERT_MIN_PRIORITY=P3
NANGMAN_ALERT_INCLUDE_RETEST_SUMMARY=true
NANGMAN_ALERT_INCLUDE_SHADOW_WAIT=true
```

Message shape is intentionally stable for operators:

```text
[P2][research-app] PROMOTE_TO_SHADOW 후보 발생

결론:
후보가 shadow 관측 단계로 올라갔습니다. 주문 실행은 없고 forward observation만 시작됩니다.

현재 상태:
- report_id: ...
- run_scope: ...
- RETEST: ...
- PRUNE: ...
- PROMOTE_TO_SHADOW: ...
- paper-watch candidates: ...
- paper candidates: 0
- max_total_notional_pct: 0.0000

주요 원인:
- deterministic_shadow_gate_passed: ...

다음 행동:
- watch shadow validation until completion evidence exists
- do not create paper/live approval from pending shadow state

안전 상태:
- order execution: disabled by research-app contract
- EXECUTION_APPROVED: never emitted by research-app
- LIVE_READY: never emitted by research-app
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

The default recent-candidate scan is intentionally wide:
`RESEARCH_BATCH_CANDIDATE_READ_LIMIT=1000` and
`RESEARCH_BATCH_MAX_CANDIDATE_BUNDLE_COUNT=1000`. This keeps current major-50
coverage checks from silently missing still-relevant approved candidates that
fall outside a very small recent-object window. For a quick smoke check, lower
both values explicitly in the shell environment.

The builder also enforces the current research holding horizon contract before
selecting a bundle. Stale candidate bundles with unsupported or over-limit
`allowed_horizons` are excluded from the local manifest and counted in
`horizon_contract_invalid_candidate_count` with
`excluded_horizon_contract_violations[]`; the source S3 objects are not
modified.

In ECS, market replay inputs are loaded from Market-L1 S3. The app first uses
`selected_market_artifacts[].artifact_key` from the candidate bundle, then falls
back to the sibling keys derived from `market_data_quality_summary/run_id=...`.
It also discovers later Market-L1 replay windows from the candidate's
`forbidden_lookahead_boundary_ms` through the currently materialized horizon.
Discovery first checks direct `market_feature_delta/run_id=l1_<window>_*` and
`market_regime_context/run_id=l1_<window>_*` objects, then falls back through
the success-only `l1_index/window_ms=1000/...` pointer to the L1 manifest. This
lets research reuse longer Market-L1 normalize runs that cover the target
window, without manually wiring every delta/context key.

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

## Current-Approved ECS Dispatch Shards

Use the dispatch shard driver after the dispatcher is intentionally in
`run_task` mode and the goal is to send the current-approved batch through the
real S3 notification -> Lambda -> ECS path.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
RESEARCH_DISPATCH_SHARD_SIZE=40 \
scripts/run-current-approved-research-dispatch-shards.sh
```

The driver keeps the Lambda dispatcher thin. It builds or copies one
`research_input_manifest_v1`, splits `candidate_bundle_refs[]` into shard
manifests, uploads each shard under `research-input-manifest/...`, waits for the
`research-s3-dispatcher` ECS task, and fails immediately if any shard exits
non-zero. The default shard size is 40 because full current-approved batches can
exceed the research task memory envelope when historical replay indexes are
included. Sharding preserves the full candidate set while avoiding one oversized
task.

Dry-run mode only writes local shard manifests and the summary:

```bash
RESEARCH_DISPATCH_DRY_RUN=true \
RESEARCH_DISPATCH_SOURCE_MANIFEST_FILE=/tmp/nangman-crypto/research-current-approved-batch/<run-id>/input-manifest.json \
scripts/run-current-approved-research-dispatch-shards.sh
```

Safety boundaries:

```text
- requires current_approved by default
- does not change dispatcher mode
- does not enable shadow, paper, or live
- does not execute orders
- writes S3 only by uploading shard manifests and letting research-app write its normal research artifacts
```

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

Runtime S3 clients use AWS S3 with IAM only. Custom S3-compatible endpoint
configuration is not part of the app contract.

When `RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX` is set, the app discovers
the latest `replay-run-index/.../part-000001.jsonl` objects under that prefix
and loads the referenced historical replay samples before running the aggregate
gate. Use a narrowed prefix, not the broad `replay-run-index/` root, for runtime
task definitions. The deployment buildspec removes that broad root prefix from
new task definition revisions so scheduler tasks do not repeatedly scan a
truncated S3 listing. The discovery path is read-only and is bounded by
`READ_LIMIT` and `SCAN_LIMIT`.
Historical replay samples are filtered to aggregate keys produced by the current
manifest's candidate bundles before they enter the gate. This preserves same
symbol/horizon accumulation while preventing unrelated symbols from appearing in
a focused research report.

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

The loop-state check uses `RESEARCH_LOOP_STATE_CANDIDATE_READ_LIMIT=1000` by
default for the same reason as the batch manifest builder: a narrow recent
window can make the system look less covered than the current artifact set
actually is. It also reads recent research reports with
`RESEARCH_LOOP_STATE_REPORT_READ_LIMIT=100` and selects the largest complete
`current_approved_auto_research_validation_shard` batch as `research_evidence`
when that batch covers more candidates than the latest single report. Override
these limits only for bounded smoke checks.

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

It also emits `coverage_gaps` and a machine-readable `next_decision`:

```text
- next_decision.verdict
- next_decision.safe_next_actions
- next_decision.blocked_actions
- next_decision.safety
- next_decision.evidence
```

The output keeps `latest_research_report` as the newest single report, but uses
`best_current_approved_shard_batch` and `research_evidence` to avoid treating a
small dispatcher smoke report as the whole research coverage surface. It also
emits `recent_research_report_coverage`; coverage-gap calculations use this
recent report symbol union so a focused replay can close a single-symbol replay
gap without rerunning the whole current-approved batch.

`next_decision` is a scheduling handoff, not an execution command. It keeps
the check read-only, records why automation should wait or continue, and keeps
shadow/paper/live blocked unless the upstream evidence is present.

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
bundles referenced by a `research_input_manifest_v1`, reads the research-run
report, and optionally discovers the latest Market-L1 universe as-of time. It
does not start ECS tasks, switch the dispatcher, or create shadow/paper
artifacts.

Prefer the app-native planner for scheduler/ECS paths:

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

cargo run -- \
  --build-retest-horizon-plan \
  --input-manifest-file /tmp/nangman-crypto/research-input-manifest.json \
  --research-report-file /tmp/nangman-crypto/research-output/research-run-report/schema=research_run_report_v1/dt=YYYY-MM-DD/hour=HH/research_run_report_id=<report-id>/report.json \
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
  --retest-horizon-plan-output-file /tmp/nangman-crypto/retest-horizon-plan.json
```

For ECS automation, read the manifest/report from S3 and write the plan under a
non-dispatching `retest-horizon-plan/` prefix. This plan object is a checkpoint
input for status/scheduler logic; it does not trigger research by itself:

```bash
cargo run -- \
  --build-retest-horizon-plan \
  --input-manifest-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --input-manifest-s3-key retest-cycle-source/schema=research_input_manifest_v1/... \
  --research-report-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --research-report-s3-key research-run-report/schema=research_run_report_v1/.../report.json \
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --output-s3-prefix retest-horizon-plan/schema=research_retest_horizon_plan_v1
```

For the runtime loop, prefer `--run-retest-refresh-cycle`. It folds the planner,
status checkpoint, and focused manifest builder into one app-native call. The
mode writes fresh checkpoints under non-dispatching `retest-horizon-plan/` and
`retest-horizon-status/` prefixes. It writes under `research-input-manifest/`
only when the rebuilt status says `RUN_FOCUSED_RETEST_RESEARCH`; that S3 write
wakes the existing S3 notification -> Lambda -> ECS research path. A WAIT status
creates no dispatcher manifest. S3 focused manifests are written with a stable
dedupe key derived from the source manifest, source report, latest Market-L1
watermark, and selected retest rows. Re-running the same refresh cycle keeps the
same key and does not emit another `ObjectCreated` event; once Market-L1 or the
selection changes, the key changes and the dispatcher can run the next research
sample.

When a prior S3 research run has completed, `research-app` also writes a
non-dispatching `retest-cycle-source-state/` checkpoint that links the source
manifest key to the report key it produced. EventBridge or another scheduler can
then call `--run-retest-refresh-cycle-from-latest-state` with only the research
bucket and Market-L1 bucket. The app discovers the latest manifest/report pair
from that checkpoint, refreshes the status from current Market-L1 evidence, and
keeps the same focused-manifest dedupe guard.

```bash
cargo run -- \
  --run-retest-refresh-cycle \
  --input-manifest-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --input-manifest-s3-key research-input-manifest/schema=research_input_manifest_v1/... \
  --research-report-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --research-report-s3-key research-run-report/schema=research_run_report_v1/.../report.json \
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix>
```

```bash
cargo run -- \
  --run-retest-refresh-cycle-from-latest-state \
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix>
```

Runtime wake-up model:

```text
research-input-manifest/ ObjectCreated
  -> S3 notification
  -> research-s3-dispatcher Lambda
  -> one ECS research task

EventBridge Scheduler rate tick
  -> one ECS retest-refresh task
  -> latest retest-cycle-source-state/ lookup
  -> Market-L1 horizon refresh
  -> writes a new research-input-manifest/ only when another research sample is due
  -> S3 notification wakes the normal dispatcher path
```

S3 is still the durable state truth. The Scheduler is only a wake-up signal for
checking whether the latest source state and Market-L1 horizon require another
focused retest manifest. It must run on `FARGATE_SPOT`, use private subnets, and
must not enable shadow, paper, live, or order execution.

During CodePipeline deployment, the buildspec updates the retest refresh and
shadow cycle EventBridge Scheduler targets to the newly registered research task
definition revision. This prevents recurring schedules from continuing to run an
older revision after the ECS image deployment succeeds.

Example Scheduler support files:

```text
ecs/scheduler-assume-role-policy.example.json
ecs/scheduler-run-task-policy.example.json
ecs/retest-refresh-scheduler-target.example.json
```

Create or update the recurring Scheduler from a private operator context after
replacing placeholders:

```bash
aws scheduler create-schedule \
  --name sch-nangman-dev-research-retest-refresh-apn2 \
  --schedule-expression 'rate(15 minutes)' \
  --flexible-time-window Mode=OFF \
  --state ENABLED \
  --description 'Runs research-app retest refresh cycle from latest source state on ECS Fargate Spot' \
  --target file://ecs/retest-refresh-scheduler-target.example.json
```

The legacy shell planner remains useful for local diagnosis:

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
- extend_market_l1_horizon_coverage
- accumulate_completed_native_replay_samples
- materialize_unseen_replay_windows
- materialize_train_validation_split
- materialize_liquidity_filter_inputs
```

Horizon `next_action` is based on horizon-specific aggregate gate reasons.
Candidate-level `summary_findings.reason_codes` are preserved separately as
`candidate_reason_codes`, but they are not used to mark every horizon as a
Market-L1 coverage gap. This keeps stale or cross-horizon candidate reasons from
masking horizons that are already ready for replay.
The planner also treats aggregate `missing_market_replay_data_count > 0` as a
horizon coverage gap, because aggregate gate reasons can otherwise collapse the
same surface to `no_completed_native_replay_samples`.

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

The same checkpoint builder is available inside `research-app`, which is the
scheduler-friendly path because it removes the local shell/JQ dependency:

```bash
cargo run -- \
  --build-retest-horizon-status \
  --retest-horizon-plan-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-plan.json \
  --retest-horizon-status-output-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json
```

For ECS automation, read the plan from S3 and write the status under a
non-dispatching `retest-horizon-status/` prefix. This status object is a
checkpoint; it does not trigger research by itself:

```bash
cargo run -- \
  --build-retest-horizon-status \
  --retest-horizon-plan-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --retest-horizon-plan-s3-key retest-horizon-plan/schema=research_retest_horizon_plan_v1/... \
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --output-s3-prefix retest-horizon-status/schema=research_horizon_status_checkpoint_v1
```

The checkpoint separates:

```text
- verdict: current next decision, such as EXTEND_MARKET_L1_HORIZON_COVERAGE
- selected_symbols and next_action_counts: top-level operator scan fields
- major50_state: observed/approved universe counts, selected candidate symbol
  coverage, eligible candidate symbol coverage, and batch-cap exclusions
- research_factory_progression: major-50 -> candidate -> replay -> promotion -> shadow/paper/live, separated by symbol and candidate id
- coverage_gaps: approved symbols without eligible candidates, approved symbols
  outside the selected batch, candidate ids without replay, and replayed
  candidate ids without promotion
- research_factory_gap_summary: the current blocking stage and safe next actions
- stage_state: candidate_generated, research_replay_completed, promotion_passed, shadow_created, paper_created, live_enabled
- batch_state: selected candidates, replay count, RETEST/PROMOTE surface
- by_symbol: per-symbol candidate and horizon status
- by_horizon: 1h/4h/24h action counts, including market coverage extension needs
- candidate_horizon_matrix: each candidate's 1h/4h/24h requested/replay/coverage/promotion-review state
- materialization_schedule: latest Market-L1 as-of time, next/last pending
  horizon due time, wait deficit, and accumulation due range
- missing_market_replay_data_count: horizon aggregate count used to identify coverage gaps
- next_decision: safe next actions, scheduler_hint, and blocked
  shadow/paper/live actions
```

The batch driver writes this checkpoint automatically as
`retest-horizon-status.json`. The summary is local-only: it does not upload S3
outputs, start ECS tasks, switch the dispatcher, or create shadow/paper/live
artifacts.
`next_decision.scheduler_hint` is the machine-readable handoff for the next
research/retest cycle. When `run_now_replay_ready` is false and
`run_research_after_l1_as_of_ms` is set, an external scheduler should wait until
Market-L1 has advanced to that as-of boundary before dispatching another
focused research run.
The app can validate this handoff without loading research inputs or producing
research outputs:

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

cargo run -- \
  --retest-horizon-status-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json
```

This mode validates that the checkpoint is local-summary only, does not enable
live trading, preserves the shadow/paper/live blocked actions, and returns the
next scheduler action. It does not upload S3 outputs, start ECS tasks, switch
the dispatcher, or create shadow/paper/live artifacts.
When the optional batch driver summary is absent, the checkpoint derives
`stage_state.research_replay_completed` from the retest horizon plan: every
candidate in the plan must have at least one replay-backed horizon. This keeps
focused source-gap replays from reporting an unknown replay state only because
they were not launched by the current-approved batch driver.

## Shadow Validation Status

Use the shadow validation status summary after a batch emits
`shadow-validation-run/schema=shadow_validation_run_v1/.../part-000001.jsonl`.
This checkpoint separates "PROMOTE_TO_SHADOW_BIAS exists" from "paper input is
ready". Pending shadow runs are not paper-ready.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RUN_DIR=/tmp/nangman-crypto/research-current-approved-batch/<run-id>
SHADOW_FILE=/tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-output/shadow-validation-run/<partition>/part-000001.jsonl

scripts/summarize-shadow-validation-status.sh \
  "$SHADOW_FILE" \
  "$RUN_DIR/retest-horizon-status.json" \
  > "$RUN_DIR/shadow-validation-status.json"
```

The summary reports:

```text
- shadow_validation_summary: pending/completed/failed/pass counts by symbol
- paper_gate.paper_generation_precondition_met: true only for completed + passed shadow runs
- paper_gate.blocked_actions: paper/live actions that must remain closed
- safety: local-only, no S3 write, no ECS task, no dispatcher change
```

The paper precondition is intentionally strict:

```text
status == completed
passed == true
paper_trade_candidate_contract_version == paper_trade_candidate_v1
termination_policy.no_order_execution == true
```

## Shadow Validation Merge

Use the merge checkpoint before recomputing observation or sample gap status
from multiple shadow runs. It deduplicates by `shadow_validation_run_id` so
repeated local loops do not inflate the effective sample count.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RUN_DIR=/tmp/nangman-crypto/research-current-approved-batch/<run-id>

scripts/merge-shadow-validation-runs.sh \
  "$RUN_DIR/shadow-validation-merged.jsonl" \
  "$RUN_DIR/research-output/shadow-validation-run/<partition>/part-000001.jsonl" \
  "$RUN_DIR/shadow-accumulation-research-output/shadow-validation-run/<partition>/part-000001.jsonl"
```

The summary reports input count, merged count, duplicate count, symbols, and
status counts. The merge is local-only: it does not mutate shadow status, does
not write S3, does not start ECS tasks, and does not create paper/live
artifacts.

## Shadow Observation Plan

Use the shadow observation plan while shadow runs are still pending. It answers
whether the target holding window has enough Market-L1 coverage to review, and
whether the shadow sample requirement is proven yet. This does not complete or
pass shadow runs.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RUN_DIR=/tmp/nangman-crypto/research-current-approved-batch/<run-id>
SHADOW_FILE=/tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-output/shadow-validation-run/<partition>/part-000001.jsonl

scripts/build-shadow-observation-plan.sh \
  "$SHADOW_FILE" \
  "$RUN_DIR/retest-horizon-status.json" \
  > "$RUN_DIR/shadow-observation-plan.json"
```

The planner uses `retest-horizon-plan.json` through the status checkpoint to
discover the latest Market-L1 `as_of` watermark. You can override it with the
third argument or `RESEARCH_SHADOW_OBSERVATION_LATEST_L1_AS_OF_MS`.

The checkpoint separates:

```text
- target_window_materialized_count: target hold window is covered by Market-L1
- absolute_window_materialized_count: force-flat absolute deadline is covered
- observed_shadow_run_count: shadow records seen for the symbol/candidate
- required_shadow_sample_count: watch_window_policy.min_shadow_samples
- blocked_actions: paper/live actions that remain closed
```

The output is local-only. It does not mark shadow as completed, does not write
S3, does not start ECS tasks, and does not create paper/live artifacts.

## Shadow Sample Gap Manifest

Use the shadow sample gap manifest after `shadow-observation-plan.json` exists.
It turns the observation plan into a candidate backlog: which promoted
candidates still need more shadow observation samples before any completed
shadow review can feed paper.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RUN_DIR=/tmp/nangman-crypto/research-current-approved-batch/<run-id>

scripts/build-shadow-sample-gap-manifest.sh \
  "$RUN_DIR/shadow-observation-plan.json" \
  > "$RUN_DIR/shadow-sample-gap-manifest.json"
```

The manifest separates:

```text
- total_sample_deficit: missing shadow observations across promoted candidates
- shadow_sample_backlog: candidate-level required/materialized/deficit counts
- sample_ready_candidates: candidates whose sample requirement is met
- partially_materialized_candidate_count: records that exist but whose later target windows are not fully covered yet
- next_observation_not_before_ms: earliest pending target window deadline to recheck
- next_decision.verdict: whether to wait, accumulate samples, or review completion evidence
- blocked_actions: shadow/paper/live actions that must remain closed
```

Only target-window-materialized shadow runs count toward the sample requirement.
A newly created pending shadow run is not a paper-ready sample until its target
holding window has enough Market-L1 coverage.

If some shadow records are materialized and newer records are still waiting for
their target window, the manifest reports
`WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION`. This prevents the local
cycle from immediately creating another focused research manifest before the
already-created pending observations can become valid samples. The
`next_observation_not_before_ms` field tells an autonomous loop the earliest
safe time to refresh the observation plan.

The output is local-only. It does not mutate shadow status, does not write S3,
does not start ECS tasks, and does not create paper/live artifacts.

## Shadow Sample Accumulation Manifest

Use the accumulation manifest after the sample gap manifest reports
`ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION`. It maps the deficient shadow
candidate lifecycle keys back to the source research manifest and creates a
focused `research_input_manifest_v1` for the next local research pass.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RUN_DIR=/tmp/nangman-crypto/research-current-approved-batch/<run-id>

RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT="$RUN_DIR/shadow-accumulation-input-manifest.json" \
RESEARCH_SHADOW_ACCUMULATION_SUMMARY_OUTPUT="$RUN_DIR/shadow-accumulation-input-manifest.summary.json" \
scripts/build-shadow-sample-accumulation-manifest.sh \
  "$RUN_DIR/shadow-sample-gap-manifest.json" \
  "$RUN_DIR/retest-horizon-status.json" \
  "$RUN_DIR/research-input-manifest.json"
```

The summary reports:

```text
- backlog_candidate_lifecycle_count: lifecycle keys still short of shadow samples
- total_sample_deficit: remaining required observations
- status_candidate_count: candidate ids mapped from the horizon status checkpoint
- selected_candidate_bundle_ref_count: refs copied into the focused manifest
- missing_candidate_ref_count: mapped candidates absent from the source manifest
- blocked_actions: shadow/paper/live actions that must remain closed
```

The output is local-only. It does not mutate shadow status, does not write S3,
does not start ECS tasks, and does not create paper/live artifacts. After a
local run with the focused manifest, rebuild the shadow observation plan and
sample gap manifest before considering any completed shadow review.

## Shadow Sample Accumulation Cycle

Use the cycle driver after one or more local runs have emitted shadow validation
files. It discovers shadow outputs under a run directory, deduplicates them,
rebuilds the observation plan and sample gap manifest, then writes the next
focused accumulation manifest when more samples are still required.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RUN_DIR=/tmp/nangman-crypto/research-current-approved-batch/<run-id>

scripts/run-shadow-sample-accumulation-cycle.sh "$RUN_DIR"
```

The driver writes:

```text
- shadow-validation-merged.jsonl
- shadow-validation-merged.summary.json
- shadow-observation-plan.cycle.json
- shadow-sample-gap-manifest.cycle.json
- shadow-accumulation-input-manifest.next.json, only when the gap verdict needs more samples now
- shadow-sample-accumulation-cycle-summary.json
- shadow-cycle-decision.json
```

The cycle summary carries `next_decision.next_observation_not_before_ms` from
the gap manifest so a scheduler can wait until the next pending target window
deadline instead of spinning the research loop immediately.

The decision file is the smallest machine-readable scheduler handoff. It maps
the cycle verdict to `scheduler_action`, `run_not_before_ms`, and the focused
research manifest path when another local research pass is safe to prepare. It
is still local-only and keeps paper/live/order execution disabled. The Rust
contract type is `ShadowCycleDecision`, and
`validate_shadow_cycle_decision` rejects wait decisions without
`run_not_before_ms`, focused research decisions without an absolute manifest
path, and any decision that enables paper/live/order execution.

Validate a scheduler decision with the app binary before wiring any scheduler:

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

cargo run -- \
  --shadow-cycle-decision-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/shadow-cycle-decision.json
```

This command only reads the local decision file and prints a validation
summary. It does not run research, write outputs, start ECS, switch the
dispatcher, or create shadow/paper/live artifacts.

The app can also build the same scheduler decision directly from
`shadow_validation_run_v1` inputs. This is the ECS-friendly path because the
container no longer needs the local shell/JQ cycle just to decide whether to
wait, hold for review, or prepare the next research pass.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

cargo run -- \
  --build-shadow-cycle-decision \
  --shadow-validation-run-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/shadow-validation-run.jsonl \
  --shadow-cycle-latest-l1-as-of-ms 1779696900000 \
  --shadow-cycle-decision-output-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/shadow-cycle-decision.json
```

The generated decision keeps `paper_live_enabled=false`,
`live_enabled=false`, and `order_execution_enabled=false`.

The cycle is local-only. It does not run ECS, switch the dispatcher, mutate
shadow status, or create paper/live artifacts.

## Market-L1 Coverage Gap Diagnosis

Use the Market-L1 coverage gap diagnosis when the retest horizon status reports
`EXTEND_MARKET_L1_HORIZON_COVERAGE`. It reads the local retest plan, optional
research report, and optional replay-run JSONL to separate three surfaces:

```text
- plan gaps: horizons that need Market-L1 coverage before replay can help
- aggregate gaps: report partitions with missing native replay market data
- current missing replay windows: concrete symbol/window starts to check in Market-L1
```

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RUN_DIR=/tmp/nangman-crypto/research-current-approved-batch/<run-id>
REPLAY_RUN_FILE=/tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-output/replay-run/<partition>/part-000001.jsonl

scripts/diagnose-market-l1-coverage-gaps.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-plan.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-output/research-run-report/<partition>/report.json \
  "$REPLAY_RUN_FILE"
```

By default this is local-only. It does not upload reports, start ECS tasks,
switch the dispatcher, or create shadow/paper/live artifacts. Enable the
read-only S3 existence check only when you need to confirm whether
the current missing replay windows are discoverable through either direct
`market_feature_delta/run_id=l1_<window>_*/delta.json` /
`market_regime_context/run_id=l1_<window>_*/context.json` keys or the
`l1_index -> manifest -> market_feature_delta_key / market_regime_context_key`
path:

S3 mode verifies the AWS STS session first. Missing or expired credentials fail
before the script classifies Market-L1 coverage.

```bash
AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
RESEARCH_MARKET_L1_S3_BUCKET=nangman-crypto-dev-market-ingest-l1-<account-suffix> \
RESEARCH_MARKET_L1_COVERAGE_CHECK_S3=true \
RESEARCH_MARKET_L1_COVERAGE_CHECK_SYMBOLS=false \
scripts/diagnose-market-l1-coverage-gaps.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-plan.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-output/research-run-report/<partition>/report.json \
  "$REPLAY_RUN_FILE" \
  > /tmp/nangman-crypto/research-current-approved-batch/<run-id>/market-l1-coverage-gap-diagnosis.json
```

The diagnosis can prove that replay is blocked by missing discoverable
Market-L1 feature delta objects for the checked windows, and it separately
reports whether regime context is discoverable for market-regime split quality.
It is not proof that research promotion passed, and it is not approval to open
shadow, paper, or live trading.

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
accumulate_completed_native_replay_samples
materialize_completed_native_replay_sample
```

Horizons classified as `extend_market_l1_horizon_coverage` are intentionally not
selected by default, because repeating research before the horizon-specific
Market-L1 aggregate coverage is materialized will keep producing
`missing_native_replay_market_data`.

Override with `RESEARCH_FOCUS_NEXT_ACTIONS=action_a,action_b`. The script is
local-manifest only. `RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS` defaults to
`auto`: it carries historical replay index refs when the focused action includes
`accumulate_completed_native_replay_samples`, and otherwise keeps small focused
runs light. Set it to `true` to always reuse the source manifest's full
historical evidence surface, or `false` to force current-run-only evidence. The
script does not fetch candidate bundles, upload reports, start ECS tasks, switch
the dispatcher, or create shadow/paper/live artifacts.
If no horizons match the requested actions, it writes an empty manifest and
summary, reports `selected_candidate_bundle_ref_count=0`, and exits non-zero so
operators do not accidentally treat an empty focused run as replay evidence.

The same focused manifest builder is available inside `research-app`, which is
the scheduler-friendly path because it removes the local shell/JQ dependency:

```bash
cargo run -- \
  --build-focused-retest-manifest \
  --retest-horizon-status-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json \
  --input-manifest-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-input-manifest.json \
  --focused-retest-manifest-output-file /tmp/nangman-crypto/research-focus/input-manifest.json
```

For repeated scheduler calls, prefer `--run-retest-cycle-scheduler`. This mode
first validates `retest-horizon-status.json` and only creates a focused manifest
when the status says `RUN_FOCUSED_RETEST_RESEARCH`. If the status is still
waiting and `now_ms < run_not_before_ms`, it exits cleanly with a WAIT summary
and writes no manifest. If the old WAIT status is already past its deadline, it
returns `REFRESH_RETEST_HORIZON_STATUS_AFTER_WAIT_DEADLINE` instead of creating a
stale S3 trigger; operators should refresh the horizon status from current
Market-L1 evidence before dispatching research.

When the caller can provide the original research manifest and research report,
use `--run-retest-refresh-cycle` instead. It refreshes the Market-L1 watermark
and status in the same process before deciding whether to write the dispatching
focused manifest.

```bash
cargo run -- \
  --run-retest-cycle-scheduler \
  --retest-horizon-status-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json \
  --input-manifest-file /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-input-manifest.json \
  --focused-retest-manifest-output-file /tmp/nangman-crypto/research-focus/input-manifest.json
```

For runtime automation, a scheduler can run the same mode with S3 input and S3
manifest output. Writing under `research-input-manifest/` wakes the existing
S3 notification -> Lambda -> ECS research path; the builder itself still does
not run research, switch the dispatcher, or create shadow/paper/live artifacts:

```bash
cargo run -- \
  --run-retest-cycle-scheduler \
  --retest-horizon-status-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --retest-horizon-status-s3-key retest-horizon-status/schema=research_horizon_status_checkpoint_v1/... \
  --input-manifest-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --input-manifest-s3-key research-input-manifest/schema=research_input_manifest_v1/... \
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --output-s3-prefix research-input-manifest/schema=research_input_manifest_v1
```

## Source-Gap Evidence Manifest

Use the source-gap evidence manifest when candidate coverage diagnosis shows an
approved symbol already has research-eligible candidate evidence, but that
evidence was outside the current research batch selection window. This path does
not rerun crawl, Market-L1 backfill, ECS, or dispatcher activation. It only turns
existing candidate evidence refs into a local `research_input_manifest_v1`.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/research-app

RESEARCH_SOURCE_GAP_MANIFEST_OUTPUT=/tmp/nangman-crypto/research-source-gap/input-manifest.json \
RESEARCH_SOURCE_GAP_SUMMARY_OUTPUT=/tmp/nangman-crypto/research-source-gap/input-manifest.summary.json \
scripts/build-source-gap-evidence-manifest.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-source-gap-diagnosis.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/research-input-manifest.json
```

By default the script selects only
`candidate_evidence_outside_research_batch_selection`. Override with
`RESEARCH_SOURCE_GAP_STATUSES=status_a,status_b` only for an explicit local
probe. If diagnosis refs are object keys instead of `s3://` URIs, the script
infers the candidate bucket from the source manifest. Without a source manifest,
set `RESEARCH_SOURCE_GAP_CANDIDATE_S3_BUCKET=<candidate-bucket>`.

`RESEARCH_SOURCE_GAP_INCLUDE_HISTORICAL_INDEX_REFS` defaults to `false` so a
symbol-focused source-gap replay does not mix unrelated historical aggregate
registry rows into the local report. Set it to `true` only when intentionally
accumulating historical replay evidence. The summary reports whether it used
full `evidence_refs` or limited `sample_evidence_refs`; limited refs are enough
for a focused probe but should not be mistaken for complete major-50 coverage.

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

Positive `RETEST_BIAS` candidates can still enter the paper-watch lane. This is
not `PROMOTE_TO_PAPER_BIAS` and it does not approve execution. The app writes
`paper-watch-candidate/schema=paper_watch_candidate_v1/.../part-000001.jsonl`
only when all of these are true:

```text
summary finding bias == RETEST_BIAS
candidate bundle approved_universe_symbol == true
aggregate positive_net_count > 0
aggregate completed_count > 0
aggregate non_positive_net_count == 0
aggregate missing_market_replay_data_count / replay_run_count <= 50%
aggregate mean net after cost > 0
no major failure event such as exchange_delisting, exchange_halt, security_incident, or chain_halt
```

Paper-watch artifacts are forward-observation inputs. Their safety block keeps
`paper_only=true`, `live_enabled=false`, `order_execution_enabled=false`, and
`execution_approval_emitted=false`.

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
