# PureLang Compiler Design

## Overview

The PureLang compiler is called **`purec`**.

It is written in Rust and currently at **Phase 1 (Lexer)**.

## Current Implementation

```
compiler/
├── Cargo.toml
├── src/
│   ├── main.rs      # CLI entry point
│   ├── token.rs     # Token definitions
│   └── lexer.rs     # Lexer implementation
```

### What works today

```bash
cd compiler
cargo build
cargo run -- ../examples/hello.pure
```

The lexer successfully tokenizes PureLang source files using the easy syntax.

## Planned Pipeline Stages

1. **Lexing** ← (completed)
2. **Parsing** → Abstract Syntax Tree (AST)
3. **Name Resolution**
4. **Type Checking + Ownership Analysis**
5. **IR Generation**
6. **Code Generation**
   - LLVM IR → native binaries
   - WebAssembly

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

Implement the **Parser** that turns the token stream into an Abstract Syntax Tree.
