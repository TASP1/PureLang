# PureLang self-host bootstrap

The production compiler (`purec`) is still implemented in **Rust**.

These programs grow a PureLang-written front end:

| File | Role |
|------|------|
| `token_scan.pure` | Character class counts |
| `lexer.pure` | Ident / number / punct tokenizer over a sample source |

Build:

```bash
cd compiler && cargo build --release
./target/release/purec --compile -o /tmp/lexer ../bootstrap/lexer.pure && /tmp/lexer
```

Next steps toward self-hosting: full token stream printer, recursive-descent parser for expr/stmt, then emit a subset of LLVM IR from PureLang.
