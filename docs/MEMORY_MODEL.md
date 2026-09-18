# PureLang Memory Model

## Goal

Deliver **complete memory safety** (no use-after-free, no double-free, no data races, no buffer overflows in safe code) **with zero runtime overhead**.

This is achieved through **compile-time ownership and borrowing**, inspired by Rust, with design choices aimed at reducing annotation burden while preserving safety.

## Core Concepts

### 1. Ownership

Every value has exactly one owner at any time.

- When the owner goes out of scope, the value is automatically dropped (destructor runs).
- Ownership can be **moved** (transferred) to another variable or function.
- Ownership can be **borrowed** temporarily.

### 2. Borrowing

- **Immutable borrow** (`&T` or `read T`): Multiple simultaneous immutable references are allowed. No mutation while borrowed.
- **Mutable borrow** (`&mut T` or `mut T`): Exactly one mutable reference is allowed at a time. No other borrows can exist.

### 3. Lifetimes

The compiler tracks how long each reference is valid. Most lifetimes are inferred; explicit annotations are only required in complex cases (similar to modern Rust).

### Design Goals vs Pure Rust

| Aspect                  | PureLang Goal                          | Notes |
|-------------------------|----------------------------------------|-------|
| Safety                  | Same as Rust (compile-time proof)      |       |
| Annotation burden       | Lower than classic Rust                | Explore group borrowing, origin tracking, smarter inference |
| Escape hatch            | Explicit `unsafe` blocks               | Same philosophy as Rust |
| Interior mutability     | Safe patterns (similar to `RefCell`, `Mutex`) | Provided in std |
| Concurrency             | Data-race freedom by construction      |       |

## Comparison with Existing Languages

- **Rust**: Gold standard. PureLang starts here and aims for slightly better ergonomics.
- **Mojo**: Ownership + Python-like syntax. Excellent reference for reducing friction.
- **Zig**: Manual memory management with excellent tools, but no borrow checker → weaker default safety.
- **Go / Java / Python**: Rely on GC → runtime cost and non-determinism that PureLang explicitly rejects.

## Implementation Notes for the Compiler

1. The ownership/borrow checker runs after type checking.
2. It builds a control-flow graph and proves that borrowing rules are never violated.
3. Moves invalidate the previous owner.
4. Drop glue is inserted automatically by the compiler (RAII).
5. `unsafe` allows raw pointers and manual memory management when needed, but the boundary is explicit.

## Example (Conceptual Syntax)

```purelang
fn process(data: Vec<i32>) {          // takes ownership
    let first = &data[0];             // immutable borrow
    println(first);
    // data is still usable after the borrow ends
}

fn main() {
    let mut numbers = vec![1, 2, 3];
    process(numbers);                 // ownership moved
    // numbers is no longer valid here
}
```

This model guarantees that if a PureLang program compiles (in safe mode), it is free of an entire class of memory safety bugs that plague C/C++.
