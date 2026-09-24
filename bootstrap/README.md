# PureLang self-hosting bootstrap

Full self-hosting (purec written in PureLang) is a long-term goal.

This directory holds **bootstrap** programs that exercise language features needed for a future purec-in-PureLang:

| File | Role |
|------|------|
| `token_scan.pure` | Scan a hard-coded source string and classify characters |

Build:

```bash
cd compiler && cargo build --release
./target/release/purec --compile -o token_scan ../bootstrap/token_scan.pure
./token_scan
```
