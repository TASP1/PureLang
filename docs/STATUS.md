# PureLang completion status (v0.37.0)

**Plan:** see [ROADMAP.md](ROADMAP.md) for phases A–E and implementation order.

Honest snapshot of language, tooling, and product tracks.

## Overall completion (weighted estimate)

| Area | Weight | Done | Notes |
|------|--------|------|-------|
| Core language (syntax → native) | 25% | **90%** | Full purec in Rust; gaps: richer types, GC-free edge cases |
| Stdlib / runtime | 15% | **75%** | Maps, strings, files, HTTPS, threads; partial UI |
| Self-host purec_sub | 15% | **45%** | if/while, multi-letter ids, + , file in; no modules/types/fns yet |
| Tooling (LSP, fmt, pkg, CI) | 15% | **80%** | CI multi-OS, NDK/iOS checks, packages |
| Mobile hosts | 10% | **40%** | Gradle host Activity + JNI slot; Cocoa alert path |
| Store pipelines | 10% | **35%** | Package trees + **STORE_SIGNING.md**; no auto-upload |
| Native UI SDKs | 10% | **30%** | Win32 widgets; HTML; not full UIKit/Jetpack |

**Weighted overall ≈ 62%** toward a “production systems language product.”  
**Compiler usable today ≈ 85%** for learning/demos/native tools on desktop.

## Track detail

| Track | Status |
|-------|--------|
| Threads / channels | MVP complete (Unix + Windows) |
| HTTPS / TLS | OpenSSL Unix + WinInet Windows |
| Native UI | Win32 + HTML + Cocoa alert helper |
| iOS / Android | Scripts + Gradle host + signing docs |
| Self-hosting | purec_sub: assign, print, +, **if**, **while**, multi-letter ids |

## v0.37.0
- purec_sub: **if** / **while** (compile-time interpret), multi-letter identifiers
- sample `bootstrap/samples/control.pure` → IR prints 10, 1, 42
- docs/STORE_SIGNING.md (Play AAB + iOS archive, no auto-upload)
- Deeper Android MainActivity (JNI run button); `ui_cocoa_alert`

## v0.36.0
- purec_sub arithmetic + file input; letter/digit parse fix

## Prior
See git history from v0.28–v0.35 (function values, OpenSSL, Win32, packaging, golden tests).
