# PureLang self-host bootstrap

| Stage | File | Output |
|-------|------|--------|
| Char scan | `token_scan.pure` | counts |
| Lexer | `lexer.pure` | token class counts |
| Parser subset | `parser_codegen.pure` | parse `print N` |
| **Mini compiler** | `purec_mini.pure` | writes `bootstrap_out.ll` |

```bash
cd compiler && cargo build --release
./target/release/purec --compile -o /tmp/mini ../bootstrap/purec_mini.pure && /tmp/mini
# optional: clang bootstrap_out.ll -o /tmp/out && /tmp/out
```

Production `purec` remains Rust until the PureLang pipeline covers modules, types, and full LLVM emission.
