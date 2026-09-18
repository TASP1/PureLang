# PureLang Syntax Design

## Guiding Principles

- **Readable first**: Should feel natural to TypeScript and Python developers.
- **Explicit when it matters**: Ownership, mutability, and types should be clear.
- **Minimal noise**: Avoid excessive ceremony while remaining unambiguous.
- **Progressive disclosure**: Simple programs look simple; advanced features are available when needed.

## High-Level Feel

PureLang aims for a hybrid of:

- Python / TypeScript indentation-friendly or light-brace style
- Rust’s explicitness around ownership and errors
- Modern type inference

## Conceptual Examples

### Hello World

```purelang
fn main() {
    println("Hello, PureLang!")
}
```

### Variables & Mutability

```purelang
let x = 42          // immutable by default
let mut y = 10      // explicitly mutable
y = y + 1
```

### Functions & Ownership

```purelang
fn take_ownership(v: Vec<i32>) {
    // v is owned here and will be dropped at the end of the function
}

fn borrow_immutably(v: &Vec<i32>) {
    println(v.len())
}

fn borrow_mutably(v: &mut Vec<i32>) {
    v.push(99)
}
```

### Structs

```purelang
struct Point {
    x: f64
    y: f64
}

fn distance(a: &Point, b: &Point) -> f64 {
    let dx = a.x - b.x
    let dy = a.y - b.y
    (dx*dx + dy*dy).sqrt()
}
```

### Error Handling (Conceptual)

```purelang
fn read_file(path: &str) -> Result<String, IoError> {
    // ...
}

fn main() {
    match read_file("config.toml") {
        Ok(content) => println(content),
        Err(e) => eprintln("Error: {e}")
    }
}
```

## Type System Highlights

- Strong static typing
- Powerful type inference (most annotations optional in local contexts)
- Generics / parametric polymorphism
- Traits / interfaces for shared behavior
- Algebraic data types (enums with payloads)

## Syntax Decisions Still Open

- Indentation vs braces (or both, like Python 3.12+ style)
- Exact keywords for ownership transfer (`move`, `^`, or implicit)
- Async / await syntax
- Pattern matching power level

These will be finalized after the first prototype compiler can run real programs.

The goal is that a developer coming from TypeScript or Python should be productive within hours, while still enjoying Rust-level safety and performance.
