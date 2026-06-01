if (
  ($shard_batch.present // false)
  and (($shard_batch.source_candidate_count // 0) > ($latest.source_candidate_count // 0))
) then
  $shard_batch + {evidence_source:"current_approved_shard_batch"}
else
  $latest + {
    evidence_source:"latest_research_report",
    selection:"latest_research_report"
  }
end
