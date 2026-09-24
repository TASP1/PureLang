# PureLang Architecture

## Vision

Systems language for extreme performance (games, engines, AI), memory safety without GC, and syntax simpler than Python — native on desktop, web (WASM), and eventually mobile/consoles.

## Pipeline

```
.pure → Lexer → Parser → AST
         → Type / ownership checker
         → Codegen
              ├─ LLVM IR → clang → native (Linux / macOS / Windows)
              └─ WASI .wat → wasmtime / browsers
```

## Compiler

- Written in **Rust** (`purec` **v0.23.0**)
- Hand-written recursive-descent parser
- LLVM IR as text; link with system `clang` + **purelang_rt.c**
- Optional LSP and package manager built into the same binary

## Design choices

| Area | Choice |
|------|--------|
| Syntax | Braces, immutable by default, minimal ceremony |
| Safety | Ownership + borrows at compile time |
| Numbers | `Number` (i64) + `Float` (f64) |
| Collections | Built-in lists; maps via runtime |
| UI (MVP) | Emit static HTML (`purelang_ui.html`) |
| Platforms | `--platform` presets + `ANDROID_NDK` / `PUREC_SYSROOT` |

## Platform strategy

- **Desktop:** CI on Ubuntu, Windows, macOS
- **Web:** WASM / WASI
- **Mobile / console:** IR + clang triples; sysroots required for real devices

## Extensibility

Future: richer stdlib, native UI backends, concurrency, self-hosting.
