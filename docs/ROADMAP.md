# PureLang Roadmap

## Phase 0 — Foundations (Current)

- [x] Repository & documentation structure
- [x] Architecture, Memory Model, Syntax, and Compiler design docs
- [ ] Basic project scaffolding (Cargo workspace for the compiler)

## Phase 1 — Minimal Viable Compiler (MVP)

**Goal**: Compile a tiny subset of PureLang to native binary and WebAssembly.

- [ ] Lexer for basic tokens
- [ ] Parser → AST for expressions, functions, variables
- [ ] Simple type checker (i32, f64, bool, strings)
- [ ] Basic ownership tracking (move semantics)
- [ ] LLVM IR emission for the subset
- [ ] Wasm emission for the same subset
- [ ] `purec hello.pl -o hello` produces a working executable
- [ ] `purec hello.pl --target wasm` produces a working `.wasm`

**Milestone Demo**: Fibonacci + Hello World running natively and in the browser.

## Phase 2 — Core Language

- [ ] Full ownership & borrow checker
- [ ] Structs, enums, pattern matching
- [ ] Generics / type parameters
- [ ] Traits / interfaces
- [ ] Error handling (`Result`, `Option`)
- [ ] Control flow (if, match, loops)
- [ ] Modules and basic visibility

## Phase 3 — Standard Library & Tooling

- [ ] Core collections (`Vec`, `String`, `HashMap`, etc.) with full ownership safety
- [ ] I/O, file system, networking primitives
- [ ] Concurrency primitives (safe threads, channels, async foundation)
- [ ] Formatter (`pure fmt`)
- [ ] Language Server Protocol (LSP) prototype
- [ ] Basic package manager design

## Phase 4 — Production Readiness

- [ ] Self-hosting compiler (PureLang compiler written in PureLang)
- [ ] Comprehensive test suite + fuzzer
- [ ] Cross-compilation for all major platforms
- [ ] Debugger support (DWARF)
- [ ] Documentation generator
- [ ] Performance benchmarks vs Rust / C / Zig / Mojo

## Phase 5 — Ecosystem

- [ ] Package registry
- [ ] IDE plugins (VS Code, Neovim, etc.)
- [ ] Official website & tutorial series
- [ ] Community governance

## Success Metrics

- A non-trivial program (e.g. simple web server or CLI tool) can be written in PureLang and run with performance comparable to Rust.
- Memory safety bugs are impossible in safe code.
- Newcomers from TypeScript/Python can become productive quickly.
- The compiler itself is fast enough for interactive development.

This roadmap is living and will be updated as the project evolves.
