# PureLang Roadmap

**Current release:** purec **v0.37.0** (October 2026)  
**Product readiness (honest):** ~**62%** overall · ~**85%** desktop compiler usable  

This document is the **single design plan** for what comes next.  
Implementation work should follow these phases in order unless a critical bug forces a detour.

---

## 1. North star

PureLang aims to be a **systems language** that is:

1. **Fast** — LLVM native (and usable WASM) without a GC in the common path  
2. **Safe** — ownership / borrows by default; unsafe only when explicit later  
3. **Simple** — syntax closer to Python than to C++ or Rust  
4. **Portable** — desktop first; mobile and consoles via C ABI + platform hosts  

**Not goals (near term):** competing with full SwiftUI/Jetpack, or auto-submitting to app stores.

---

## 2. Where we are (v0.37.0 baseline)

### Done (usable today)

| Layer | What works |
|-------|------------|
| **Compiler (`purec`, Rust)** | Lexer → parser → AST → type/ownership check → LLVM → clang native; WASM `.wat` |
| **Language** | Functions (incl. function values), structs/methods, lists, maps, enums/`match`, modules/`pub`, generics/traits (MVP), `?`, `while`/`break`/`continue`, Number + Float, strings |
| **Stdlib / runtime** | Math, file I/O, string helpers, maps, `time_ms`/`sleep_ms`/`exit`, OpenSSL/WinInet HTTPS, channels + `thread_spawn` |
| **Tooling** | `--fmt`, `--lsp`, `pkg`, multi-OS CI, golden tests, platform scripts |
| **Self-host seed** | `bootstrap/purec_sub.pure`: multi-letter ids, assign, `+`, `print`, **if**, **while**, file input → valid LLVM IR |
| **Mobile / store** | Gradle Android host, iOS package skeleton, `docs/STORE_SIGNING.md` (manual signing only) |

### Explicit gaps

| Gap | Why it matters |
|-----|----------------|
| Ownership still MVP | False confidence on large programs |
| Types mostly `i64`/`double` in codegen | Limits real APIs |
| `purec_sub` ≠ full purec | No modules/types/fns/real IR control-flow yet |
| UI / mobile SDKs thin | Host shells, not product UIs |
| Store upload manual | By design for security |

---

## 3. Principles for the next work

1. **Desktop compiler quality first** — fix soundness and types before flashy platforms.  
2. **Self-host grows as a second compiler**, not a rewrite of Rust purec until it can compile itself.  
3. **Every milestone must be demoable** (example + CI or script).  
4. **Docs update with every release** (STATUS %, ROADMAP checkboxes, README version).  
5. **No store auto-upload** in this repo; signing stays documented and local/secrets-based.

---

## 4. Phased roadmap

### Phase A — Language solidity (target: v0.40)

**Goal:** Trustworthy desktop purec for medium programs.

| # | Item | Done when |
|---|------|-----------|
| A1 | Stronger ownership (use-after-move on maps/structs consistent; clearer errors) | Spec in MEMORY_MODEL + tests |
| A2 | Clearer type rules (Number vs Float promotion documented; reject nonsense mixes) | SYNTAX + golden tests |
| A3 | String model (copy vs move policy locked; UTF-8 length documented) | MEMORY_MODEL + tests |
| A4 | Expand golden tests to ≥30 examples with expected stdout | **Done (v0.38)** `tests/run.sh` |
| A5 | Clippy + fmt + all platforms green on every push | CI matrix all success |

**Exit criteria:** New contributor can write a 200-line tool without fighting the checker daily.

---

### Phase B — Self-host compiler path (target: v0.45)

**Goal:** `purec_sub` becomes a **real subset compiler**, still not full purec.

| # | Item | Done when |
|---|------|-----------|
| B1 | Multi-char operators (`<=`, `>=`, `!=`, `==`) in purec_sub | Sample compiles |
| B2 | Nested blocks + `else` on `if` | control samples |
| B3 | Simple functions in subset (`fn add(a, b) { return a + b }`) with compile-time or IR call | IR runs |
| B4 | Emit **runtime** LLVM control flow (alloca + `br`) instead of only interpreting | while loop in IR has labels |
| B5 | `purec_sub` reads CLI path argument (via convention file or `read_file` arg) | `samples/*.pure` batch |
| B6 | Bootstrap README = “how to grow toward self-host” | docs only |

**Exit criteria:** purec_sub compiles a small suite of programs to clang-runnable IR without human IR editing.

**Non-goal this phase:** Replacing Rust purec.

