# PureLang self-host bootstrap

| Stage | File | Result |
|-------|------|--------|
| Lexer | `lexer.pure` | token counts |
| Parser | `parser_codegen.pure` | parse `print N` |
| **Mini compiler** | `purec_mini.pure` | writes **valid** `bootstrap_out.ll` |

```bash
cd compiler && cargo build --release
./target/release/purec --compile -o /tmp/mini ../bootstrap/purec_mini.pure
/tmp/mini
clang bootstrap_out.ll -o /tmp/out && /tmp/out   # prints 42
```

Production `purec` is still Rust. The mini compiler proves parse → IR → native is reachable in PureLang.
