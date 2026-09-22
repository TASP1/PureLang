# PureLang Syntax Design

**Goal: Far easier than even Python**

PureLang prioritizes extreme readability and minimal cognitive load while remaining a high-performance systems language.

## Core Principles

- Minimal keywords
- Almost no noise / ceremony
- Immutable by default
- Ownership mostly invisible (compiler handles safety)
- One clear preferred way to do most things
- Looks clean and modern

## Basic Syntax

### Hello World
```pure
print "Hello, PureLang"
```

### Variables
```pure
name = "PureLang"          // immutable by default
mut count = 0              // only use `mut` when mutation is needed
count = count + 1
```

### Functions
```pure
fn add(a, b) {
    return a + b
}

fn greet(name) {
    print "Hello, " + name
}
```

### Control Flow
```pure
if score > 100 {
    print "You win!"
} else {
    print "Try again"
}

for i in 1..10 {
    print i
}

for item in list {
    print item
}
```

### Structs
```pure
struct Point {
    x
    y
}

p = Point(10, 20)
print p.x
```

### Ownership (Mostly Invisible)
```pure
fn process(data) {         // ownership handled automatically
    print data.length
}

numbers = [1, 2, 3]
process(numbers)           // ownership transferred safely by the compiler
```

You almost never need to write special ownership syntax. The compiler protects you.

### Error Handling
```pure
fn read_file(path) {
    content = file.read(path)?     // `?` propagates errors cleanly
    return content
}
```

### Full Small Example
```pure
fn main() {
    name = "Player"
    mut health = 100

    print "Welcome, " + name

    for i in 1..5 {
        health = health - 10
        print "Health: " + health

        if health <= 0 {
            print "Game Over"
            return
        }
    }

    print "You survived!"
}
```

## Key Differences from Python

| Feature              | Python                     | PureLang                          |
|----------------------|----------------------------|-----------------------------------|
| Mutability           | Everything mutable         | Immutable by default (safer)      |
| Function keyword     | `def`                      | `fn` (shorter)                    |
| Blocks               | Indentation only           | Braces (clearer + reliable)       |
| Ownership / Safety   | None                       | Automatic compile-time safety     |
| Error handling       | try/except boilerplate     | Simple `?`                        |
| Type noise           | Often messy                | Almost never needed               |

## Design Philosophy

Simple programs look extremely simple.  
Powerful features (ownership, types, generics, SIMD, etc.) are available when needed but stay out of the way for everyday code.

This syntax is intentionally designed so that someone who knows basic Python can become productive in PureLang within minutes, while still getting systems-level performance and safety.

### Methods (on structs)

```pure
struct Point {
    x
    y
}

fn Point.sum(self) {
    return self.x + self.y
}

fn Point.scale(self, factor) {
    return self.x * factor + self.y * factor
}

fn main() {
    p = Point(3, 4)
    print p.sum()      // 7
    print p.scale(2)   // 14
}
```

Define methods with `fn TypeName.method(self, ...)`. Call them with `obj.method(args)`.

### Enums and match

```pure
enum Color {
    Red
    Green
    Blue
}

enum Option {
    Some(value)
    None
}

fn main() {
    c = Color.Red
    match c {
        Color.Red => print "red"
        Color.Green => print "green"
        Color.Blue => print "blue"
    }

    o = Option.Some(42)
    match o {
        Option.Some(v) => print v
        Option.None => print "none"
    }
}
```

Unit variants and single-Number payload variants are supported. Match arms use `Enum.Variant => body` or `Enum.Variant(binding) => body`.

### Typed parameters

```pure
fn add(a: Number, b: Number) {
    return a + b
}

fn take(p: Point) {
    print p.x
}
```

### Error handling with `?`

```pure
enum Result {
    Ok(value)
    Err
}

fn maybe(x: Number) {
    if x < 0 {
        return Result.Err
    }
    return Result.Ok(x)
}

fn main() {
    v = maybe(10)?
    print v
}
```

### Modules

```pure
mod math {
    fn add(a: Number, b: Number) {
        return a + b
    }
}

fn main() {
    print math.add(1, 2)
}
```
