#!/usr/bin/env bash
# Build a PureLang program as an Android arm64 shared library (requires ANDROID_NDK).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${1:?usage: android_build.sh file.pure [out.so]}"
OUT="${2:-libpureapp.so}"
if [[ -z "${ANDROID_NDK:-}" ]]; then
  echo "Set ANDROID_NDK to your NDK root" >&2
  exit 1
fi
PUREC="${ROOT}/compiler/target/release/purec"
if [[ ! -x "$PUREC" ]]; then
  (cd "$ROOT/compiler" && cargo build --release)
fi
"$PUREC" --emit-ir -o /tmp/pure_android.ll "$SRC"
# Prefer llvm toolchain in NDK
PREBUILT=$(echo "$ANDROID_NDK"/toolchains/llvm/prebuilt/* | awk '{print $1}')
CLANG="$PREBUILT/bin/aarch64-linux-android24-clang"
SYSROOT="$PREBUILT/sysroot"
if [[ ! -x "$CLANG" ]]; then
  CLANG=clang
fi
"$CLANG" --target=aarch64-linux-android24 --sysroot="$SYSROOT" -shared -fPIC -O2 \
  -o "$OUT" /tmp/pure_android.ll "$ROOT/runtime/purelang_rt.c" -lm 2>/dev/null \
  || "$CLANG" --target=aarch64-linux-android24 -shared -fPIC -O2 \
  -o "$OUT" /tmp/pure_android.ll "$ROOT/runtime/purelang_rt.c" -lm
echo "Wrote $OUT"
