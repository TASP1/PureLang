# PureLang completion status (v0.34.0)

Honest snapshot of the five “complete all” tracks.

| Track | MVP shipped | Production-complete? |
|-------|-------------|----------------------|
| **Threads / channels** | `channel_new/send/recv/len`, `thread_spawn_send` (pthread) | Yes (MVP) — Unix pthread; Windows CreateThread + CRITICAL_SECTION/CONDITION_VARIABLE |
| **HTTPS / TLS** | `https://` via system `curl`; `http://` via sockets | Expanded — WinInet (Windows); OpenSSL on Unix (no CLI when libssl present) |
| **Native UI** | HTML export + `ui_native_available()==0` | Expanded — Win32 window+edit+listbox+button; HTML elsewhere; not full Cocoa/Android SDKs |
| **iOS / Android sysroots** | `scripts/android_build.sh`, `ios_build.sh`, `--platform` | Expanded — NDK job + packaging artifacts; not Play/App Store submission |
| **Self-hosting** | `bootstrap/` seed programs | Expanded — purec_mini.pure emits .ll for print N; production purec still Rust |

## What “done” means here

These foundations are **usable for demos and further development**. They are **not** substitutes for mature runtimes (Tokio, NSURLSession, SwiftUI, full purec-in-PureLang).

Next engineering priority: **`thread_spawn(fn)`** using function values; then Windows threads/channels; then self-host lexer growth.


## v0.34.0
- str_char_at / str_slice — bootstrap lexer progress
