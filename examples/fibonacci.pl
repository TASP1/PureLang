// PureLang Fibonacci example (conceptual)
// Will compile once the MVP compiler is ready.

fn fib(n: i32) -> i32 {
    if n < 2 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

fn main() {
    let result = fib(10)
    println("fib(10) = {result}")
}
