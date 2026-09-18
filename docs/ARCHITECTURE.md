# PureLang Architecture

## High-Level Vision

PureLang is designed as a **systems programming language** with the developer experience of modern high-level languages.

### Core Principles

1. **Memory Safety by Default** — Compile-time ownership and borrowing (Rust-inspired). No garbage collector.
2. **Zero-Cost Abstractions** — High-level features compile down to the same machine code a skilled C programmer would write.
3. **Unified Multi-Target** — One language, one toolchain → native binaries + WebAssembly.
4. **Minimalist Modern Syntax** — Readable like TypeScript/Python, powerful like Rust.
5. **No Heavy Runtime** — No VM, no GC pauses, no large standard runtime.

## Compilation Pipeline

```
PureLang Source (.pl)
        │
        ▼
   Lexer + Parser  (written in Rust)
        │
        ▼
   Abstract Syntax Tree (AST)
        │
        ▼
   Type Checker + Ownership/Borrow Checker
        │
        ▼
   Intermediate Representation (PureLang IR)
        │
        ├──────────────────────┐
        ▼                      ▼
   LLVM IR                 WebAssembly
        │                      │
        ▼                      ▼
 Native Binary              .wasm module
 (Windows/macOS/Linux/     (Browsers + WASI)
  iOS/Android)
```

### Frontend (written in Rust)

- Hand-written or generated lexer/parser
- AST construction
- Name resolution
- Type inference + checking
- Ownership and lifetime analysis (borrow checker)
- Error reporting with beautiful diagnostics

### Backend

- **Native path**: Lower to LLVM IR → use LLVM’s optimizers and code generators for all major architectures.
- **Web path**: Lower to WebAssembly (via LLVM’s Wasm backend or custom lowering) with WASI support.

## Key Components

| Component              | Responsibility                                      | Implementation Language |
|------------------------|-----------------------------------------------------|-------------------------|
| Lexer / Parser         | Tokenize & parse source                             | Rust                    |
| AST                    | In-memory representation of the program             | Rust                    |
| Type System            | Static typing + inference                           | Rust                    |
| Ownership Checker      | Compile-time memory safety                          | Rust                    |
| Codegen (LLVM)         | Emit optimized LLVM IR                              | Rust + inkwell/llvm-sys |
| Codegen (Wasm)         | Emit WebAssembly                                    | Rust + LLVM Wasm        |
| Standard Library       | Safe collections, I/O, concurrency primitives       | PureLang + Rust FFI     |
| Package Manager        | Dependency resolution & publishing                  | Future                  |
| LSP / Tooling          | Editor support, formatter, debugger                 | Future                  |

## Why This Design?

- **Rust for the compiler**: The compiler itself benefits from the same memory safety guarantees it provides to users.
- **LLVM as the workhorse**: Decades of optimization work, multi-target support, and excellent Wasm backend are reused instead of reinvented.
- **Single source of truth**: One frontend produces both native and web targets, ensuring consistent semantics.

## Future Extensibility

- MLIR dialect (inspired by Mojo) for higher-level optimizations and GPU/accelerator support.
- Cranelift as an alternative fast backend for debug builds.
- Incremental compilation and parallel front-end.

This architecture deliberately reuses proven infrastructure (LLVM + Rust) while focusing innovation on the language design and ownership model.
