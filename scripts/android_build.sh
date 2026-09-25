#!/usr/bin/env bash
# Build a PureLang program as an Android arm64 shared library (requires ANDROID_NDK).
# Usage: android_build.sh file.pure [out.so]
#        android_build.sh --check   # CI: validate environment / dry-run
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [[ "${1:-}" == "--check" ]]; then
  echo "android_build.sh check"
  test -f "$ROOT/runtime/purelang_rt.c"
  test -d "$ROOT/compiler"
  if [[ -n "${ANDROID_NDK_HOME:-}${ANDROID_NDK_ROOT:-}" ]]; then
    echo "NDK: ${ANDROID_NDK_HOME:-$ANDROID_NDK_ROOT}"
  else
    echo "NDK: not set (install Android NDK and export ANDROID_NDK_HOME)"
  fi
  echo "OK dry-run"
  exit 0
fi

SRC="${1:?usage: android_build.sh file.pure [out.so] | android_build.sh --check}"
OUT="${2:-libpureapp.so}"
PUREC="${ROOT}/compiler/target/release/purec"
if [[ ! -x "$PUREC" ]]; then
  (cd "$ROOT/compiler" && cargo build --release)
fi
NDK="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [[ -z "$NDK" ]]; then
  echo "error: set ANDROID_NDK_HOME to your NDK path" >&2
  exit 1
fi
# Prefer llvm toolchain under NDK
CLANG=$(find "$NDK" -name 'aarch64-linux-android*-clang' 2>/dev/null | head -1 || true)
if [[ -z "$CLANG" ]]; then
  CLANG=$(find "$NDK" -path '*toolchains/llvm*' -name 'clang' 2>/dev/null | head -1 || true)
fi
if [[ -z "$CLANG" ]]; then
  echo "error: could not find NDK clang under $NDK" >&2
  exit 1
fi
"$PUREC" --emit-ir -o /tmp/pure_android.ll "$SRC"
"$CLANG" --target=aarch64-linux-android24 -shared -fPIC -O2 \
  -o "$OUT" /tmp/pure_android.ll "$ROOT/runtime/purelang_rt.c" -lm \
  || "$CLANG" -shared -fPIC -O2 -o "$OUT" /tmp/pure_android.ll "$ROOT/runtime/purelang_rt.c" -lm
echo "Wrote $OUT"
