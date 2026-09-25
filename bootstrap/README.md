# PureLang self-host bootstrap

Production compiler (`purec`) is still **Rust**. These programs implement a growing PureLang front/middle end:

| Stage | File | Status |
|-------|------|--------|
| Char scan | `token_scan.pure` | done |
| Lexer | `lexer.pure` | done |
| Parser + codegen (subset) | `parser_codegen.pure` | parses `print <int>`, reports IR template size |

```bash
cd compiler && cargo build --release
./target/release/purec --compile -o /tmp/pc ../bootstrap/parser_codegen.pure && /tmp/pc
```

Roadmap to full self-host: token list → recursive descent for expr/stmt → emit `.ll` via `write_file` → invoke `clang`.
