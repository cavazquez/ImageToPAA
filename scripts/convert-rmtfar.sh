#!/usr/bin/env bash
# convert-rmtfar.sh — Convert the eight RMTFAR radio textures without Wine (#17).
#
# Usage:
#   ./scripts/convert-rmtfar.sh /path/to/rmtfar [output-dir]
#
# Does NOT modify rmtfar/addon/ui/radios. Writes to a temp/output directory and
# prints SHA-256 hashes + encoder version for the promotion checklist.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RMTFAR="${1:-}"
OUT="${2:-}"

resolve_bin() {
  if [[ -n "${IMAGE_TO_PAA_BIN:-}" && -x "$IMAGE_TO_PAA_BIN" ]]; then
    echo "$IMAGE_TO_PAA_BIN"
    return
  fi
  local target_dir="${CARGO_TARGET_DIR:-$ROOT/target}"
  if [[ -x "$target_dir/release/image-to-paa" ]]; then
    echo "$target_dir/release/image-to-paa"
    return
  fi
  (cd "$ROOT" && cargo build --release >/dev/null)
  target_dir="${CARGO_TARGET_DIR:-$ROOT/target}"
  echo "$target_dir/release/image-to-paa"
}

BIN="$(resolve_bin)"
if [[ ! -x "$BIN" ]]; then
  echo "No se encontró image-to-paa en $BIN" >&2
  exit 1
fi

if [[ -z "$RMTFAR" ]]; then
  echo "Uso: $0 /ruta/a/rmtfar [directorio-salida]" >&2
  exit 2
fi

ART="$RMTFAR/artwork/radios"
if [[ ! -d "$ART" ]]; then
  echo "No se encontró artwork/radios en $RMTFAR" >&2
  exit 1
fi

if [[ -z "$OUT" ]]; then
  OUT="$(mktemp -d /tmp/imagetopaa-rmtfar.XXXXXX)"
else
  mkdir -p "$OUT"
fi

VERSION="$("$BIN" --version)"
echo "encoder: $VERSION"
echo "output:  $OUT"

declare -a EXPORTS=(
  "r210_west_paa_source.png:r210_west.paa"
  "r210_west_icon_paa_source.png:r210_west_icon.paa"
  "r320_east_paa_source.png:r320_east.paa"
  "r320_east_icon_paa_source.png:r320_east_icon.paa"
  "r110_independent_paa_source.png:r110_independent.paa"
  "r110_independent_icon_paa_source.png:r110_independent_icon.paa"
  "r6000_lr_paa_source.png:r6000_lr.paa"
  "r6000_lr_icon_paa_source.png:r6000_lr_icon.paa"
)

HASHES="$OUT/SHA256SUMS"
: >"$HASHES"
REPORT="$OUT/CONVERT_REPORT.txt"
{
  echo "encoder: $VERSION"
  echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "rmtfar: $RMTFAR"
  echo "command_template: image-to-paa <src> <dst> --format dxt5 --force"
} >"$REPORT"

for entry in "${EXPORTS[@]}"; do
  src_name="${entry%%:*}"
  dst_name="${entry##*:}"
  src="$ART/$src_name"
  dst="$OUT/$dst_name"
  [[ -s "$src" ]] || { echo "Falta fuente: $src" >&2; exit 1; }
  echo "Convirtiendo $src_name -> $dst_name"
  "$BIN" "$src" "$dst" --format dxt5 --force
  # Verify header is DXT5
  hdr=$(xxd -p -l 2 "$dst")
  if [[ "$hdr" != "05ff" ]]; then
    echo "FAIL $dst_name: expected 05ff got $hdr" >&2
    exit 1
  fi
  sha256sum "$dst" | tee -a "$HASHES" >>"$REPORT"
done

echo "Listo. Hashes en $HASHES"
echo "Promoción a addon/ui/radios: paso explícito en el repo RMTFAR (backup primero)."
