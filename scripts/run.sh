#!/usr/bin/env bash
# Runs easyuv-lint for the GitHub Action: annotations on the files, a job summary,
# an optional HTML report and the counts as step outputs.
set -uo pipefail

args=(--format github --lang "${EUL_LANG:-en}")
[ -n "${EUL_PRESET:-}" ] && args+=(--preset "$EUL_PRESET")
[ -n "${EUL_CONFIG:-}" ] && args+=(--config "$EUL_CONFIG")
[ -n "${EUL_TEXTURE:-}" ] && args+=(--texture-size "$EUL_TEXTURE")
[ "${EUL_STRICT:-false}" = "true" ] && args+=(--strict)
[ -n "${EUL_REPORT:-}" ] && args+=(--html "$EUL_REPORT")
[ -n "${GITHUB_STEP_SUMMARY:-}" ] && args+=(--markdown "$GITHUB_STEP_SUMMARY")

json="${RUNNER_TEMP:-/tmp}/easyuv-lint.json"
args+=(--json "$json")

# paths may be separated by spaces or new lines
read -r -d '' -a paths <<< "${EUL_PATHS:-.}" || true
[ "${#paths[@]}" -eq 0 ] && paths=(.)

easyuv-lint "${paths[@]}" "${args[@]}"
code=$?

if [ -f "$json" ]; then
  read -r errors warnings passed < <(python3 - "$json" <<'PY' 2>/dev/null || echo "0 0 false"
import json, sys
s = json.load(open(sys.argv[1], encoding="utf-8"))["summary"]
print(s["errors"], s["warnings"], str(s["passed"]).lower())
PY
)
  {
    echo "errors=$errors"
    echo "warnings=$warnings"
    echo "passed=$passed"
  } >> "$GITHUB_OUTPUT"
fi
echo "exit-code=$code" >> "$GITHUB_OUTPUT"
exit 0
