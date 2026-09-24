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


### Generics

```pure
fn id[T](x: T) {
    return x
}

fn main() {
    print id(42)
}
```

### Traits and impl

```pure
trait HasSum {
    fn sum(self)
}

impl HasSum for Point {
    fn sum(self) {
        return self.x + self.y
    }
}
```

### Visibility

```pure
mod util {
    pub fn open(x: Number) { return x }
    fn secret(x: Number) { return x }  // private
}

fn main() {
    print util.open(1)   // ok
    // print util.secret(1)  // error: private
}
```


### Standard library (math)

Always available (linked with `clang -lm`):

```pure
fn main() {
    print abs(0 - 5)     // 5
    print min(3, 9)      // 3
    print max(3, 9)      // 9
    print pow(2, 10)     // 1024
    print sqrt(49)       // 7
    print floor(10)
    print ceil(10)
    print sin(0)
    print cos(0)
    // also: std.sqrt(16), std.max(1, 100), ...
}
```

Functions: `abs`, `min`, `max`, `pow`, `sqrt`, `floor`, `ceil`, `round`, `sin`, `cos`, `tan`, `log`, `exp`
(and the same under the `std.` prefix).


### Maps

String keys → number values (runtime `PLMap`):

```pure
fn main() {
    m = map_new()
    map_set(m, "score", 42)
    print map_get(m, "score")
    print map_has(m, "score")
    print map_len(m)
}
```

### UI (HTML)

Generates `purelang_ui.html` in the working directory:

```pure
fn main() {
    ui_begin("My App", 480, 320)
    ui_label("Hello")
    ui_text("Built with PureLang")
    ui_button("OK")
    ui_end()
}
```

Open `purelang_ui.html` in a browser.


### String helpers

```pure
fn main() {
    print str_contains("PureLang", "Lang")  // 1
    print str_eq("a", "a")                  // 1
    s = str_concat("Hello, ", "world")
    print s
    print str_from_num(42)
}
```


### Number vs Float

- Integer-looking literals (`42`) have type **Number** (codegen `i64`).
- Literals with a decimal point (`1.5`) have type **Float** (codegen `double`).
- Mixing Number and Float in arithmetic promotes to **Float**.

```pure
fn main() {
    print 1.5 + 2.5   // 4
    print 10.0 / 4.0  // 2.5
    print 3 + 0.5     // 3.5
}
```

### Ownership (moves)

- **String** and **Map** move by default when assigned or passed by value.
- Field access, indexing, `print`, and reading builtins (`str_len`, `map_get`, …) **borrow**.
- Use after move is a type error.


### Process & time

```pure
fn main() {
    t = time_ms()   // monotonic milliseconds
    print t
    // exit(0)      // terminate process with code
}
```

### Maps

```pure
fn main() {
    m = map_new()
    map_set(m, "score", 42)
    print map_get(m, "score")
    print map_has(m, "score")
    print map_len(m)
}
```

### while / break / continue

```pure
fn main() {
    mut i = 0
    while i < 5 {
        i = i + 1
        if i == 3 {
            continue
        }
        print i
    }
}
```


### Sleep & HTTP

```pure
fn main() {
    sleep_ms(100)
    // HTTP/1.0 GET — http:// only (no TLS in MVP)
    body = http_get("http://example.com/")
    print str_len(body)
}
```
