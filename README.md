# PureLang

**PureLang** is a next-generation systems programming language designed for **ultra-native speed**, **absolute memory safety**, and a syntax that is **far easier than Python**.

It is built for:
- Games and game engines
- 3D software
- AI / high-performance computing
- Desktop, mobile, web, and **native console** applications

## 🎯 Goals

- Native performance on all platforms (Windows, macOS, Linux, iOS, Android, consoles)
- Compile-time memory safety (ownership model, no garbage collector)
- Syntax that is significantly easier and cleaner than Python
- Compiles to LLVM IR (native) + WebAssembly (web)

## ✨ Easy Syntax Example

```pure
fn main() {
    mut health = 100
    name = "Player"

    print "Welcome, " + name

    for i in 1..5 {
        health = health - 10
        print "Health: " + health

        if health <= 0 {
            print "Game Over"
            return
        }
    }

    print "You survived!"
}
```

## 📁 Project Structure

```
PureLang/
├── compiler/              # The PureLang compiler (purec) written in Rust
│   └── src/
│       ├── main.rs
│       ├── lexer.rs
│       └── token.rs
├── docs/                  # Design documents
├── examples/              # Sample .pure programs
├── .github/workflows/     # CI
├── README.md
├── LICENSE
└── CONTRIBUTING.md
```

## 🚀 Current Status

- [x] Project initialized
- [x] Core design documents
- [x] **Lexer implemented** (Phase 1)
- [ ] Parser + AST
- [ ] Type checker + Ownership
- [ ] LLVM IR code generation
- [ ] WebAssembly target
- [ ] Standard library

### Try the Lexer right now

```bash
cd compiler
cargo build
./target/debug/purec ../examples/hello.pure
```

## 📖 Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Memory Model](docs/MEMORY_MODEL.md)
- [Syntax Design](docs/SYNTAX.md)
- [Compiler Design](docs/COMPILER.md)
- [Roadmap](docs/ROADMAP.md)

## License

MIT License
