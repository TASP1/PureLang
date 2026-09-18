# PureLang Compiler Source

This directory will contain the Rust implementation of the PureLang compiler (`purec`).

## Planned Structure (Phase 1+)

```
src/
├── main.rs              # CLI entry point
├── lexer/
├── parser/
├── ast/
├── typeck/
├── borrowck/
├── codegen/
│   ├── llvm.rs
│   └── wasm.rs
└── diagnostics/
```

The compiler is intentionally written in Rust so that it benefits from the same class of memory safety guarantees that PureLang will provide to its users.
