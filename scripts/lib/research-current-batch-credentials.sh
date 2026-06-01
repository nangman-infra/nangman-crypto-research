#!/usr/bin/env bash

load_aws_exported_credentials_file() {
  local credential_env_file="$1"
  local line
  local key
  local value

  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    if [[ -z "$line" || "$line" == \#* ]]; then
      continue
    fi
    if [[ "$line" == export[[:space:]]* ]]; then
      line="${line#export}"
      line="${line#"${line%%[![:space:]]*}"}"
    fi
    if [[ "$line" != *=* ]]; then
      fail "invalid AWS credential env line in $credential_env_file: expected KEY=VALUE"
    fi
    key="${line%%=*}"
    value="${line#*=}"
    key="${key%"${key##*[![:space:]]}"}"
    value="${value#"${value%%[![:space:]]*}"}"
    if [[ ! "$key" =~ ^AWS_[A-Za-z0-9_]*$ ]]; then
      fail "unexpected AWS credential env key in $credential_env_file: $key"
    fi
    if [[ "$value" == \"*\" && "$value" == *\" && "${#value}" -ge 2 ]]; then
      value="${value:1:${#value}-2}"
    elif [[ "$value" == \'*\' && "$value" == *\' && "${#value}" -ge 2 ]]; then
      value="${value:1:${#value}-2}"
    fi
    export "$key=$value"
  done < "$credential_env_file"
}

prepare_aws_sdk_credentials() {
  if [[ -n "${AWS_ACCESS_KEY_ID:-}" && -n "${AWS_SECRET_ACCESS_KEY:-}" ]]; then
    return
  fi
  if [[ -z "${AWS_PROFILE:-}" ]]; then
    return
  fi

  local credential_env_file
  credential_env_file="${RUN_DIR}/aws-exported-credentials.env"
  rm -f "$credential_env_file"
  if ! aws configure export-credentials \
    --profile "$AWS_PROFILE" \
    --format env-no-export > "$credential_env_file"; then
    rm -f "$credential_env_file"
    echo "failed to export AWS CLI credentials for AWS_PROFILE=$AWS_PROFILE" >&2
    exit 1
  fi
  chmod 600 "$credential_env_file"
  load_aws_exported_credentials_file "$credential_env_file"
  rm -f "$credential_env_file"
  export AWS_ACCESS_KEY_ID
  export AWS_SECRET_ACCESS_KEY
  export AWS_SESSION_TOKEN
  export AWS_CREDENTIAL_EXPIRATION
}

credential_loader_self_test() {
  local env_tmp marker_tmp
  env_tmp="$(mktemp)"
  marker_tmp="$(mktemp)"
  rm -f "$marker_tmp"
  cat > "$env_tmp" <<EOF
AWS_ACCESS_KEY_ID=test-access-key
export AWS_SECRET_ACCESS_KEY="test secret"
AWS_SESSION_TOKEN='test token'
AWS_CREDENTIAL_EXPIRATION=2026-05-31T00:00:00Z
AWS_SELF_TEST_LITERAL=\$(touch "$marker_tmp")
EOF
  load_aws_exported_credentials_file "$env_tmp"
  [[ "${AWS_ACCESS_KEY_ID:-}" == "test-access-key" ]] || fail "credential loader self-test expected access key"
  [[ "${AWS_SECRET_ACCESS_KEY:-}" == "test secret" ]] || fail "credential loader self-test expected secret key"
  [[ "${AWS_SESSION_TOKEN:-}" == "test token" ]] || fail "credential loader self-test expected session token"
  [[ "${AWS_SELF_TEST_LITERAL:-}" == "\$(touch \"$marker_tmp\")" ]] || fail "credential loader self-test expected literal value"
  [[ ! -e "$marker_tmp" ]] || fail "credential loader self-test must not execute commands"
  rm -f "$env_tmp" "$marker_tmp"
  echo "credential loader self-test passed"
}
