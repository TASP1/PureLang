# PureLang Platforms

**purec v0.12.0** targets production use across major desktop platforms and the web.

## Supported today

| Platform | Native binary | WebAssembly | CI |
|----------|:-------------:|:-----------:|:--:|
| **Linux** (x86_64, aarch64) | ✅ | ✅ | ✅ `ubuntu-latest` |
| **macOS** (Intel + Apple Silicon) | ✅ | ✅ | ✅ `macos-latest` |
| **Windows** (x86_64) | ✅ | ✅ | ✅ `windows-latest` |
| **Browsers / WASI** | — | ✅ `.wat` | ✅ emit step |

## Requirements

| Component | Linux | macOS | Windows |
|-----------|-------|-------|---------|
| Rust stable | ✅ | ✅ | ✅ |
| **clang** (to link `.ll` → binary) | `apt install clang` | Xcode CLT / `brew install llvm` | [LLVM](https://llvm.org/) or `winget install LLVM.LLVM` |

Without clang you can still:

```bash
purec --emit-ir app.pure          # LLVM IR only
purec --emit-wasm -o app.wat app.pure
```

## Compiler flags

```bash
# Host platform (default), -O2
purec --compile -o app app.pure

# Optimization
purec --compile --opt 3 -o app app.pure
purec --compile --opt s -o app app.pure   # size

# Explicit target triple
purec --compile --target x86_64-unknown-linux-gnu -o app app.pure
purec --compile --target arm64-apple-darwin -o app app.pure
purec --compile --target x86_64-pc-windows-msvc -o app.exe app.pure

# WebAssembly (WASI)
purec --emit-wasm -o app.wat app.pure
wasmtime app.wat
```

Host triples are auto-detected (`x86_64-unknown-linux-gnu`, `arm64-apple-darwin`, `x86_64-pc-windows-msvc`, …).

## Linking notes

- **Unix (Linux/macOS):** links with `-lm` (libm) for math stdlib.
- **Windows:** math comes from the C runtime; no `-lm`.
- LLVM IR embeds `target triple = "..."` matching `--target`.

## Cross-compilation

You can pass any clang-supported triple with `--target`. You need a matching clang sysroot / toolchain for true cross builds (e.g. Linux → Windows). Same-host builds on Linux, macOS, and Windows are covered by CI.

## Mobile & consoles (roadmap)

| Target | Status |
|--------|--------|
| iOS / Android | Planned (C ABI + NDK / Xcode) |
| Game consoles | Planned (vendor SDKs + C interop) |

## CI

GitHub Actions runs on **ubuntu-latest**, **windows-latest**, and **macos-latest** (public repo → free minutes). Artifacts upload `purec` for each OS.


## `--platform` presets (v0.15+)

```bash
purec --list-platforms
purec --compile --platform android -o libapp.so app.pure   # needs ANDROID_NDK
purec --compile --platform ios -o app.o app.pure           # needs Xcode (macOS)
purec --compile --platform console -o game app.pure
```

Environment:

| Variable | Purpose |
|----------|---------|
| `ANDROID_NDK` | NDK root; purec auto-adds matching llvm sysroot when present |
| `PUREC_SYSROOT` | Explicit clang `--sysroot` for any cross target |

## Language Server

```bash
purec --lsp
```

stdio JSON-RPC. Supports `initialize`, `textDocument/didOpen`, `didChange`, `publishDiagnostics`, `hover`.

VS Code: point a generic LSP client at `purec --lsp`.

## Package manager

```bash
purec pkg init myapp
purec pkg add path:../utils
purec pkg list
purec pkg build          # compiles entry from Pure.toml
```

`Pure.toml` example:

```toml
[package]
name = "myapp"
version = "0.1.0"
entry = "src/main.pure"

[dependencies]
utils = { path = "../utils" }
```


## Sysroot helper scripts

```bash
# Android NDK → .so
export ANDROID_NDK=/path/to/ndk
./scripts/android_build.sh examples/hello.pure libhello.so

# iOS (macOS + Xcode) → .o
./scripts/ios_build.sh examples/hello.pure hello.o
```
