# PureLang iOS packaging

1. On macOS: `./scripts/ios_build.sh examples/hello.pure dist/ios-app/pureapp.o`
2. Create an Xcode app target and link `pureapp.o` + runtime.
3. Archive → Organizer → App Store Connect (manual signing).

CI only validates SDK availability; it does **not** submit builds.
