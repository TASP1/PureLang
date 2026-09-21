# PureLang Compiler (`purec`)

This is the official compiler for the PureLang programming language.

## Current Stage: Lexer (Phase 1)

The compiler can currently **tokenize** PureLang source files (`.pure`).

### Build

```bash
cargo build
```

### Run

```bash
cargo run -- ../examples/hello.pure
# or
./target/debug/purec ../examples/hello.pure
```

### Next Steps

1. Parser → AST
2. Type checking + Ownership analysis
3. LLVM IR generation
4. WebAssembly backend
