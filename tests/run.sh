#!/usr/bin/env bash
# Golden output tests for PureLang examples (Phase A solidity)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PUREC="${PUREC:-$ROOT/compiler/target/release/purec}"
if [[ ! -x "$PUREC" ]]; then
  (cd "$ROOT/compiler" && cargo build --release)
  PUREC="$ROOT/compiler/target/release/purec"
fi
cd "$ROOT"

# Skip flaky / interactive / network-bound examples in CI
SKIP_REGEX='^(channels|thread_spawn|https_get|ui_demo|ui_native|net_sleep|time_exit)$'

fail=0
pass=0
skip=0
for f in examples/*.pure; do
  name=$(basename "$f" .pure)
  if echo "$name" | grep -qE "$SKIP_REGEX"; then
    echo "==> $name (skipped)"
    skip=$((skip + 1))
    continue
  fi
  exp="tests/expected/${name}.out"
  echo "==> $name"
  if ! "$PUREC" --compile -o "/tmp/pl_test_$name" "$f" >/tmp/pl_compile_$name.txt 2>&1; then
    echo "  COMPILE FAIL"
    tail -5 /tmp/pl_compile_$name.txt || true
    fail=$((fail + 1))
    continue
  fi
  timeout 5 "/tmp/pl_test_$name" >"/tmp/pl_got_$name.txt" 2>&1 || true
  if [[ -f "$exp" ]]; then
    if diff -u "$exp" "/tmp/pl_got_$name.txt"; then
      echo "  output OK"
      pass=$((pass + 1))
    else
      echo "  OUTPUT MISMATCH"
      fail=$((fail + 1))
    fi
  else
    echo "  compile OK (no expected file — add tests/expected/${name}.out)"
    pass=$((pass + 1))
  fi
done
echo "--- pass=$pass fail=$fail skip=$skip ---"
exit $fail
