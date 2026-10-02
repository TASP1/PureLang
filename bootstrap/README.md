# PureLang self-host bootstrap

| Stage | File | Capability |
|-------|------|------------|
| Lexer | `lexer.pure` | token counts |
| Mini | `purec_mini.pure` | `print N` → IR |
| **Subset compiler** | `purec_sub.pure` | multi-letter ids, assign, `+`, **if**, **while**, file input |

```bash
cd compiler && cargo build --release
./target/release/purec --compile -o /tmp/sub ../bootstrap/purec_sub.pure
/tmp/sub
clang bootstrap_out.ll -o /tmp/out && /tmp/out
# 10
# 1
# 42
```

Sample: `bootstrap/samples/control.pure`
