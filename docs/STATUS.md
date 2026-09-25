# PureLang completion status (v0.32.0)

Honest snapshot of the five “complete all” tracks.

| Track | MVP shipped | Production-complete? |
|-------|-------------|----------------------|
| **Threads / channels** | `channel_new/send/recv/len`, `thread_spawn_send` (pthread) | Yes (MVP) — Unix pthread; Windows CreateThread + CRITICAL_SECTION/CONDITION_VARIABLE |
| **HTTPS / TLS** | `https://` via system `curl`; `http://` via sockets | MVP+ — WinInet (Windows); default -lcurl on Unix; curl CLI fallback |
| **Native UI** | HTML export + `ui_native_available()==0` | MVP+ — Win32 CreateWindow widgets; HTML/osascript/zenity elsewhere |
| **iOS / Android sysroots** | `scripts/android_build.sh`, `ios_build.sh`, `--platform` | MVP+ — CI installs NDK (Ubuntu) + checks iPhoneOS SDK (macOS) |
| **Self-hosting** | `bootstrap/` seed programs | Partial — lexer + parser_codegen subset; compiler still Rust |

## What “done” means here

These foundations are **usable for demos and further development**. They are **not** substitutes for mature runtimes (Tokio, NSURLSession, SwiftUI, full purec-in-PureLang).

Next engineering priority: **`thread_spawn(fn)`** using function values; then Windows threads/channels; then self-host lexer growth.


## v0.32.0
- str_char_at / str_slice — bootstrap lexer progress
