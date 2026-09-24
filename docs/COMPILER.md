# PureLang Compiler Design

## Overview

**`purec` v0.25.0** — Rust implementation (edition 2024).

```
Source (.pure)
  → Lexer (Spanned tokens, line numbers)
  → Parser → AST (StmtNode with lines)
  → Type checker + ownership / borrows
  → LLVM IR text → clang (+ purelang_rt.c, -lm) → native binary
  → WASI .wat ( --emit-wasm )
```

## Layout

```
compiler/
├── Cargo.toml          # purec 0.25.0
└── src/
    ├── main.rs         # CLI: compile, IR, WASM, fmt, lsp, pkg, --platform
    ├── token.rs        # Token + Spanned
    ├── lexer.rs
    ├── ast.rs          # Program, Item, StmtNode, Expr, …
    ├── parser.rs
    ├── types.rs
    ├── checker.rs
    ├── codegen.rs      # LLVM IR + stdlib/runtime calls
    ├── wasm.rs
    ├── fmt.rs
    ├── lsp.rs
    ├── pkg.rs
    └── platforms.rs
runtime/
└── purelang_rt.c       # maps, string helpers, HTML UI, time_ms
```

## Implemented stages

| Stage | Status |
|-------|--------|
| Lexing (with lines) | ✅ |
| Parsing → AST | ✅ |
| Type + ownership checking | ✅ |
| LLVM native | ✅ |
| WASM (WASI) | ✅ |
| Formatter | ✅ |
| LSP (stdio) | ✅ |
| Package manager (path deps) | ✅ |
| Golden tests | ✅ |

## CLI (selected)

```bash
purec <file.pure>                 # type-check
purec --compile -o out file.pure
purec --target <triple> --opt 3 …
purec --platform android|ios|linux|macos|windows|console
purec --emit-ir / --emit-wasm
purec --fmt file.pure
purec --lsp
purec pkg init|add|list|build
purec --list-platforms
purec --version
```

## Linking

Native builds invoke `clang` with optimization flags, host/target triple, `-lm` on Unix, and `runtime/purelang_rt.c` when present (maps, UI, `time_ms`, string helpers).

## Next

- Column-level diagnostics
- Stronger incremental compilation
- Self-hosting
