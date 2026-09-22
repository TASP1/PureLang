# PureLang

**PureLang** is a modern systems programming language designed for:

- **Ultra-native performance** (games, engines, 3D, AI)
- **Absolute memory safety** without a garbage collector
- **Syntax far easier than Python**
- Native execution on desktop, mobile, web, and consoles

**Repository:** [TASP1/PureLang](https://github.com/TASP1/PureLang) (public · MIT)  
**Compiler:** `purec` **v0.8.2**

## Quick Example

```pure
fn add(a, b) {
    return a + b
}

struct Point {
    x
    y
}

fn main() {
    print add(40, 2)

    p = Point(3, 4)
    print p.x + p.y

    nums = [10, 20, 30]
    print nums.length
    print nums[0]
}
```

## Current Status (September 2026)

### Done
- [x] Public repository & design documents
- [x] **Lexer**
- [x] **Parser + AST**
- [x] **Type & ownership checker** (MVP)
- [x] **LLVM codegen** → native binaries (`clang`)
- [x] **WebAssembly** (WASI `.wat`)
- [x] **Functions** (define, call, return)
- [x] **Structs** (declare, construct, fields)
- [x] **Lists** (literal, `.length`, indexing, `for x in list`)
- [x] **Methods** (`fn Point.sum(self)` + `p.sum()`)
- [x] **Enums + match**
- [x] CI on **public** GitHub Actions (free unlimited minutes)

### Next
- [ ] Full borrow checker
- [ ] Generics / modules
- [ ] Standard library
- [ ] Formatter / LSP

## Try it

Requires: **Rust** (stable), **clang** (native), optional **wasmtime** (WASM).

```bash
git clone https://github.com/TASP1/PureLang.git
cd PureLang/compiler
cargo build --release

# Type-check
cargo run --release -- ../examples/hello.pure

# Native binary
cargo run --release -- --compile -o hello ../examples/hello.pure
./hello

# Functions / structs / lists
cargo run --release -- --compile -o funcs ../examples/funcs.pure && ./funcs
cargo run --release -- --compile -o structs ../examples/structs.pure && ./structs
cargo run --release -- --compile -o lists ../examples/lists.pure && ./lists
cargo run --release -- --compile -o list_loop ../examples/list_loop.pure && ./list_loop

# WebAssembly
cargo run --release -- --emit-wasm ../examples/hello.pure
wasmtime hello.wat
```

## Project Structure

```
PureLang/
├── compiler/           # purec (Rust)
│   └── src/
│       ├── main.rs     # CLI
│       ├── lexer.rs / token.rs
│       ├── parser.rs / ast.rs
│       ├── types.rs / checker.rs
│       ├── codegen.rs  # LLVM IR → native
│       └── wasm.rs     # WASI .wat
├── docs/
│   ├── ARCHITECTURE.md
│   ├── SYNTAX.md
│   ├── MEMORY_MODEL.md
│   ├── COMPILER.md
│   └── ROADMAP.md
├── examples/
│   ├── hello.pure
│   ├── game_loop.pure
│   ├── funcs.pure
│   ├── structs.pure
│   ├── lists.pure
│   └── list_loop.pure
└── .github/workflows/ci.yml
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Syntax](docs/SYNTAX.md)
- [Memory Model](docs/MEMORY_MODEL.md)
- [Compiler Design](docs/COMPILER.md)
- [Roadmap](docs/ROADMAP.md)

## License

MIT License

---

**PureLang** — Native speed. Absolute safety. Easier than Python.
