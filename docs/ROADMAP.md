# PureLang Roadmap

## Current Status (September 2026)

- [x] Project initialized & public repository
- [x] Core design documents
- [x] **Lexer implemented**
- [x] **Parser + AST implemented**
- [x] **Type checker + Ownership analysis**
- [x] **LLVM IR code generation**
- [x] **WebAssembly backend**
- [ ] Standard library

> **Note:** PureLang is a **public** repository. CI uses free public GitHub Actions minutes (does not consume private-repo Actions quotas).

## Phase 1 — Minimal Viable Compiler (In Progress)

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
- [x] Lists (literal, length, indexing)
- [ ] Full ownership & borrow checker
- [ ] Methods, enums
- [ ] Pattern matching
- [ ] Generics / traits
- [ ] Error handling with `?`
- [ ] Modules and visibility

## Phase 3 — Performance & Platforms

- Full LLVM optimization pipeline
- WebAssembly + WASI target
- Cross-compilation (Windows, macOS, Linux, iOS, Android)
- Console targets (via platform SDKs + C ABI)
- SIMD and data-oriented design helpers
- Fast incremental compilation

## Phase 4 — Standard Library & Tooling

- Core collections (Vec, String, Map, etc.)
- File I/O, networking, concurrency primitives
- Math / vector / matrix helpers (games & 3D)
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
