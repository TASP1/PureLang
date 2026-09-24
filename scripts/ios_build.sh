#!/usr/bin/env bash
# Compile PureLang to an iOS arm64 object (requires macOS + Xcode).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${1:?usage: ios_build.sh file.pure [out.o]}"
OUT="${2:-pureapp.o}"
PUREC="${ROOT}/compiler/target/release/purec"
if [[ ! -x "$PUREC" ]]; then
  (cd "$ROOT/compiler" && cargo build --release)
fi
"$PUREC" --emit-ir -o /tmp/pure_ios.ll "$SRC"
xcrun -sdk iphoneos clang -arch arm64 -c -O2 -o "$OUT" /tmp/pure_ios.ll
echo "Wrote $OUT (link with your Xcode app target)"
