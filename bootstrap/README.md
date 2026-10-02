# PureLang self-host bootstrap

| Stage | File | Capability |
|-------|------|------------|
| Lexer | `lexer.pure` | token counts |
| Mini | `purec_mini.pure` | `print N` → IR |
| **Subset compiler** | `purec_sub.pure` | assigns, `print x`, `print x + y`, file input |

```bash
cd compiler && cargo build --release
./target/release/purec --compile -o /tmp/sub ../bootstrap/purec_sub.pure
/tmp/sub
clang bootstrap_out.ll -o /tmp/out && /tmp/out
# 42
# 7
```

Sample source: `bootstrap/samples/add.pure`
