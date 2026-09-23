# PureLang Roadmap

## Current Status (September 2026)

- [x] Project initialized & public repository
- [x] Core design documents
- [x] **Lexer implemented**
- [x] **Parser + AST implemented**
- [x] **Type checker + Ownership analysis**
- [x] **LLVM IR code generation**
- [x] **WebAssembly backend**
- [x] Standard library (math builtins via libm)

> **Note:** PureLang is a **public** repository. CI uses free public GitHub Actions minutes (does not consume private-repo Actions quotas).

## Phase 1 — Minimal Viable Compiler (Complete)

**Goal**: Tokenize → Parse → Generate simple native code

- [x] Lexer for the easy PureLang syntax
- [x] Parser → Abstract Syntax Tree (AST)
- [x] Basic type checking
- [x] Simple ownership tracking
- [x] LLVM IR emission for a tiny subset
- [x] First native executable (`purec --compile -o hello hello.pure`)
- [x] First WebAssembly module (WASI .wat via --emit-wasm)

**Milestone**: Hello World + simple game loop running natively.

## Phase 2 — Core Language Features

- [x] User-defined functions + calls + return values
- [x] Structs (declaration, construction, field access)
- [x] Lists (literal, length, indexing, for-in)
- [x] Methods on structs
- [x] Typed parameters (`fn f(x: Number)`) (`fn Point.distance(self)`)
- [x] Full ownership & borrow checker (moves + borrows)
- [x] Enums + match (`enum Color { Red Green }` + `match c { Color.Red => ... }`)

- [x] Generics (`fn id[T](x: T)`) + traits (`trait` / `impl`)
- [x] Error handling with `?`
- [x] Modules (`mod name { ... }` + `name.fn()`)
- [x] Visibility (`pub` enforced across modules)

## Phase 3 — Performance & Platforms

- [x] Host optimization levels (`--opt 0|1|2|3|s`)
- [x] WebAssembly + WASI target (basic)
- [x] Native CI: Linux, Windows, macOS
- [x] `--target` triple + platform-aware clang link
- Cross-compilation sysroots (iOS, Android) — planned
- Console targets (via platform SDKs + C ABI)
- SIMD and data-oriented design helpers
- Fast incremental compilation

## Phase 4 — Standard Library & Tooling

- [x] Math helpers (abs, min, max, pow, sqrt, floor, ceil, round, sin, cos, tan, log, exp)
- [x] File I/O (`read_file`, `write_file`, `file_exists`)
- [x] List helpers (`list_len`, `list_get`, `list_sum`)
- [x] Formatter (`purec --fmt`)
- [x] Language Server (`purec --lsp`)
- [x] Package manager foundation (`purec pkg`)
- [x] Platform presets (android/ios/console/…)
- Core collections (Vec, Map, etc.) — partial (List built-in)
- File I/O, networking, concurrency primitives
- Formatter (`pure fmt`)
- Language Server (LSP)
- Package manager foundation

## Phase 5 — Ecosystem & Production

- Self-hosting compiler
- Comprehensive test suite + fuzzer
- Debugger support
- Documentation generator
- Official website + tutorials
- Community packages

## Long-term Vision

PureLang should become a practical choice for:

- Game engines and games
- 3D tools and real-time applications
- AI / high-performance numerical code
- Cross-platform native applications
- Console development (where platform agreements allow)

## Success Metrics

- Newcomers can write useful programs within hours
- Performance competitive with Rust / C++ / Zig
- Memory safety guaranteed in safe code
- Real projects (small games, tools, AI kernels) shipping in PureLang
