#!/usr/bin/env bash

aws_cmd() {
  aws --region "$REGION" "$@"
}

verify_aws_access() {
  local identity_output
  if ! identity_output="$(aws_cmd sts get-caller-identity --output json 2>&1)"; then
    {
      echo "AWS credentials unavailable or expired for region=$REGION"
      echo "Refresh the AWS login/session, then rerun this check."
      echo "$identity_output"
    } | redact >&2
    exit 1
  fi

  echo "aws identity ok: account=$(jq -r '.Account' <<<"$identity_output")" | redact
}

task_env_value() {
  local name="$1"
  jq -r \
    --arg container "$CONTAINER_NAME" \
    --arg name "$name" \
    '[.taskDefinition.containerDefinitions[]
        | select(.name == $container)
        | (.environment // [])[]?
        | select(.name == $name)
        | .value][0] // ""' "$task_json"
}
