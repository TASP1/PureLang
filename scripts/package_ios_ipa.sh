#!/usr/bin/env bash
# iOS packaging skeleton (not App Store upload).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist/ios-app"
mkdir -p "$DIST"
cat > "$DIST/README.md" << 'G'
# PureLang iOS packaging

1. On macOS: `./scripts/ios_build.sh examples/hello.pure dist/ios-app/pureapp.o`
2. Create an Xcode app target and link `pureapp.o` + runtime.
3. Archive → Organizer → App Store Connect (manual signing).

CI only validates SDK availability; it does **not** submit builds.
G
if [[ "$(uname -s)" == "Darwin" ]]; then
  bash "$ROOT/scripts/ios_build.sh" "$ROOT/examples/hello.pure" "$DIST/pureapp.o" || true
fi
echo "iOS package tree: $DIST"
