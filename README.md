# PureLang

**PureLang** is a modern systems programming language designed for:

- **Ultra-native performance** (games, engines, 3D, AI)
- **Absolute memory safety** without a garbage collector
- **Syntax far easier than Python**
- Native execution on desktop, mobile, web, and consoles

**Repository:** [TASP1/PureLang](https://github.com/TASP1/PureLang) (public · MIT)  
**Compiler:** `purec` **v0.30.0**

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
    print list_sum(nums)
    print sqrt(49)
    print time_ms()
}
```

## Current Status (September 2026)

### Language & compiler
- [x] Lexer / parser / AST (line-accurate tokens & parse errors)
- [x] Type checker + ownership / borrows (errors with line numbers)
- [x] LLVM native codegen (`clang`) + WebAssembly (WASI `.wat`)
- [x] Functions + **function values** (`f = double; f(21)`), structs, methods, enums + `match`
- [x] Modules, `pub`, traits, generics, `?`
- [x] Control flow: `if` / `for` / **`while` / `break` / `continue`**
- [x] Types: Number, **Float**, String, Bool, List, **Map**, structs, enums
- [x] **assert**, **exit**, **time_ms**, **sleep_ms**, **http_get**
- [x] **Channels / threads** (`channel_new/send/recv`, `thread_spawn_send`, **`thread_spawn(fn)`**)
- [x] **Mobile sysroot scripts** (`scripts/android_build.sh`, `ios_build.sh`)
- [x] **Self-host bootstrap** (`bootstrap/`)

### Standard library
- [x] Math (`abs`, `min`, `max`, `pow`, `sqrt`, trig, … + `std.*`)
- [x] File I/O (`read_file`, `write_file`, `file_exists`)
- [x] Lists (`list_len` / `list_get` / `list_sum` / `list_max` / `list_min`)
- [x] Strings (`str_len`, `str_char_at`, `str_slice`, `str_is_empty`, `str_contains`, `str_eq`, `str_concat`, `str_from_num`)
- [x] Maps (`map_new` / `map_set` / `map_get` / `map_has` / `map_len`)
- [x] HTML UI foundation (`ui_begin` … → `purelang_ui.html`)

### Tooling & platforms
- [x] Formatter (`purec --fmt`)
- [x] LSP (`purec --lsp` — diagnostics, hover, completions)
- [x] Package manager (`purec pkg init|add|list|build`)
- [x] Platform presets (`--platform android|ios|linux|macos|windows|console`)
- [x] CI: **Linux / Windows / macOS** (public free Actions + Rust cache)
- [x] Golden output tests (`tests/run.sh`)

### Next
- [x] `thread_spawn(fn)` — spawn a zero-arg function value on a thread
- [ ] Native UI backends (beyond HTML export)
- [ ] iOS / Android sysroot automation in CI
- [ ] Self-hosting compiler (beyond bootstrap seeds)
- [x] Windows: real channels/threads (`CreateThread` + condition variables)

## Try it

Requires: **Rust** (stable), **clang**, optional **wasmtime**.

```bash
git clone https://github.com/TASP1/PureLang.git
cd PureLang/compiler
cargo build --release

cargo run --release -- ../examples/hello.pure
cargo run --release -- --compile -o hello ../examples/hello.pure && ./hello

# Function values, channels, floats
cargo run --release -- --compile -o /tmp/fn ../examples/fn_values.pure && /tmp/fn
cargo run --release -- --compile -o /tmp/ch ../examples/channels.pure && /tmp/ch

# Formatter / LSP / packages
cargo run --release -- --fmt ../examples/hello.pure
cargo run --release -- --lsp
cargo run --release -- pkg init myapp

# Tests
../tests/run.sh
```

## Project structure

```
PureLang/
├── compiler/           # purec v0.30.0 (Rust)
│   └── src/            # lexer → parser → checker → codegen / wasm / lsp / pkg
├── runtime/            # purelang_rt.c (maps, UI, time_ms, strings)
├── docs/               # architecture, syntax, platforms, UI, package, …
├── examples/           # 25+ .pure programs
├── tests/              # golden expected outputs + run.sh
├── editors/vscode/     # generic LSP client notes
└── .github/workflows/  # multi-OS CI
```

## Documentation

| Doc | Topic |
|-----|--------|
| [Architecture](docs/ARCHITECTURE.md) | Pipeline & design |
| [Syntax](docs/SYNTAX.md) | Language surface |
| [Memory model](docs/MEMORY_MODEL.md) | Ownership |
| [Compiler](docs/COMPILER.md) | purec internals |
| [Platforms](docs/PLATFORMS.md) | Desktop / mobile / console |
| [Package manager](docs/PACKAGE.md) | `purec pkg` |
| [UI](docs/UI.md) | HTML UI export |
| [Roadmap](docs/ROADMAP.md) | Phases |
| [Status](docs/STATUS.md) | Honest completion matrix |

## License

MIT License

---

**PureLang** — Native speed. Absolute safety. Easier than Python.
