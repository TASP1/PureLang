#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PUREC="${PUREC:-$ROOT/compiler/target/release/purec}"
if [[ ! -x "$PUREC" ]]; then
  (cd "$ROOT/compiler" && cargo build --release)
  PUREC="$ROOT/compiler/target/release/purec"
fi
cd "$ROOT"
fail=0
for f in examples/*.pure; do
  name=$(basename "$f" .pure)
  echo "==> $name"
  if ! "$PUREC" --compile -o "/tmp/pl_$name" "$f"; then
    echo "COMPILE FAIL: $name"
    fail=1
    continue
  fi
  "/tmp/pl_$name" >/tmp/pl_out_$name.txt 2>&1 || true
  echo "    ok"
done
exit $fail
