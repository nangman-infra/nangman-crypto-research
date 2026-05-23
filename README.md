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

In ECS, market replay inputs are loaded from Market-L1 S3. The app first uses
`selected_market_artifacts[].artifact_key` from the candidate bundle, then falls
back to the sibling keys derived from `market_data_quality_summary/run_id=...`.
It also discovers later Market-L1 15-minute replay windows from the candidate's
`forbidden_lookahead_boundary_ms` through the currently materialized horizon, so
native replay can progress after new post-decision market data lands without
manually wiring every delta/context key.

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
RESEARCH_OUTPUT_S3_BUCKET=nangman-crypto-dev-research-<account-suffix>
RESEARCH_OUTPUT_S3_PREFIX optional, default empty
```

ECS placement:

```text
cluster = ecs-nangman-dev-intel-apn2
task definition = td-nangman-dev-research-apn2
capacity provider = FARGATE_SPOT only
log group = /aws/ecs/log-nangman-dev-research-apn2
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
