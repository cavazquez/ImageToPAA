#!/usr/bin/env bash
# regenerate-oracle.sh — Produce an oracle manifest using official ImageToPAA.
#
# Usage (Windows or Wine):
#   export IMAGETOPAA_PATH=/path/to/ImageToPAA.exe
#   ./scripts/regenerate-oracle.sh
#
# The proprietary executable is NEVER committed. Generated .paa binaries are
# NOT committed by default (see fixtures/README.md). This script writes:
#   fixtures/oracle/manifest.json  — hashes, headers, tool version/options
#   fixtures/oracle/*.png          — optional TexView decode if TEXVIEW_PATH set
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMAGETOPAA_PATH="${IMAGETOPAA_PATH:-}"
OUT="$ROOT/fixtures/oracle"
SRC="$ROOT/fixtures/sources"

if [[ -z "$IMAGETOPAA_PATH" ]]; then
  echo "Set IMAGETOPAA_PATH to ImageToPAA.exe" >&2
  exit 2
fi
if [[ ! -f "$IMAGETOPAA_PATH" ]]; then
  echo "IMAGETOPAA_PATH not found: $IMAGETOPAA_PATH" >&2
  exit 1
fi

python3 "$ROOT/scripts/generate-fixtures.py"
mkdir -p "$OUT"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

run_convert() {
  local in="$1" out="$2"
  if command -v wine >/dev/null 2>&1 && [[ "$IMAGETOPAA_PATH" == *.exe ]]; then
    wine "$IMAGETOPAA_PATH" "$in" "$out"
  else
    "$IMAGETOPAA_PATH" "$in" "$out"
  fi
}

MANIFEST="$OUT/manifest.json"
{
  echo '{'
  echo "  \"generator\": \"scripts/regenerate-oracle.sh\","
  echo "  \"imagetopaa_path\": $(python3 -c 'import json,os; print(json.dumps(os.environ["IMAGETOPAA_PATH"]))'),"
  echo "  \"note\": \"PAA binaries are local-only unless redistribution is confirmed; commit hashes + decoded PNG.\","
  echo '  "cases": ['
} >"$MANIFEST"

first=1
for src in \
  "opaque_blocks_16.png:dxt1:mips" \
  "soft_radial_16.png:dxt5:mips" \
  "soft_diagonal_8x16.png:dxt5:no-mips"
do
  name="${src%%:*}"
  rest="${src#*:}"
  expect_fmt="${rest%%:*}"
  mips="${rest##*:}"
  in="$SRC/$name"
  base="${name%.png}"
  paa="$TMP/${base}.paa"
  run_convert "$in" "$paa"
  sha_in=$(sha256sum "$in" | awk '{print $1}')
  sha_out=$(sha256sum "$paa" | awk '{print $1}')
  header=$(xxd -p -l 2 "$paa")
  [[ $first -eq 1 ]] || echo ',' >>"$MANIFEST"
  first=0
  cat >>"$MANIFEST" <<EOF
    {
      "source": "fixtures/sources/$name",
      "source_sha256": "$sha_in",
      "paa_sha256": "$sha_out",
      "header_hex": "$header",
      "expected_format": "$expect_fmt",
      "mips_mode": "$mips",
      "provenance": "synthetic fixture; oracle via ImageToPAA at \$IMAGETOPAA_PATH"
    }
EOF
  # Keep PAA out of the repo copy by default.
done

cat >>"$MANIFEST" <<'EOF'

  ]
}
EOF

echo "Wrote $MANIFEST"
echo "Oracle PAAs left in $TMP (not copied). Review redistribution policy in fixtures/README.md before committing binaries."
