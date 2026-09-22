# PureLang Memory Model

## Goal

Deliver complete memory safety (no use-after-free, no double-free, no data races) with **zero runtime overhead** and **almost invisible syntax**.

## Core Rules

1. Every value has exactly one owner at a time.
2. When the owner goes out of scope, the value is automatically destroyed.
3. Values can be borrowed temporarily (immutably or mutably) under strict rules.
4. The compiler enforces all rules at compile time.

## Design Choice: Ownership is Mostly Invisible

Unlike classic Rust, PureLang aims to make ownership feel automatic for everyday code.

```pure
fn process(data) {          // ownership handled by compiler
    print data.length
}

list = [1, 2, 3]
process(list)               // safe transfer — list is no longer usable
```

Most of the time you do not write special ownership annotations. The compiler tracks everything.

## Mutability

- Variables are **immutable by default**
- Use `mut` only when you need to change a value

```pure
name = "Pure"               // cannot change
mut health = 100            // can change
health = health - 10
```

## Borrowing (Advanced)

When needed, the language supports:

- Immutable borrows (multiple allowed)
- Mutable borrows (exclusive)

These will surface more clearly as the type system matures. For early code, the simple ownership model is sufficient.

## Benefits

- No garbage collector pauses
- Deterministic destruction (important for games and real-time systems)
- Safety without sacrificing performance
- Syntax stays clean and approachable

## Comparison

| Approach              | Runtime Cost | Safety          | Syntax Complexity |
|-----------------------|--------------|-----------------|-------------------|
| C / C++               | Zero         | None            | High              |
| Garbage Collected     | Yes          | High            | Low               |
| Classic Rust          | Zero         | High            | Medium-High       |
| **PureLang**          | Zero         | High            | **Very Low**      |

PureLang takes the safety and performance of ownership systems and packages them in the simplest practical syntax.