---

### Phase C — Stdlib & packaging (target: v0.50)

**Goal:** Batteries that match a small systems language.

| # | Item | Done when |
|---|------|-----------|
| C1 | Lists: push/pop or grow API (runtime-backed) | example + tests |
| C2 | Better maps (or string→string map) | example |
| C3 | JSON or minimal config parse (optional) | example |
| C4 | `package_desktop.sh` produces versioned tarball in CI artifacts | CI artifact |
| C5 | Android: documented path from `.pure` → `libpureapp.so` → host Activity | STORE_SIGNING + script |
| C6 | iOS: documented path from `.pure` → `.o` → Xcode | same |

**Exit criteria:** One “sample app” per major platform documented end-to-end (desktop + one mobile).

---

### Phase D — Platforms & UI depth (target: v0.55–v0.60)

**Goal:** Deeper hosts, still not full vendor SDKs.

| # | Item | Done when |
|---|------|-----------|
| D1 | Win32: more widgets or message callbacks usable from PureLang | example |
| D2 | Cocoa: thin bridge doc + one native window path (or explicit “HTML-only on macOS”) | docs + example |
| D3 | Android: JNI glue template for `purelangMain` return codes | template in dist/ |
| D4 | WASM: browser demo page (optional) | static HTML in repo |
| D5 | Console/SDK notes only (no SDK redistrib) | PLATFORMS.md |

**Exit criteria:** Clear matrix: what UI works where, without pretending UIKit/Jetpack are complete.

---

### Phase E — Self-host stretch (target: v1.0 aspirational)

**Goal:** PureLang can compile a **defined subset** of itself; Rust purec remains production until parity.

| # | Item | Done when |
|---|------|-----------|
| E1 | purec_sub (or successor) parses modules of the subset | |
| E2 | Type tags in subset (optional annotations) | |
| E3 | Emit IR for functions + calls | |
| E4 | Dogfood: subset compiler compiles its own samples in CI | CI job |
| E5 | Decision gate: continue self-host vs keep Rust purec primary | written in STATUS |

**v1.0 product definition (proposed):**

- purec (Rust) stable API + semver  
- Spec for language subset + full language  
- CI green on Linux/Windows/macOS  
- Desktop packages + mobile **manual** store docs  
- Self-host **subset** in CI (not necessarily full bootstrap of purec)

---

## 5. Priority order (what to implement next)

Do **not** start D/E until A and B are mostly done.

```text
NOW → Phase A (solidity)
    → Phase B (self-host subset growth)
    → Phase C (stdlib + packaging polish)
    → Phase D (platform depth)
    → Phase E (self-host stretch / v1.0 gate)
```

**Immediate next implementation slice (after this roadmap lands):**

1. **A4** — expand golden tests  
2. **A1/A2** — ownership + type rule tightening  
3. **B1/B2** — purec_sub `else` + comparison operators  
4. **B4** — real IR branches for while (big step)

---

## 6. Release cadence

| Cadence | Action |
|---------|--------|
| Every feature PR | Bump purec patch/minor, update STATUS %, README version |
| Every milestone end | Tag `v0.x.0`, refresh ROADMAP checkboxes |
| CI | Must stay green on Ubuntu + Windows + macOS for purec |

Versioning: **0.x** until Phase E gate; then **1.0.0** only when v1.0 definition above is met.

---

## 7. Success metrics

| Metric | Target |
|--------|--------|
| Time to first native binary for a newcomer | < 30 minutes |
| Example suite | 100% typecheck + compile on CI |
| purec_sub sample suite | All samples → IR → run |
| Performance | Within ~1.5× of C for numeric loops (goal, measure later) |
| Safety | No silent use-after-move in tested patterns |

---

## 8. Out of scope (until after v1.0)

- Full SwiftUI / Jetpack / Compose in-tree  
- Automatic Play / App Store submission from this GitHub repo  
- Full concurrent GC or full Rust-level borrow checker  
- Guaranteed real-time / functional-safety certification  

---

## 9. Document ownership

| Doc | Role |
|-----|------|
| **ROADMAP.md** (this file) | Plan and phase order |
| **STATUS.md** | % complete and release notes |
| **SYNTAX.md** / **MEMORY_MODEL.md** | Language rules |
| **STORE_SIGNING.md** | Manual mobile release |
| **README.md** | Entry point + version |

Update **STATUS** and **README** on every release; update **ROADMAP** checkboxes when a phase item ships.

---

*Last designed: 2026-10-02 · purec v0.37.0*
