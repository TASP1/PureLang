#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$ROOT/dist/desktop}"
mkdir -p "$OUT"
PUREC="$ROOT/compiler/target/release/purec"
if [[ ! -x "$PUREC" ]]; then
  (cd "$ROOT/compiler" && cargo build --release)
fi
cp "$PUREC" "$OUT/purec"
"$PUREC" --compile -o "$OUT/hello" "$ROOT/examples/hello.pure"
"$PUREC" --compile -o "$OUT/maps" "$ROOT/examples/maps.pure" || true
cat > "$OUT/README.txt" << R
PureLang desktop package
- purec: compiler
- hello, maps: sample binaries
Build date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
R
echo "Packaged into $OUT"
ls -la "$OUT"
