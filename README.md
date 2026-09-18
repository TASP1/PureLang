# PureLang

**PureLang** is a next-generation, all-in-one programming language designed for **ultra-native speed**, **absolute memory safety**, and **minimalist modern syntax**.

## 🎯 The Goal

Create a single, unified language that runs at maximum hardware efficiency everywhere — Windows, macOS, Linux, iOS, Android, and web browsers — **without requiring a heavy runtime or a slow Garbage Collector**.

It combines:
- The **bulletproof compile-time memory safety** of Rust (ownership + lifetimes)
- The **clean, readable syntax** of TypeScript and Python
- True zero-cost abstractions and native performance via LLVM + WebAssembly

## 🛠️ How It Will Be Made

PureLang is engineered through a highly efficient, multi-platform translation pipeline:

### 1. The Core Compiler
- Written from scratch in **Rust**
- Enforces strict compile-time ownership, lifetime tracking, and zero-cost abstractions
- Full borrow checker and memory safety guarantees at compile time (no GC)

### 2. Desktop & Mobile Performance
- Translates PureLang code directly into **LLVM Intermediate Representation (IR)**
- Leverages global open-source LLVM compiler infrastructure
- Auto-optimizes binaries for all native computer and mobile chips (x86_64, AArch64, etc.)

### 3. Web Integration
- Compiles into **WebAssembly (Wasm)**
- Executes inside secure browser environments at near-native hardware speeds
- Completely bypasses slow JavaScript translation layers
- Supports WASI for system interfaces

## 📁 Project Structure

```
PureLang/
├── docs/                  # Design documents, architecture, roadmap
│   ├── ARCHITECTURE.md
│   ├── MEMORY_MODEL.md
│   ├── SYNTAX.md
│   ├── COMPILER.md
│   └── ROADMAP.md
├── src/                   # Future compiler source code (Rust)
├── examples/              # Sample PureLang programs
├── .github/workflows/     # GitHub Actions CI
├── README.md
├── LICENSE
└── CONTRIBUTING.md
```

## 🚀 Current Status

- [x] Project initialized
- [x] Core design documents written
- [ ] Minimal lexer + parser
- [ ] Ownership & type checker
- [ ] LLVM IR emission
- [ ] WebAssembly target
- [ ] Standard library

## 📖 Documentation

Start here:

1. [Architecture Overview](docs/ARCHITECTURE.md)
2. [Memory Safety Model](docs/MEMORY_MODEL.md)
3. [Syntax Design](docs/SYNTAX.md)
4. [Compiler Pipeline](docs/COMPILER.md)
5. [Roadmap](docs/ROADMAP.md)

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT License — see [LICENSE](LICENSE).

---

**PureLang** — Native speed. Absolute safety. Beautiful syntax. Everywhere.
