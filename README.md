# PureLang

**PureLang** is a modern systems programming language designed for:

- **Ultra-native performance** (games, engines, 3D, AI)
- **Absolute memory safety** without a garbage collector
- **Syntax far easier than Python**
- Native execution on desktop, mobile, web, and consoles

## Quick Example

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

## Current Status

- [x] Public repository & design documents
- [x] **Lexer complete** (Phase 1)
- [ ] Parser + AST
- [ ] Type & Ownership checker
- [ ] LLVM code generation
- [ ] WebAssembly target

## Project Structure

```
PureLang/
├── compiler/           # purec compiler (Rust)
│   └── src/
│       ├── main.rs
│       ├── lexer.rs
│       └── token.rs
├── docs/               # Full design documentation
│   ├── ARCHITECTURE.md
│   ├── SYNTAX.md
│   ├── MEMORY_MODEL.md
│   ├── COMPILER.md
│   └── ROADMAP.md
├── examples/           # .pure example programs
├── .github/workflows/  # CI
├── README.md
├── LICENSE
└── CONTRIBUTING.md
```

## Try the Lexer

```bash
cd compiler
cargo build
cargo run -- ../examples/hello.pure
cargo run -- ../examples/game_loop.pure
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Syntax (Easy Mode)](docs/SYNTAX.md)
- [Memory Model](docs/MEMORY_MODEL.md)
- [Compiler Design](docs/COMPILER.md)
- [Roadmap](docs/ROADMAP.md)

## License

MIT License

---

**PureLang** — Native speed. Absolute safety. Easier than Python.
