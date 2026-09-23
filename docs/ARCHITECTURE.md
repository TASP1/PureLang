# PureLang Architecture

## Vision

PureLang is a modern systems programming language designed for:

- Extreme performance (games, engines, 3D, AI)
- Absolute memory safety without garbage collection
- Syntax that is significantly easier than Python
- Native execution on every major platform (including consoles)

## High-Level Pipeline

```
PureLang Source (.pure)
        │
        ▼
   Lexer  ✅
        │
        ▼
   Parser → AST  ✅
        │
        ▼
   Type Checker + Ownership / Borrow Checker  ✅
        │
        ▼
   Code generation
        │
        ├──────────────────────┐
        ▼                      ▼
   LLVM IR ✅              WebAssembly ✅
        │                      │
        ▼                      ▼
 Native Binary              .wat / WASI
 (Linux today; more         (wasmtime, browsers)
  targets planned)
```

## Compiler Implementation

- Written entirely in **Rust** (`purec` **v0.11.0**)
- Hand-written recursive-descent parser
- LLVM IR emitted as text; linked with `clang`
- Math via **libm** (`-lm`)

### Why Rust for the compiler?

- Memory safety for the compiler itself
- Strong ecosystem and tooling
- Fast, reliable CI builds with caching

## Key Design Decisions

| Area            | Decision                         | Reason                |
|-----------------|----------------------------------|------------------------|
| Syntax          | Simple, braces, immutable default | Ease of use          |
| Memory safety   | Ownership + borrowing (compile-time) | Zero-cost safety  |
| Backend         | LLVM + WebAssembly               | Performance + reach   |
| Interop         | C ABI (via clang)                | SDKs, existing libs   |
| Stdlib math     | libm builtins                    | Portable numerics     |

## Platform Strategy

- **Desktop:** LLVM native (Linux working; Windows/macOS planned)
- **Web:** WebAssembly / WASI
- **Mobile / consoles:** via C ABI + platform SDKs (future)

## Extensibility

- Stronger optimization pipeline
- Possible Cranelift debug backend
- Incremental compilation for game-style iteration

This architecture reuses proven infrastructure (LLVM) while focusing innovation on language design and developer experience.
