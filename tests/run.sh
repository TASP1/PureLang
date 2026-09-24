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
  exp="tests/expected/${name}.out"
  echo "==> $name"
  if ! "$PUREC" --compile -o "/tmp/pl_test_$name" "$f"; then
    echo "  COMPILE FAIL"
    fail=1
    continue
  fi
  "/tmp/pl_test_$name" >"/tmp/pl_got_$name.txt" 2>&1 || true
  if [[ -f "$exp" ]]; then
    if diff -u "$exp" "/tmp/pl_got_$name.txt"; then
      echo "  output OK"
    else
      echo "  OUTPUT MISMATCH"
      fail=1
    fi
  else
    echo "  compile OK (no expected file)"
  fi
done
exit $fail
