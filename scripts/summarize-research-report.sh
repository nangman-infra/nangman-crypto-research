#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPORT_FILE="${RESEARCH_REPORT_FILE:-${1:-}}"
REGISTRY_FILE="${RESEARCH_AGGREGATE_REGISTRY_FILE:-${2:-}}"

# shellcheck source=scripts/lib/research-report-summary-runtime.sh
source "$SCRIPT_DIR/lib/research-report-summary-runtime.sh"

require_command jq
require_absolute_file "RESEARCH_REPORT_FILE or first argument" "$REPORT_FILE"

registry_summary="$(registry_summary_json "$REGISTRY_FILE")"

jq \
  --arg report_file "$REPORT_FILE" \
  --arg registry_file "$REGISTRY_FILE" \
  --argjson registry "$registry_summary" \
  -f "$SCRIPT_DIR/jq/research-report-summary.jq" \
  "$REPORT_FILE"

echo "research report summary completed" >&2
