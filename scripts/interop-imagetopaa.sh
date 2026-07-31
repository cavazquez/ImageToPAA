#!/usr/bin/env bash
# interop-imagetopaa.sh — Opt-in interoperability suite (issue #12).
#
# Requires:
#   IMAGETOPAA_PATH  — official ImageToPAA.exe (Windows or Wine)
# Optional:
#   TEXVIEW_PATH     — TexView 2 for PAA→PNG (manual checklist if unset)
#
# CI must NOT set these. Local / release gate only.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${IMAGE_TO_PAA_BIN:-}"
IMAGETOPAA_PATH="${IMAGETOPAA_PATH:-}"

if [[ -z "$BIN" ]]; then
  TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
  BIN="$TARGET_DIR/release/image-to-paa"
fi

if [[ -z "$IMAGETOPAA_PATH" ]]; then
  echo "SKIP: set IMAGETOPAA_PATH to run interop suite" >&2
  exit 0
fi
if [[ ! -x "$BIN" ]]; then
  echo "Building release binary…"
  (cd "$ROOT" && cargo build --release)
  TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
  BIN="$TARGET_DIR/release/image-to-paa"
fi

python3 "$ROOT/scripts/generate-fixtures.py"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

run_official() {
  local in="$1" out="$2"
  if command -v wine >/dev/null 2>&1 && [[ "$IMAGETOPAA_PATH" == *.exe ]]; then
    wine "$IMAGETOPAA_PATH" "$in" "$out"
  else
    "$IMAGETOPAA_PATH" "$in" "$out"
  fi
}

check_case() {
  local src_name="$1" fmt="$2" mips_flag="$3"
  local src="$ROOT/fixtures/sources/$src_name"
  local ours="$TMP/ours_${src_name%.png}.paa"
  local official="$TMP/official_${src_name%.png}.paa"

  local args=("$src" "$ours" --format "$fmt" --force)
  if [[ "$mips_flag" == "no-mips" ]]; then
    args+=(--no-mips)
  fi
  "$BIN" "${args[@]}"

  run_official "$src" "$official"

  local h_ours h_off
  h_ours=$(xxd -p -l 2 "$ours")
  h_off=$(xxd -p -l 2 "$official")
  if [[ "$h_ours" != "$h_off" ]]; then
    echo "FAIL $src_name: type bytes ours=$h_ours official=$h_off" >&2
    exit 1
  fi

  # Structural compare via our parser (dimensions / tag names), not BCn identity.
  cargo run --quiet --manifest-path "$ROOT/Cargo.toml" --example interop_compare -- "$ours" "$official" "$src_name"
  echo "OK $src_name ($fmt, $mips_flag)"
}

check_case opaque_blocks_16.png dxt1 mips
check_case soft_radial_16.png dxt5 mips
check_case soft_diagonal_8x16.png dxt5 no-mips
check_case opaque_gradient_16x8.png dxt1 mips

cat <<'EOF'
Interop structural checks passed.

Manual TexView 2 / Eden checklist (release gate):
  [ ] Open each ours-*.paa in TexView 2
  [ ] Compose soft alpha over white and black — no RGB halos
  [ ] Confirm mip chain length in TexView
  [ ] Drop one dialog + one icon PAA into Arma/Eden smoke test
EOF
