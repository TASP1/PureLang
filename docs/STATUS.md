# PureLang completion status (v0.31.0)

Honest snapshot of the five “complete all” tracks.

| Track | MVP shipped | Production-complete? |
|-------|-------------|----------------------|
| **Threads / channels** | `channel_new/send/recv/len`, `thread_spawn_send` (pthread) | Yes (MVP) — Unix pthread; Windows CreateThread + CRITICAL_SECTION/CONDITION_VARIABLE |
| **HTTPS / TLS** | `https://` via system `curl`; `http://` via sockets | MVP — WinInet (Windows); libcurl if PURELANG_HAVE_CURL=1; else curl CLI |
| **Native UI** | HTML export + `ui_native_available()==0` | MVP — ui_alert (Win32/osascript/zenity) + HTML export |
| **iOS / Android sysroots** | `scripts/android_build.sh`, `ios_build.sh`, `--platform` | MVP — scripts --check in CI; full NDK/Xcode still host-local |
| **Self-hosting** | `bootstrap/` seed programs | Partial — bootstrap/lexer.pure; compiler still Rust |

## What “done” means here

These foundations are **usable for demos and further development**. They are **not** substitutes for mature runtimes (Tokio, NSURLSession, SwiftUI, full purec-in-PureLang).

Next engineering priority: **`thread_spawn(fn)`** using function values; then Windows threads/channels; then self-host lexer growth.


## v0.31.0
- str_char_at / str_slice — bootstrap lexer progress
