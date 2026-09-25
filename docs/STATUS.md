# PureLang completion status (v0.28.1)

Honest snapshot of the five “complete all” tracks.

| Track | MVP shipped | Production-complete? |
|-------|-------------|----------------------|
| **Threads / channels** | `channel_new/send/recv/len`, `thread_spawn_send` (pthread) | Partial — Unix pthread; Windows stubs (non-blocking); function values enable future spawn(fn) |
| **HTTPS / TLS** | `https://` via system `curl`; `http://` via sockets | No — not in-process TLS (OpenSSL/rustls) |
| **Native UI** | HTML export + `ui_native_available()==0` | No — no Win32/Cocoa/Android toolkits |
| **iOS / Android sysroots** | `scripts/android_build.sh`, `ios_build.sh`, `--platform` | No — no CI device matrix or store packaging |
| **Self-hosting** | `bootstrap/` seed programs | No — compiler still Rust-only |

## What “done” means here

These foundations are **usable for demos and further development**. They are **not** substitutes for mature runtimes (Tokio, NSURLSession, SwiftUI, full purec-in-PureLang).

Next engineering priority: deepen one track at a time (likely channels → real `spawn(fn)` once function values exist).


## v0.28.1
- str_char_at / str_slice — bootstrap lexer progress
