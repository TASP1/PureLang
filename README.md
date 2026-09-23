# PureLang

**PureLang** is a modern systems programming language designed for:

- **Ultra-native performance** (games, engines, 3D, AI)
- **Absolute memory safety** without a garbage collector
- **Syntax far easier than Python**
- Native execution on desktop, mobile, web, and consoles

**Repository:** [TASP1/PureLang](https://github.com/TASP1/PureLang) (public · MIT)  
**Compiler:** `purec` **v0.15.0**

## Quick Example

```pure
fn add(a: Number, b: Number) {
    return a + b
}

struct Point {
    x
    y
}

fn Point.sum(self) {
    return self.x + self.y
}

fn main() {
    print add(40, 2)
    p = Point(3, 4)
    print p.sum()
    nums = [10, 20, 30]
    print nums.length
    print sqrt(49)
}
```

## Current Status (September 2026)

### Done
- [x] Public repository & design documents
- [x] **Lexer / Parser / AST**
- [x] **Type checker + ownership / borrows**
- [x] **LLVM codegen** → native binaries (`clang`)
- [x] **WebAssembly** (WASI `.wat`)
- [x] **Functions** (define, call, return, typed params)
- [x] **Structs** + **methods** (`fn Point.sum(self)`)
- [x] **Lists** (literal, `.length`, index, `for x in list`)
- [x] **Enums + match**
- [x] **Modules** + **visibility** (`pub`)
- [x] **Traits** + **generics**
- [x] **Error handling** (`?`)
- [x] **Stdlib math** (`abs`, `min`, `max`, `pow`, `sqrt`, … + `std.*`)
- [x] CI on **public** GitHub Actions (free minutes + Rust cache)
- [x] **Multi-platform**: Linux, Windows, macOS CI + `--target` / `--opt`

### Next
- [ ] File I/O & richer collections
- [ ] Formatter (`pure fmt`) / LSP
- [ ] Cross-compilation & optimizations
- [ ] Package manager foundation

## Try it

Requires: **Rust** (stable), **clang** (native), optional **wasmtime** (WASM).

```bash
git clone https://github.com/TASP1/PureLang.git
cd PureLang/compiler
cargo build --release

cargo run --release -- ../examples/hello.pure
cargo run --release -- --compile -o hello ../examples/hello.pure && ./hello

# More examples
for f in funcs structs lists list_loop methods enums ownership \
         typed_params try_op modules generics traits visibility stdlib; do
  cargo run --release -- --compile -o /tmp/$f ../examples/$f.pure && /tmp/$f
done

cargo run --release -- --emit-wasm ../examples/hello.pure
wasmtime hello.wat
```

## Project Structure

```
PureLang/
├── compiler/                 # purec v0.15.0 (Rust)
│   └── src/
│       ├── main.rs           # CLI
│       ├── lexer.rs / token.rs
│       ├── parser.rs / ast.rs
│       ├── types.rs / checker.rs
│       ├── codegen.rs        # LLVM IR → native
│       └── wasm.rs           # WASI .wat
├── docs/
│   ├── ARCHITECTURE.md
│   ├── SYNTAX.md
│   ├── MEMORY_MODEL.md
│   ├── COMPILER.md
│   └── ROADMAP.md
├── examples/                 # 17 .pure programs
└── .github/workflows/ci.yml  # public free Actions + rust-cache
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Syntax](docs/SYNTAX.md)
- [Memory Model](docs/MEMORY_MODEL.md)
- [Compiler Design](docs/COMPILER.md)
- [Roadmap](docs/ROADMAP.md)
- [Platforms](docs/PLATFORMS.md)

## License

MIT License

---

**PureLang** — Native speed. Absolute safety. Easier than Python.
