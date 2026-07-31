#!/usr/bin/env bash
# check.sh — Quality gate for ImageToPAA (mirrors CI).
#
# Steps (all must pass for exit 0):
#   1. fixtures   — regenerate synthetic PNGs
#   2. rustfmt    — cargo fmt --check
#   3. clippy     — -D warnings
#   4. tests      — cargo test --all-targets --locked
#   5. lockfile   — cargo metadata --locked
#   6. snapcraft  — YAML present (snapcraft pack skipped unless SNAPCRAFT=1)
#
# Usage:
#   ./check.sh           # normal run
#   ./check.sh --fix     # auto-format before checking
#   SNAPCRAFT=1 ./check.sh  # also run `snapcraft pack` if snapcraft is installed
#   QUIET=1 ./check.sh   # quieter cargo output

set -uo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

FIX=0
for arg in "$@"; do
  case "$arg" in
    --fix) FIX=1 ;;
    --help|-h)
      echo "Usage: $0 [--fix]"
      echo "  --fix   Auto-format with rustfmt before checking."
      echo "  SNAPCRAFT=1  Also pack the snap if snapcraft is available."
      exit 0
      ;;
    *)
      echo "Unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

if [[ -t 1 ]]; then
  RED=$'\033[0;31m' GRN=$'\033[0;32m' YEL=$'\033[0;33m'
  CYN=$'\033[0;36m' BLD=$'\033[1m' RST=$'\033[0m'
else
  RED='' GRN='' YEL='' CYN='' BLD='' RST=''
fi

PASS="${GRN}✔ PASS${RST}"
FAIL="${RED}✘ FAIL${RST}"
SKIP="${YEL}– SKIP${RST}"

ERRORS=0
SKIPPED=0
START_TIME=$(date +%s)

banner() { echo; echo "${CYN}${BLD}── $* ──${RST}"; }
ok()     { echo "  ${PASS}  $*"; }
fail()   { echo "  ${FAIL}  $*"; ERRORS=$((ERRORS + 1)); }
skip()   { echo "  ${SKIP}  $*"; SKIPPED=$((SKIPPED + 1)); }

run() {
  if [[ "${QUIET:-0}" == "1" ]]; then
    "$@" >/dev/null
  else
    "$@"
  fi
}

banner "Environment"
echo "  Rust  : $(rustc --version 2>/dev/null || echo missing)"
echo "  Cargo : $(cargo --version 2>/dev/null || echo missing)"
echo "  Python: $(python3 --version 2>/dev/null || echo missing)"

if ! command -v cargo >/dev/null 2>&1; then
  fail "cargo not found"
  exit 1
fi

banner "Fixtures"
if python3 scripts/generate-fixtures.py; then
  ok "scripts/generate-fixtures.py"
else
  fail "generate-fixtures.py"
fi

banner "rustfmt"
if [[ $FIX -eq 1 ]]; then
  if run cargo fmt --all; then
    ok "cargo fmt --all"
  else
    fail "cargo fmt --all"
  fi
fi
if run cargo fmt --all -- --check; then
  ok "cargo fmt --all -- --check"
else
  fail "cargo fmt --check (try ./check.sh --fix)"
fi

banner "clippy"
if run cargo clippy --all-targets -- -D warnings; then
  ok "cargo clippy --all-targets -- -D warnings"
else
  fail "clippy"
fi

banner "tests"
if run cargo test --all-targets --locked; then
  ok "cargo test --all-targets --locked"
else
  fail "tests"
fi

banner "lockfile"
if cargo metadata --locked --format-version 1 >/dev/null; then
  ok "cargo metadata --locked"
else
  fail "lockfile / metadata --locked"
fi

banner "snapcraft"
if [[ ! -f snap/snapcraft.yaml ]]; then
  fail "missing snap/snapcraft.yaml"
elif [[ "${SNAPCRAFT:-0}" == "1" ]]; then
  if command -v snapcraft >/dev/null 2>&1; then
    if run snapcraft pack; then
      ok "snapcraft pack"
    else
      fail "snapcraft pack"
    fi
  else
    skip "snapcraft not installed (SNAPCRAFT=1 requested)"
  fi
else
  ok "snap/snapcraft.yaml present (set SNAPCRAFT=1 to pack)"
fi

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
echo
if [[ $ERRORS -eq 0 ]]; then
  echo "${GRN}${BLD}check.sh: OK${RST} (${ELAPSED}s, skipped=$SKIPPED)"
  exit 0
fi
echo "${RED}${BLD}check.sh: FAILED${RST} ($ERRORS error(s), ${ELAPSED}s, skipped=$SKIPPED)" >&2
exit 1
