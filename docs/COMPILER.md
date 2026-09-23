# PureLang Compiler Design

## Overview

The PureLang compiler is **`purec`**, written in Rust.

**Current version: v0.18.0**

Pipeline:

```
Source (.pure)
  → Lexer → Parser → AST
  → Type checker + ownership / borrows
  → LLVM IR text  →  clang  →  native binary
  → WASI .wat     →  wasmtime / browsers
```

## Layout

```
compiler/
├── Cargo.toml          # purec 0.11.0, edition 2024
└── src/
    ├── main.rs         # CLI (--compile, --emit-ir, --emit-wasm, …)
    ├── token.rs
    ├── lexer.rs
    ├── ast.rs
    ├── parser.rs       # recursive descent + Pratt
    ├── types.rs
    ├── checker.rs      # types, ownership, modules, traits, generics
    ├── codegen.rs      # LLVM IR (opaque pointers) + libm builtins
    └── wasm.rs         # WebAssembly Text (WASI)
```

## Implemented

| Stage | Status |
|-------|--------|
| Lexing | ✅ |
| Parsing → AST | ✅ |
| Type checking | ✅ |
| Ownership / borrows (MVP+) | ✅ |
| LLVM native codegen | ✅ |
| WASM backend | ✅ |
| Functions, structs, methods | ✅ |
| Lists, for-in | ✅ |
| Enums + match | ✅ |
| Modules + `pub` | ✅ |
| Generics + traits | ✅ |
| `?` on Result/Option-style enums | ✅ |
| Math stdlib via libm | ✅ |

## CLI

```bash
purec <file.pure>              # type-check
purec --compile -o out file.pure
purec --target <triple> --opt 3 -o out file.pure
purec --emit-ir file.pure
purec --emit-wasm file.pure
purec --ast / --tokens file.pure
```

## Technology

- **Language:** Rust (edition 2024)
- **Backend:** hand-written LLVM IR text + system `clang` (`-lm` for math)
- **WASM:** WASI snapshot preview1 text format
- **CI:** public repo, free Actions minutes, `Swatinem/rust-cache`

## Diagnostics goals

Precise locations, helpful messages, colored context (to be expanded).

## Next

- File I/O & richer collections in stdlib
- Formatter / LSP
- Stronger LLVM optimization pipeline
- Cross-compilation targets
