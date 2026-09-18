# PureLang Compiler Design

## Overview

The PureLang compiler (`purec`) is written entirely in **Rust**. This choice provides:

- Memory safety while developing the compiler itself
- Excellent ecosystem for parsing, diagnostics, and LLVM interop
- Ability to dog-food the same ownership model the language provides

## Pipeline Stages

1. **Lexing**  
   Source text → Token stream

2. **Parsing**  
   Token stream → Abstract Syntax Tree (AST)

3. **Name Resolution**  
   Resolve identifiers to definitions, build symbol tables

4. **Type Checking & Inference**  
   Assign and verify types, perform type inference

5. **Ownership & Borrow Checking**  
   Prove memory safety rules

6. **HIR / MIR Lowering** (optional intermediate forms)  
   High-level IR → Mid-level IR for optimizations and analysis

7. **Code Generation**
   - Native path → LLVM IR → object code / executable
   - Web path → WebAssembly module

## Technology Choices

| Stage              | Recommended Crates / Tools          | Notes |
|--------------------|-------------------------------------|-------|
| Parsing            | `pest`, `nom`, `lalrpop`, or hand-written recursive descent | Start simple |
| Diagnostics        | `codespan`, `ariadne`, or `miette`  | Beautiful error messages |
| LLVM Interop       | `inkwell` or `llvm-sys`             | Preferred: inkwell for safety |
| CLI                | `clap`                              |       |
| Testing            | `cargo test` + snapshot testing     |       |

## LLVM Integration

- Use LLVM’s C API via safe Rust bindings (`inkwell`).
- Emit LLVM IR for the host architecture.
- Reuse LLVM’s powerful optimization passes (`-O0` to `-O3`, LTO, etc.).
- For WebAssembly: target `wasm32-unknown-unknown` or `wasm32-wasi` via LLVM.

## Bootstrapping Plan

1. **Stage 0**: Compiler written in Rust, can compile a tiny subset of PureLang.
2. **Stage 1**: PureLang compiler can compile itself (self-hosting).
3. **Stage 2**: Full language features + standard library written in PureLang.

## Diagnostics Philosophy

Errors should be:
- Precise (point to the exact location)
- Helpful (suggest fixes when possible)
- Beautiful (colored, structured, with source context)

Example style (inspired by Rust and Elm):

```
error[E0308]: mismatched types
  --> src/main.pl:12:18
   |
12 |     let x: i32 = "hello"
   |                  ^^^^^^^ expected `i32`, found `&str`
```

## Build & Distribution

- Single static binary for `purec` (easy distribution)
- Cross-compilation support via LLVM
- Future: package manager (`pure`) integrated with the compiler

This design keeps the compiler lean, safe, and focused on producing excellent machine code while providing a delightful developer experience.
