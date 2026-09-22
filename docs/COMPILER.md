# PureLang Compiler Design

## Overview

The PureLang compiler is called **`purec`**.

It is written in Rust and currently at **v0.7.0 — Phase 1 complete + Phase 2 in progress (functions, structs, lists)**.

## Current Implementation

```
compiler/
├── Cargo.toml
├── src/
│   ├── main.rs      # CLI entry point
│   ├── token.rs     # Token definitions
│   ├── lexer.rs     # Lexer implementation
│   ├── ast.rs       # Abstract Syntax Tree
│   ├── parser.rs    # Recursive-descent parser
│   ├── types.rs     # Type system
│   ├── checker.rs   # Type checker + ownership analysis
│   ├── codegen.rs   # LLVM IR text emitter → clang
│   └── wasm.rs      # WebAssembly Text (WASI) emitter
```

### What works today

```bash
cd compiler
cargo build
cargo run -- ../examples/hello.pure
```

The lexer tokenizes source, the parser builds a full AST, and the type checker enforces types + basic ownership (immutable-by-default, move of non-Copy values).

## Planned Pipeline Stages

1. **Lexing** ← (completed)
2. **Parsing** → Abstract Syntax Tree (AST) ← (completed)
3. **Type Checking + Ownership Analysis** ← (completed)
4. **LLVM IR Code Generation** ← (completed)
5. **WebAssembly backend** ← (completed)
6. **Richer stdlib / optimizations**

## Technology Choices

- **Language**: Rust
- **Parsing**: Hand-written recursive descent (starting simple) or parser combinators later
- **LLVM interop**: `inkwell` or `llvm-sys`
- **CLI**: Simple argument parsing (will use `clap` later)

## Diagnostics Philosophy

Errors should be:

- Precise (exact location)
- Helpful (suggest fixes when possible)
- Beautiful (colored, with source context)

## Build Goals

- Single static binary for easy distribution
- Fast incremental compilation (critical for game development)
- Excellent cross-compilation support

## Next Immediate Step

Next: methods, enums, full borrow checker, stdlib.
