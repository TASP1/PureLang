# Store signing & release (manual — not auto-upload)

PureLang CI **builds and packages** artifacts. It does **not** submit to Google Play or App Store Connect.
Signing and upload stay on your machine / CI secrets you control.

## Android (Play Store)

1. **Package tree:** `./scripts/package_android_aab.sh` → `dist/android-app/`
2. **Native lib (optional):** set `ANDROID_NDK_HOME` and rebuild so `jniLibs/arm64-v8a/libpureapp.so` exists.
3. **Keystore (once):**
   ```bash
   keytool -genkey -v -keystore purelang-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias purelang
   ```
4. **Gradle signing** — in `app/build.gradle` (do not commit passwords):
   ```gradle
   android {
     signingConfigs {
       release {
         storeFile file(System.getenv("PURELANG_KEYSTORE") ?: "purelang-release.jks")
         storePassword System.getenv("PURELANG_KEYSTORE_PASSWORD")
         keyAlias "purelang"
         keyPassword System.getenv("PURELANG_KEY_PASSWORD")
       }
     }
     buildTypes { release { signingConfig signingConfigs.release } }
   }
   ```
5. **Build AAB:**
   ```bash
   cd dist/android-app && ./gradlew bundleRelease
   # output: app/build/outputs/bundle/release/app-release.aab
   ```
6. **Upload:** Play Console → your app → Production/Testing → Create release → upload AAB.  
   Or `fastlane supply` with a service account JSON (never commit the JSON).

## iOS (App Store)

1. **Package:** `./scripts/package_ios_ipa.sh` (macOS + Xcode).
2. **Xcode:** New App target → link `pureapp.o` + runtime → set Team & Bundle ID.
3. **Signing:** Xcode → Signing & Capabilities → Automatic or Distribution certificate + App Store profile.
4. **Archive:** Product → Archive → Distribute App → App Store Connect.
5. **CLI alternative:**
   ```bash
   xcodebuild -scheme YourApp -configuration Release -archivePath build/App.xcarchive archive
   xcodebuild -exportArchive -archivePath build/App.xcarchive -exportOptionsPlist ExportOptions.plist -exportPath build/ipa
   xcrun altool --upload-app -f build/ipa/*.ipa -t ios -u "$APPLE_ID" -p "$APP_SPECIFIC_PASSWORD"
   ```
6. CI may store `APPLE_ID` / app-specific password as secrets; **this repo does not upload**.

## Desktop

`./scripts/package_desktop.sh [outdir]` produces `purec` + sample binaries. Code-sign on macOS with `codesign` / notarize separately if distributing outside the App Store.

## Security

- Never commit keystores, `.p12`, or Play service accounts.
- Prefer environment variables / GitHub Actions secrets for passwords.
