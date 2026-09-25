# PureLang Android packaging

This is a **Gradle app skeleton** ready for local `./gradlew assembleRelease`.

1. Build native lib: `ANDROID_NDK_HOME=... ./scripts/android_build.sh examples/hello.pure dist/android-app/app/src/main/jniLibs/arm64-v8a/libpureapp.so`
2. Open in Android Studio or run Gradle.
3. Play Store upload is **manual** (signing + Play Console) — not performed by CI.
