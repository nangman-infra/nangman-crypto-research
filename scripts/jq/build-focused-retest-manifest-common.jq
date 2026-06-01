def unique_sorted: unique | sort;

def action_list($focus_next_actions):
  $focus_next_actions
  | split(",")
  | map(gsub("^\\s+|\\s+$"; ""))
  | map(select(length > 0))
  | unique_sorted;

def candidate_id_from_uri:
  ((.uri // "" | capture("candidate_id=(?<candidate_id>[^/]+)")? // {}) | .candidate_id) // null;

def horizon_order:
  if . == "1h" then 1
  elif . == "4h" then 2
  elif . == "24h" or . == "1d" then 3
  elif . == "72h" then 4
  elif . == "7d" then 5
  else 99 end;
