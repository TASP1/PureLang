#!/usr/bin/env bash
# Produce an Android packaging tree (not Play Store upload).
# With NDK: builds arm64 shared lib; always writes Gradle skeleton under dist/android-app.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist/android-app"
mkdir -p "$DIST/app/src/main/java/com/purelang/app" "$DIST/app/src/main/res/values"

cat > "$DIST/settings.gradle" << 'G'
rootProject.name = "PureLangApp"
include ":app"
G

cat > "$DIST/build.gradle" << 'G'
buildscript {
    repositories { google(); mavenCentral() }
    dependencies { classpath "com.android.tools.build:gradle:8.2.0" }
}
allprojects { repositories { google(); mavenCentral() } }
G

cat > "$DIST/app/build.gradle" << 'G'
plugins { id "com.android.application" }
android {
    namespace "com.purelang.app"
    compileSdk 34
    defaultConfig {
        applicationId "com.purelang.app"
        minSdk 24
        targetSdk 34
        versionCode 1
        versionName "0.35.0"
    }
    buildTypes { release { minifyEnabled false } }
}
G

cat > "$DIST/app/src/main/AndroidManifest.xml" << 'G'
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
  <application android:label="PureLang" android:allowBackup="true">
    <activity android:name=".MainActivity" android:exported="true">
      <intent-filter>
        <action android:name="android.intent.action.MAIN"/>
        <category android:name="android.intent.category.LAUNCHER"/>
      </intent-filter>
    </activity>
  </application>
</manifest>
G

cat > "$DIST/app/src/main/java/com/purelang/app/MainActivity.java" << 'G'
package com.purelang.app;
import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;
public class MainActivity extends Activity {
  static { try { System.loadLibrary("pureapp"); } catch (Throwable t) {} }
  @Override protected void onCreate(Bundle b) {
    super.onCreate(b);
    TextView tv = new TextView(this);
    tv.setText("PureLang Android host");
    tv.setTextSize(20f);
    setContentView(tv);
  }
}
G

cat > "$DIST/app/src/main/res/values/strings.xml" << 'G'
<resources><string name="app_name">PureLang</string></resources>
G

cat > "$DIST/README.md" << 'G'
# PureLang Android packaging

This is a **Gradle app skeleton** ready for local `./gradlew assembleRelease`.

1. Build native lib: `ANDROID_NDK_HOME=... ./scripts/android_build.sh examples/hello.pure dist/android-app/app/src/main/jniLibs/arm64-v8a/libpureapp.so`
2. Open in Android Studio or run Gradle.
3. Play Store upload is **manual** (signing + Play Console) — not performed by CI.
G

if [[ -n "${ANDROID_NDK_HOME:-}" ]]; then
  mkdir -p "$DIST/app/src/main/jniLibs/arm64-v8a"
  bash "$ROOT/scripts/android_build.sh" "$ROOT/examples/hello.pure" \
    "$DIST/app/src/main/jniLibs/arm64-v8a/libpureapp.so" || true
fi
echo "Android package tree: $DIST"
