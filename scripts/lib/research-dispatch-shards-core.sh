# shellcheck shell=bash

positive_integer_arg() {
  local name="$1"
  local value="$2"
  if ! [[ "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

require_absolute_path() {
  local name="$1"
  local value="$2"
  case "$value" in
    /*) ;;
    *)
      echo "$name must be an absolute path; got $value" >&2
      exit 1
      ;;
  esac
}

require_absolute_file() {
  local name="$1"
  local value="$2"
  require_absolute_path "$name" "$value"
  if [[ ! -f "$value" ]]; then
    echo "$name does not exist: $value" >&2
    exit 1
  fi
}

bool_is_true() {
  local lowered
  lowered="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')"
  case "$lowered" in
    1 | true | yes) return 0 ;;
    *) return 1 ;;
  esac
}
