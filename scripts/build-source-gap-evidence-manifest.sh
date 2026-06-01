#!/usr/bin/env bash
set -euo pipefail

DIAGNOSIS_FILE="${RESEARCH_SOURCE_GAP_DIAGNOSIS_FILE:-${1:-}}"
SOURCE_MANIFEST_FILE="${RESEARCH_SOURCE_MANIFEST_FILE:-${2:-}}"
FOCUS_STATUSES="${RESEARCH_SOURCE_GAP_STATUSES:-candidate_evidence_outside_research_batch_selection}"
CANDIDATE_BUCKET="${RESEARCH_SOURCE_GAP_CANDIDATE_S3_BUCKET:-${RESEARCH_CANDIDATE_S3_BUCKET:-}}"
PACKET_ID="${RESEARCH_SOURCE_GAP_PACKET_ID:-research_source_gap_$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_SCOPE="${RESEARCH_SOURCE_GAP_RUN_SCOPE:-source_gap_existing_evidence_local_validation}"
INCLUDE_HISTORICAL_INDEX_REFS="${RESEARCH_SOURCE_GAP_INCLUDE_HISTORICAL_INDEX_REFS:-false}"
INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED="$(printf '%s' "$INCLUDE_HISTORICAL_INDEX_REFS" | tr '[:upper:]' '[:lower:]')"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=scripts/lib/runtime-common-core.sh
source "$SCRIPT_DIR/lib/runtime-common-core.sh"
# shellcheck source=scripts/lib/source-gap-evidence-manifest-output.sh
source "$SCRIPT_DIR/lib/source-gap-evidence-manifest-output.sh"
# shellcheck source=scripts/lib/source-gap-evidence-manifest-validation.sh
source "$SCRIPT_DIR/lib/source-gap-evidence-manifest-validation.sh"
# shellcheck source=scripts/lib/source-gap-evidence-manifest-runtime.sh
source "$SCRIPT_DIR/lib/source-gap-evidence-manifest-runtime.sh"
# shellcheck source=scripts/lib/source-gap-evidence-manifest-reporting.sh
source "$SCRIPT_DIR/lib/source-gap-evidence-manifest-reporting.sh"

validate_source_gap_evidence_manifest_config
prepare_source_gap_evidence_manifest_outputs
prepare_source_gap_evidence_manifest_inputs
write_source_gap_evidence_manifest_files
assert_source_gap_evidence_manifest_selected
print_source_gap_evidence_manifest_summary

echo "source-gap evidence manifest build completed"
