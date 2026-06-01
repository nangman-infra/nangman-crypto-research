#!/usr/bin/env bash

find_latest_report_file() {
  if [[ ! -d "$RESEARCH_OUTPUT_DIR/research-run-report" ]]; then
    return 0
  fi
  find "$RESEARCH_OUTPUT_DIR/research-run-report" \
    -type f \
    -name "report.json" \
    -print 2>/dev/null \
  | sort \
  | tail -n 1
}

find_latest_registry_file() {
  if [[ ! -d "$RESEARCH_OUTPUT_DIR/research-aggregate-registry" ]]; then
    return 0
  fi
  find "$RESEARCH_OUTPUT_DIR/research-aggregate-registry" \
    -type f \
    -name "part-000001.jsonl" \
    -print 2>/dev/null \
  | sort \
  | tail -n 1
}
