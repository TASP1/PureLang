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
   Lexer  (implemented)
        │
        ▼
   Parser → AST
        │
        ▼
   Type Checker + Ownership / Borrow Checker
        │
        ▼
   Intermediate Representation
        │
        ├──────────────────────┐
        ▼                      ▼
   LLVM IR                 WebAssembly
        │                      │
        ▼                      ▼
 Native Binary              .wasm / WASI
 (Desktop, Mobile,         (Browsers + Edge)
  Consoles)
```

## Compiler Implementation

- Written entirely in **Rust**
- Current stage: **Lexer complete**
- Future stages: Parser → Type/Ownership checking → Code generation

### Why Rust for the compiler?

- The compiler itself benefits from memory safety
- Excellent ecosystem for parsing and LLVM interop (`inkwell`)
- Fast and reliable

## Key Design Decisions

| Area                | Decision                                      | Reason |
|---------------------|-----------------------------------------------|--------|
| Syntax              | Extremely simple, braces, immutable by default | Ease of use |
| Memory Safety       | Ownership + borrowing (compile-time)          | Zero-cost safety |
| Backend             | LLVM + WebAssembly                            | Performance + reach |
| Interop             | Excellent C ABI                               | Console SDKs, existing libraries |
| Metaprogramming     | Keep simple at first, expand later            | Avoid complexity |

## Platform Strategy

- **Desktop / Mobile**: Full LLVM native targets
- **Web**: WebAssembly
- **Consoles**: Native code + platform SDKs via C interop (requires developer agreements)

## Future Extensibility

- Possible MLIR dialect later for advanced GPU / accelerator support
- Cranelift as a fast debug backend
- Incremental compilation for rapid game iteration

This architecture reuses proven infrastructure (LLVM) while focusing innovation on the language design and developer experience.
