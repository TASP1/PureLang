#!/usr/bin/env bash
# Compile PureLang to an iOS arm64 object (requires macOS + Xcode).
# Usage: ios_build.sh file.pure [out.o]
#        ios_build.sh --check
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [[ "${1:-}" == "--check" ]]; then
  echo "ios_build.sh check"
  test -f "$ROOT/runtime/purelang_rt.c"
  if [[ "$(uname -s)" == "Darwin" ]]; then
    xcrun -sdk iphoneos --show-sdk-path >/dev/null 2>&1 && echo "iphoneos SDK: ok" || echo "iphoneos SDK: missing"
  else
    echo "host: not macOS — iOS link step skipped on this OS"
  fi
  echo "OK dry-run"
  exit 0
fi

SRC="${1:?usage: ios_build.sh file.pure [out.o] | ios_build.sh --check}"
OUT="${2:-pureapp.o}"
PUREC="${ROOT}/compiler/target/release/purec"
if [[ ! -x "$PUREC" ]]; then
  (cd "$ROOT/compiler" && cargo build --release)
fi
"$PUREC" --emit-ir -o /tmp/pure_ios.ll "$SRC"
if [[ "$(uname -s)" == "Darwin" ]]; then
  xcrun -sdk iphoneos clang -arch arm64 -c -O2 -o "$OUT" /tmp/pure_ios.ll
  echo "Wrote $OUT (link with your Xcode app target)"
else
  echo "error: iOS object build requires macOS + Xcode" >&2
  exit 1
fi
