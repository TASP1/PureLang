# PureLang self-host bootstrap

| Stage | File | Capability |
|-------|------|------------|
| Lexer | `lexer.pure` | token counts |
| Mini | `purec_mini.pure` | `print N` → IR |
| **Subset compiler** | `purec_sub.pure` | `x = N`, `print x`, `print N` → IR |

```bash
cd compiler && cargo build --release
./target/release/purec --compile -o /tmp/sub ../bootstrap/purec_sub.pure
/tmp/sub
clang bootstrap_out.ll -o /tmp/out && /tmp/out
# 40
# 2
# 99
```

Still not a full purec (no modules/types/ownership/full AST in PureLang yet).
