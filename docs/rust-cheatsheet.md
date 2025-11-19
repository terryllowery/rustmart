# Rust Cheatsheet - Comprehensive Guide

A comprehensive reference for Rust syntax, patterns, idioms, and when to use what.

## Official Documentation References

- **The Rust Book**: https://doc.rust-lang.org/book/
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **Standard Library Docs**: https://doc.rust-lang.org/std/
- **Rust Reference**: https://doc.rust-lang.org/reference/
- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Rust Patterns Book**: https://rust-unofficial.github.io/patterns/
- **Async Book**: https://rust-lang.github.io/async-book/
- **Rustlings (Interactive)**: https://github.com/rust-lang/rustlings

---

## Table of Contents

1. [Variables and Mutability](#variables-and-mutability)
2. [Data Types](#data-types)
3. [Functions](#functions)
4. [Control Flow](#control-flow)
5. [Ownership and Borrowing](#ownership-and-borrowing)
6. [Structs](#structs)
7. [Enums and Pattern Matching](#enums-and-pattern-matching)
8. [Error Handling](#error-handling)
9. [Collections](#collections)
10. [Traits](#traits)
11. [Generics and Lifetimes](#generics-and-lifetimes)
12. [Modules and Crates](#modules-and-crates)
13. [Async/Await](#asyncawait)
14. [Common Patterns](#common-patterns)
15. [Idiomatic Rust](#idiomatic-rust)
16. [When to Use What](#when-to-use-what)
17. [Cargo Commands](#cargo-commands)

---

## Variables and Mutability

**Reference**: [Rust Book - Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)

### Basic Variables

```rust
// ✅ Immutable by default (preferred)
let x = 5;
// x = 6; // ❌ Error: cannot assign twice to immutable variable

// ✅ Mutable when needed
let mut y = 5;
y = 6; // OK

// ✅ Constants (always immutable, SCREAMING_SNAKE_CASE)
const MAX_POINTS: u32 = 100_000;
const PI: f64 = 3.14159;

// ✅ Shadowing (creating new variable with same name)
let x = 5;
let x = x + 1;       // New variable, different value
let x = "string";    // New variable, different type
```

### When to Use What

| Use Case | Use | Why |
|----------|-----|-----|
| Value won't change | `let x = 5` | Prevents bugs, clearer intent |
| Value will change | `let mut x = 5` | Allows modification |
| True constant | `const MAX: u32 = 100` | Compile-time constant, no memory address |
| Transform value | Shadowing | Create new variable, possibly new type |

**Idiom**: Prefer immutable by default. Only use `mut` when necessary.

---

## Data Types

**Reference**: [Rust Book - Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)

### Scalar Types

```rust
// Integers
let a: i8 = 127;          // -128 to 127
let b: i32 = 42;          // -2^31 to 2^31-1 (default)
let c: i64 = 1_000_000;   // Underscores for readability
let d: u32 = 42;          // 0 to 2^32-1 (unsigned)
let e: usize = 42;        // Pointer-sized (use for indexing)

// Floats
let x: f32 = 3.14;        // 32-bit
let y: f64 = 3.14;        // 64-bit (default, more precise)

// Boolean
let t: bool = true;
let f = false;

// Character (Unicode, 4 bytes)
let c: char = 'z';
let emoji: char = '😀';
```

### String Types

```rust
// &str - String slice (immutable, fixed size)
let s1: &str = "hello";              // String literal
let s2: &str = &String::from("hi");  // Borrowed from String

// String - Owned string (growable, heap-allocated)
let s1 = String::from("hello");
let s2 = "hello".to_string();
let s3 = format!("hello {}", "world");

// When to use what
// Use &str for: function parameters, read-only strings
// Use String for: owned data, building/modifying strings
```

### Compound Types

```rust
// Tuple (fixed size, mixed types)
let tuple: (i32, f64, char) = (42, 3.14, 'x');
let (x, y, z) = tuple;           // Destructuring
let first = tuple.0;              // Index access

// Array (fixed size, same type)
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let arr = [0; 100];               // [0, 0, 0, ... 100 times]
let first = arr[0];

// Vector (dynamic size)
let mut vec: Vec<i32> = Vec::new();
let vec = vec![1, 2, 3];          // vec! macro
vec.push(4);
```

### When to Use What

| Type | Use When | Don't Use When |
|------|----------|----------------|
| `i32` | Default integer, most common | Need specific size |
| `u32` | Never negative, bit operations | Might be negative |
| `usize` | Array/vec indexing, sizes | General arithmetic |
| `f64` | Need precision (default) | Performance critical |
| `f32` | Memory/performance critical | Need precision |
| `&str` | Reading strings, function params | Need to own/modify |
| `String` | Building/owning strings | Just reading |
| Tuple | 2-4 related values | Many fields (use struct) |
| Array | Fixed size, stack allocated | Dynamic size (use Vec) |
| Vector | Dynamic size, unknown length | Fixed size (use array) |

---

## Functions

**Reference**: [Rust Book - Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html)

### Function Syntax

```rust
// ✅ Basic function
fn greet() {
    println!("Hello!");
}

// ✅ With parameters (type annotations required)
fn add(x: i32, y: i32) -> i32 {
    x + y  // ✅ Expression (no semicolon) = return value
}

// ❌ Less idiomatic (unnecessary return)
fn add_verbose(x: i32, y: i32) -> i32 {
    return x + y;  // Only use 'return' for early returns
}

// ✅ Early return pattern
fn divide(x: i32, y: i32) -> Result<i32, String> {
    if y == 0 {
        return Err("division by zero".to_string());
    }
    Ok(x / y)
}

// ✅ Multiple return values (tuple)
fn divmod(x: i32, y: i32) -> (i32, i32) {
    (x / y, x % y)
}

// Unit type () - "no return value"
fn do_side_effect() -> () {
    println!("Something");
    // Returns () implicitly
}

// ✅ Usually omit () return type
fn do_side_effect() {
    println!("Something");
}
```

### Function Patterns

```rust
// ✅ Builder pattern with self
impl User {
    fn new(name: String) -> Self {
        Self { name, age: 0 }
    }
    
    fn with_age(mut self, age: u32) -> Self {
        self.age = age;
        self
    }
}

let user = User::new("Terry".to_string()).with_age(25);

// ✅ Destructuring parameters
fn print_point((x, y): (i32, i32)) {
    println!("({}, {})", x, y);
}

// ✅ Ignoring parameters
fn ignore_second(x: i32, _: i32) -> i32 {
    x
}
```

**Idiom**: Omit semicolon on last expression to return it. Use `return` only for early returns.

---

## Control Flow

**Reference**: [Rust Book - Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html)

### If/Else

```rust
// ✅ Standard if/else
let number = 6;
if number % 2 == 0 {
    println!("even");
} else {
    println!("odd");
}

// ✅ If as expression (idiomatic!)
let result = if number > 5 { "big" } else { "small" };

// ✅ If let (pattern matching shorthand)
if let Some(value) = option {
    println!("Got: {}", value);
}

// ❌ Don't use if for simple checks - use match or if let
if option.is_some() {
    let value = option.unwrap(); // Bad!
}
```

### Loops

```rust
// ✅ Loop (infinite, use for event loops)
loop {
    if condition { break; }
}

// ✅ Loop with return value
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2; // Returns 20
    }
};

// ✅ While (condition-based)
while n > 0 {
    n -= 1;
}

// ✅ For (iterating, most common)
for i in 0..5 {              // Range: 0, 1, 2, 3, 4
    println!("{}", i);
}

for i in 0..=5 {             // Inclusive: 0, 1, 2, 3, 4, 5
    println!("{}", i);
}

for item in &vec {           // Borrow
    println!("{}", item);
}

for item in &mut vec {       // Mutable borrow
    *item += 1;
}

for item in vec {            // Take ownership (consumes vec)
    println!("{}", item);
}

// ✅ Enumerate (index + value)
for (i, value) in vec.iter().enumerate() {
    println!("{}: {}", i, value);
}

// ❌ Don't use C-style loops
for i in 0..vec.len() {
    println!("{}", vec[i]); // Bad! Use iterators
}
```

### Match (Pattern Matching)

```rust
// ✅ Match (exhaustive, powerful)
let number = 3;
match number {
    1 => println!("one"),
    2 | 3 => println!("two or three"),    // Multiple patterns
    4..=10 => println!("four to ten"),    // Range
    _ => println!("anything else"),        // Catch-all
}

// ✅ Match with binding
match number {
    n @ 1..=5 => println!("small: {}", n),
    n @ 6..=10 => println!("medium: {}", n),
    n => println!("large: {}", n),
}

// ✅ Match on enums (most common use)
match result {
    Ok(value) => println!("Success: {}", value),
    Err(e) => println!("Error: {}", e),
}

// ✅ Match guards
match number {
    n if n < 0 => println!("negative"),
    n if n > 0 => println!("positive"),
    _ => println!("zero"),
}

// ✅ Destructuring in match
match point {
    Point { x: 0, y: 0 } => println!("origin"),
    Point { x, y: 0 } => println!("x-axis at {}", x),
    Point { x, y } => println!("({}, {})", x, y),
}
```

### When to Use What

| Loop Type | Use When | Example |
|-----------|----------|---------|
| `for` | Iterating over collection | `for item in &vec` |
| `while` | Condition-based loop | `while !done` |
| `loop` | Infinite loop, need `break` value | Event loops |
| `match` | Pattern matching, enums | `match result { Ok/Err }` |
| `if let` | Single pattern match | `if let Some(x) = opt` |
| `if/else` | Simple boolean condition | `if x > 5` |

**Idiom**: Prefer `for` over manual indexing. Use `match` for exhaustive handling.

---

## Ownership and Borrowing

**Reference**: [Rust Book - Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)

### The Three Rules

1. Each value has an **owner**
2. There can only be **one owner** at a time
3. When owner goes out of scope, value is **dropped**

### Move Semantics

```rust
// ✅ Move (ownership transferred)
let s1 = String::from("hello");
let s2 = s1;  // s1 moved to s2, s1 is invalid
// println!("{}", s1); // ❌ Error: value moved

// ✅ Clone (deep copy, explicit)
let s1 = String::from("hello");
let s2 = s1.clone();
println!("{} {}", s1, s2); // Both valid

// ✅ Copy (implicit, for simple types)
let x = 5;
let y = x;  // x copied to y, both valid
println!("{} {}", x, y);

// Types that implement Copy: integers, floats, bool, char, tuples of Copy types
```

### Borrowing

**Reference**: [Rust Book - References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)

```rust
// ✅ Immutable borrow (&T) - read-only access
let s = String::from("hello");
let len = calculate_length(&s);  // Borrow, don't move
println!("{} is {} chars", s, len); // s still valid

fn calculate_length(s: &String) -> usize {
    s.len()
}

// ✅ Mutable borrow (&mut T) - read-write access
let mut s = String::from("hello");
change(&mut s);

fn change(s: &mut String) {
    s.push_str(", world");
}

// ✅ Multiple immutable borrows OK
let s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{} {}", r1, r2); // OK

// ❌ Cannot mix mutable and immutable borrows
let mut s = String::from("hello");
let r1 = &s;
// let r2 = &mut s; // ❌ Error: cannot borrow as mutable

// ❌ Only one mutable borrow at a time
let mut s = String::from("hello");
let r1 = &mut s;
// let r2 = &mut s; // ❌ Error: second mutable borrow
```

### Borrowing Rules

**The Golden Rules:**
1. ✅ Any number of immutable borrows (`&T`)
2. ✅ **OR** exactly one mutable borrow (`&mut T`)
3. ✅ References must always be valid (no dangling)

### When to Use What

| Pattern | Use When | Example |
|---------|----------|---------|
| `fn foo(s: String)` | Take ownership, consume value | Moving into thread |
| `fn foo(s: &String)` | Read-only access, don't own | Printing, reading |
| `fn foo(s: &mut String)` | Modify but don't own | Updating state |
| `fn foo(s: &str)` | Read-only, more flexible | String params (idiomatic) |
| `.clone()` | Need independent copy | Storing in multiple places |

**Idiom**: 
- Use `&str` for string parameters (more flexible than `&String`)
- Borrow by default, clone only when necessary
- Return owned values when creating new data

---

## Structs

**Reference**: [Rust Book - Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)

### Defining and Creating

```rust
// ✅ Named struct (most common)
#[derive(Debug, Clone)] // Common derives
struct User {
    username: String,
    email: String,
    age: u32,
    active: bool,
}

// Create instance
let user = User {
    username: String::from("terry"),
    email: String::from("terry@example.com"),
    age: 25,
    active: true,
};

// ✅ Field init shorthand (idiomatic!)
fn create_user(username: String, email: String) -> User {
    User {
        username,  // Same as username: username
        email,
        age: 0,
        active: true,
    }
}

// ✅ Struct update syntax
let user2 = User {
    email: String::from("new@example.com"),
    ..user  // Copy rest from user (moves non-Copy fields!)
};

// ✅ Tuple struct (when field names don't add clarity)
struct Color(u8, u8, u8);
struct Point(i32, i32);

let black = Color(0, 0, 0);
let origin = Point(0, 0);

// ✅ Unit struct (no fields, marker types)
struct AlwaysEqual;
let instance = AlwaysEqual;
```

### Methods and Associated Functions

**Reference**: [Rust Book - Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)

```rust
impl User {
    // ✅ Associated function (no self, like "static method")
    fn new(username: String, email: String) -> Self {
        Self {
            username,
            email,
            age: 0,
            active: true,
        }
    }
    
    // ✅ Method with immutable borrow
    fn is_adult(&self) -> bool {
        self.age >= 18
    }
    
    // ✅ Method with mutable borrow
    fn set_age(&mut self, age: u32) {
        self.age = age;
    }
    
    // ✅ Method that consumes self (takes ownership)
    fn into_email(self) -> String {
        self.email // self is consumed, can return owned field
    }
    
    // ✅ Builder pattern
    fn with_age(mut self, age: u32) -> Self {
        self.age = age;
        self
    }
}

// Usage patterns
let user = User::new("terry".to_string(), "t@ex.com".to_string());
let is_adult = user.is_adult();

let mut user = user; // Re-bind as mutable
user.set_age(25);

let email = user.into_email(); // user consumed

// Builder
let user = User::new("terry".to_string(), "t@ex.com".to_string())
    .with_age(25);
```

### Common Patterns

```rust
// ✅ Builder pattern (complex construction)
#[derive(Default)]
struct Config {
    host: String,
    port: u16,
    timeout: u64,
}

impl Config {
    fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<u64>,
}

impl ConfigBuilder {
    fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }
    
    fn build(self) -> Config {
        Config {
            host: self.host.unwrap_or_default(),
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
        }
    }
}

// Usage
let config = Config::builder()
    .host("localhost".to_string())
    .timeout(60)
    .build();
```

**Idiom**:
- Use `new` for constructors
- Use `Self` instead of struct name in impl blocks
- Use builder pattern for complex construction
- Derive `Debug` for all structs during development

---

## Enums and Pattern Matching

**Reference**: [Rust Book - Enums](https://doc.rust-lang.org/book/ch06-00-enums.html)

### Defining Enums

```rust
// ✅ Simple enum (C-style)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    Active,
    Inactive,
    Pending,
}

// ✅ Enum with data (Rust's superpower!)
enum Message {
    Quit,                        // No data
    Move { x: i32, y: i32 },    // Named fields (like struct)
    Write(String),               // Tuple-like
    ChangeColor(u8, u8, u8),    // Multiple values
}

// ✅ Enum with methods
impl Message {
    fn process(&self) {
        match self {
            Message::Quit => println!("Quitting"),
            Message::Move { x, y } => println!("Moving to ({}, {})", x, y),
            Message::Write(text) => println!("Writing: {}", text),
            Message::ChangeColor(r, g, b) => println!("Color: ({}, {}, {})", r, g, b),
        }
    }
}
```

### Option<T>

**Reference**: [Rust Book - Option](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values)

```rust
// Option replaces null/nil in Rust
enum Option<T> {
    Some(T),
    None,
}

// ✅ Creating Options
let some_number: Option<i32> = Some(5);
let no_number: Option<i32> = None;

// ✅ Pattern matching (explicit)
match some_number {
    Some(n) => println!("Got: {}", n),
    None => println!("Got nothing"),
}

// ✅ if let (single pattern)
if let Some(n) = some_number {
    println!("Got: {}", n);
}

// ✅ Option methods (idiomatic!)
let value = some_number.unwrap_or(0);              // Default value
let value = some_number.unwrap_or_else(|| 0);      // Lazy default
let value = some_number.expect("no value");        // Panic with message
let doubled = some_number.map(|n| n * 2);          // Transform Some
let filtered = some_number.filter(|&n| n > 3);     // Keep if predicate true
let chained = some_number.and_then(|n| Some(n * 2)); // Chain operations

// ✅ ? operator (propagate None)
fn divide(x: i32, y: i32) -> Option<i32> {
    if y == 0 { None } else { Some(x / y) }
}

fn calculate() -> Option<i32> {
    let result = divide(10, 2)?; // Returns None if divide returns None
    Some(result + 1)
}
```

### Result<T, E>

**Reference**: [Rust Book - Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)

```rust
// Result for operations that can fail
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// ✅ Creating Results
fn divide(x: i32, y: i32) -> Result<i32, String> {
    if y == 0 {
        Err("division by zero".to_string())
    } else {
        Ok(x / y)
    }
}

// ✅ Pattern matching
match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e),
}

// ✅ Result methods (idiomatic!)
let value = divide(10, 2).unwrap_or(0);
let value = divide(10, 2).expect("Failed to divide");
let doubled = divide(10, 2).map(|n| n * 2);
let result = divide(10, 2).map_err(|e| format!("Error: {}", e));

// ✅ ? operator (propagate error)
fn calculate() -> Result<i32, String> {
    let result = divide(10, 2)?; // Returns Err early if divide fails
    let result2 = divide(result, 3)?;
    Ok(result2 + 1)
}

// ✅ Combining multiple Results
let results: Vec<Result<i32, String>> = vec![Ok(1), Ok(2), Ok(3)];
let sum: Result<i32, String> = results.into_iter()
    .try_fold(0, |acc, r| r.map(|n| acc + n));
```

### Pattern Matching Best Practices

```rust
// ✅ Exhaustive matching (compiler enforced)
match status {
    Status::Active => { },
    Status::Inactive => { },
    Status::Pending => { },
    // Compiler error if you forget a variant!
}

// ✅ Use _ for catch-all
match number {
    1 => println!("one"),
    2 => println!("two"),
    _ => println!("other"),
}

// ✅ Match guards
match point {
    Point { x, y } if x == y => println!("diagonal"),
    Point { x, .. } if x > 0 => println!("right side"),
    _ => println!("other"),
}

// ✅ Destructuring
match message {
    Message::Move { x: 0, y } => println!("y-axis: {}", y),
    Message::Move { x, y } => println!("({}, {})", x, y),
    _ => { },
}

// ✅ @ binding
match number {
    n @ 1..=5 => println!("small: {}", n),
    n @ 6..=10 => println!("medium: {}", n),
    n => println!("large: {}", n),
}
```

**Idiom**:
- Use `Option` instead of null/nil patterns
- Use `Result` for recoverable errors
- Use `?` operator to propagate errors/None
- Prefer method chaining (`.map`, `.and_then`) over explicit matches
- Always handle all enum variants (exhaustive matching)

---

## Error Handling

**Reference**: [Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

### Error Types

```rust
// ✅ Using Result with custom error
fn parse_number(s: &str) -> Result<i32, String> {
    s.parse().map_err(|e| format!("Parse error: {}", e))
}

// ✅ Custom error enum
#[derive(Debug)]
enum MyError {
    ParseError(String),
    NotFound,
    PermissionDenied,
}

// ✅ Using thiserror crate (idiomatic!)
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {field}")]
    Invalid { field: String },
}

// ✅ Using anyhow for applications (not libraries!)
use anyhow::{Result, Context};

fn do_something() -> Result<()> {
    let file = std::fs::read_to_string("config.txt")
        .context("Failed to read config")?;
    Ok(())
}
```

### Error Handling Patterns

```rust
// ✅ ? operator (idiomatic!)
fn process() -> Result<i32, AppError> {
    let data = read_file()?;        // Early return if error
    let parsed = parse_data(&data)?;
    Ok(parsed * 2)
}

// ✅ Match for different error handling
match result {
    Ok(value) => println!("Success: {}", value),
    Err(AppError::NotFound(msg)) => println!("Not found: {}", msg),
    Err(e) => println!("Other error: {}", e),
}

// ✅ unwrap_or for defaults
let value = result.unwrap_or(0);
let value = result.unwrap_or_else(|_| compute_default());

// ✅ map and and_then for chaining
let result = read_file()
    .map(|content| content.trim())
    .and_then(|content| parse_content(content))?;

// ❌ Avoid unwrap in production
let value = result.unwrap(); // Panics on error - only for prototypes!

// ✅ Use expect with descriptive message
let value = result.expect("Config file must exist");

// ✅ Propagate errors up
fn outer() -> Result<()> {
    inner()?; // Propagate error
    Ok(())
}
```

### When to Use What

| Pattern | Use When | Example |
|---------|----------|---------|
| `Result<T, E>` | Recoverable errors | File I/O, parsing |
| `Option<T>` | Value might not exist | Finding in collection |
| `panic!` | Unrecoverable errors | Broken invariants |
| `unwrap()` | Prototyping, you're sure it won't fail | Testing |
| `expect()` | Like unwrap, but with context | Setup code |
| `?` operator | Propagating errors | Most error handling |
| `thiserror` | Library error types | Public error enums |
| `anyhow` | Application errors | main(), CLI tools |

**Idiom**:
- Use `Result` for fallible operations
- Use `?` operator to propagate errors
- Use `thiserror` for library errors
- Use `anyhow` for application errors
- Reserve `panic!` for truly unrecoverable situations

---

## Collections

**Reference**: [Rust Book - Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html)

### Vector

```rust
// ✅ Creating vectors
let v: Vec<i32> = Vec::new();
let v = vec![1, 2, 3];              // vec! macro (most common)
let v = Vec::with_capacity(10);     // Pre-allocate

// ✅ Adding elements
let mut v = vec![1, 2, 3];
v.push(4);
v.extend([5, 6, 7]);

// ✅ Accessing elements
let third = &v[2];                  // Panics if out of bounds
let third = v.get(2);               // Returns Option<&T>

if let Some(third) = v.get(2) {
    println!("{}", third);
}

// ✅ Iterating (see Iterators section below)
for item in &v {                    // Borrow
    println!("{}", item);
}

for item in &mut v {                // Mutable borrow
    *item += 1;
}

for item in v {                     // Take ownership (consumes v)
    println!("{}", item);
}

// ✅ Vector methods
v.len();
v.is_empty();
v.contains(&3);
v.pop();                            // Remove last, returns Option<T>
v.remove(0);                        // Remove at index
v.clear();                          // Remove all
```

### String

```rust
// ✅ Creating strings
let s = String::new();
let s = String::from("hello");
let s = "hello".to_string();
let s = format!("{} {}", "hello", "world"); // Doesn't take ownership!

// ✅ Appending
let mut s = String::from("hello");
s.push_str(" world");               // Append &str
s.push('!');                        // Append char

// ✅ Concatenation
let s1 = String::from("Hello, ");
let s2 = String::from("world!");
let s3 = s1 + &s2;                  // s1 moved, s2 borrowed

// ✅ Format macro (idiomatic!)
let s1 = String::from("Hello");
let s2 = String::from("world");
let s3 = format!("{}, {}!", s1, s2); // Doesn't move s1 or s2

// ✅ Iterating
for c in "hello".chars() {          // Unicode scalars
    println!("{}", c);
}

for b in "hello".bytes() {          // Raw bytes
    println!("{}", b);
}

// ❌ Cannot index directly
// let h = s[0]; // Error: strings are UTF-8

// ✅ Slicing (use with caution, must be valid UTF-8 boundary)
let hello = "Здравствуйте";
let s = &hello[0..4];               // First 2 chars (4 bytes)

// ✅ String methods
s.len();                            // Bytes, not chars!
s.is_empty();
s.contains("world");
s.starts_with("hello");
s.ends_with("!");
s.trim();                           // Remove whitespace
s.replace("old", "new");
s.split_whitespace();
s.lines();
```

### HashMap

**Reference**: [Rust Book - Hash Maps](https://doc.rust-lang.org/book/ch08-03-hash-maps.html)

```rust
use std::collections::HashMap;

// ✅ Creating
let mut scores: HashMap<String, i32> = HashMap::new();
let scores: HashMap<&str, i32> = [("Blue", 10), ("Red", 50)].into_iter().collect();

// ✅ Inserting
scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Red"), 50);

// ✅ Accessing
let score = scores.get("Blue");     // Returns Option<&V>

if let Some(score) = scores.get("Blue") {
    println!("Blue: {}", score);
}

// ✅ Iterating
for (key, value) in &scores {
    println!("{}: {}", key, value);
}

// ✅ Updating
scores.insert(String::from("Blue"), 25);        // Overwrite

scores.entry(String::from("Yellow")).or_insert(50); // Insert if absent

let score = scores.entry(String::from("Blue")).or_insert(0);
*score += 10;                       // Update based on old value

// ✅ Removing
scores.remove("Blue");
```

### When to Use What

| Collection | Use When | Don't Use When |
|------------|----------|----------------|
| `Vec<T>` | Sequential access, unknown size | Need key-value, middle inserts |
| `[T; N]` | Fixed size, stack allocated | Size unknown at compile time |
| `String` | Owned UTF-8 text | Read-only (use `&str`) |
| `&str` | String slice, function param | Need to modify |
| `HashMap<K, V>` | Key-value mapping | Need ordering (use `BTreeMap`) |
| `HashSet<T>` | Unique values, fast lookup | Need ordering (use `BTreeSet`) |
| `VecDeque<T>` | Need push/pop from both ends | Only push/pop from one end |
| `LinkedList<T>` | Rarely! | Almost always use `Vec` instead |

---

## Traits

**Reference**: [Rust Book - Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)

### Defining Traits

```rust
// ✅ Basic trait
trait Summary {
    fn summarize(&self) -> String;
}

// ✅ Trait with default implementation
trait Summary {
    fn summarize(&self) -> String {
        String::from("(Read more...)")
    }
    
    fn summarize_author(&self) -> String;
    
    // Can call other methods
    fn full_summary(&self) -> String {
        format!("{} by {}", self.summarize(), self.summarize_author())
    }
}

// ✅ Implementing traits
struct Article {
    headline: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}", self.headline, self.content)
    }
    
    fn summarize_author(&self) -> String {
        String::from("Author Name")
    }
}
```

### Trait Bounds

```rust
// ✅ Trait as parameter
fn notify(item: &impl Summary) {
    println!("{}", item.summarize());
}

// ✅ Trait bound syntax (equivalent, more explicit)
fn notify<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}

// ✅ Multiple trait bounds
fn notify<T: Summary + Display>(item: &T) { }

// ✅ Where clause (more readable)
fn notify<T>(item: &T)
where
    T: Summary + Display,
{ }

// ✅ Return traits
fn make_article() -> impl Summary {
    Article {
        headline: String::from("Title"),
        content: String::from("Content"),
    }
}

// ❌ Cannot return different types
fn make_summary(is_article: bool) -> impl Summary {
    if is_article {
        Article { /* ... */ }  // ❌ Error
    } else {
        Tweet { /* ... */ }    // Different type!
    }
}

// ✅ Use Box<dyn Trait> for dynamic dispatch
fn make_summary(is_article: bool) -> Box<dyn Summary> {
    if is_article {
        Box::new(Article { /* ... */ })
    } else {
        Box::new(Tweet { /* ... */ })
    }
}
```

### Common Traits

```rust
// ✅ Debug - {:?} formatting
#[derive(Debug)]
struct Point { x: i32, y: i32 }
println!("{:?}", point);

// ✅ Clone - explicit deep copy
#[derive(Clone)]
struct Point { x: i32, y: i32 }
let p2 = p1.clone();

// ✅ Copy - implicit copy (for simple types only!)
#[derive(Copy, Clone)]  // Copy requires Clone
struct Point { x: i32, y: i32 }
let p2 = p1;  // p1 still valid (copied, not moved)

// ✅ PartialEq - == and !=
#[derive(PartialEq)]
struct Point { x: i32, y: i32 }
assert_eq!(p1, p2);

// ✅ Eq - full equality (PartialEq + reflexive)
#[derive(PartialEq, Eq)]
struct Point { x: i32, y: i32 }

// ✅ PartialOrd - <, >, <=, >=
#[derive(PartialOrd)]
struct Point { x: i32, y: i32 }

// ✅ Ord - full ordering
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Point { x: i32, y: i32 }

// ✅ Display - {} formatting
use std::fmt;

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// ✅ Default - default values
#[derive(Default)]
struct Config {
    host: String,  // ""
    port: u16,     // 0
}

let config = Config::default();

// ✅ From/Into - type conversions
impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Point { x, y }
    }
}

let point = Point::from((1, 2));
let point: Point = (1, 2).into();  // Into is automatic
```

### Trait Objects (Dynamic Dispatch)

```rust
// ✅ Box<dyn Trait> - heap allocated, sized
let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle { radius: 5 }),
    Box::new(Rectangle { width: 10, height: 5 }),
];

for shape in shapes {
    shape.draw();
}

// ✅ &dyn Trait - borrowed
fn draw_shape(shape: &dyn Draw) {
    shape.draw();
}

// When to use trait objects:
// - Need collection of different types implementing same trait
// - Runtime polymorphism
// - Trade-off: slight performance cost (virtual dispatch)
```

**Idiom**:
- Derive common traits (`Debug`, `Clone`, `PartialEq`, etc.)
- Use `impl Trait` for return types when possible
- Use `Box<dyn Trait>` when need different types at runtime
- Implement `From/Into` for conversions instead of custom methods

---

## Generics and Lifetimes

**Reference**: [Rust Book - Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html)

### Generics

```rust
// ✅ Generic function
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// ✅ Generic struct
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

// ✅ Multiple type parameters
struct Point<T, U> {
    x: T,
    y: U,
}

// ✅ Generic enum
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// ✅ Impl block with constraints
impl<T: Display> Point<T> {
    fn print(&self) {
        println!("({}, {})", self.x, self.y);
    }
}

// Specific type impl
impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

### Lifetimes

**Reference**: [Rust Book - Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)

```rust
// ✅ Lifetime annotation (tells compiler how references relate)
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// 'a means: returned reference lives as long as both x and y

// ✅ Struct with lifetime (holds reference)
struct ImportantExcerpt<'a> {
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
    fn level(&self) -> i32 {
        3
    }
    
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention: {}", announcement);
        self.part
    }
}

// ✅ Static lifetime (lives for entire program)
let s: &'static str = "I have a static lifetime";

// ✅ Multiple lifetimes
fn first_word<'a, 'b>(s: &'a str, _other: &'b str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}
```

### Lifetime Elision Rules

Most of the time, you don't need to write lifetimes explicitly:

```rust
// ❌ Explicit (before elision rules)
fn first_word<'a>(s: &'a str) -> &'a str { }

// ✅ Elided (compiler infers)
fn first_word(s: &str) -> &str { }

// Elision rules:
// 1. Each parameter gets its own lifetime
// 2. If one input lifetime, assigned to all outputs
// 3. If multiple input lifetimes and one is &self, 
//    self's lifetime assigned to all outputs
```

**When you need explicit lifetimes:**
- Multiple references in, can't infer which relates to output
- Struct holds references
- Complex relationships between references

**Idiom**:
- Let compiler infer lifetimes when possible
- Only add explicit lifetimes when compiler asks
- Use `'static` sparingly (usually only for string literals)

---

## Modules and Crates

**Reference**: [Rust Book - Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)

### Module Basics

```rust
// ✅ Inline module
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
        
        fn seat_at_table() {} // Private
    }
    
    mod serving {  // Private module
        fn take_order() {}
    }
}

// ✅ Using modules
use front_of_house::hosting;
hosting::add_to_waitlist();

// Or bring function directly
use front_of_house::hosting::add_to_waitlist;
add_to_waitlist();

// ✅ Re-export (make public from this module)
pub use front_of_house::hosting;
```

### File-Based Modules

```
src/
├── main.rs
├── lib.rs
└── front_of_house/
    ├── mod.rs          # Module root
    ├── hosting.rs
    └── serving.rs
```

```rust
// lib.rs
pub mod front_of_house;  // Loads front_of_house/mod.rs

// front_of_house/mod.rs
pub mod hosting;  // Loads front_of_house/hosting.rs
pub mod serving;

// front_of_house/hosting.rs
pub fn add_to_waitlist() { }

// main.rs
use my_crate::front_of_house::hosting;

fn main() {
    hosting::add_to_waitlist();
}
```

### Use Patterns

```rust
// ✅ Bring module into scope
use std::collections::HashMap;

// ✅ Multiple items
use std::collections::{HashMap, HashSet, BTreeMap};

// ✅ Nested paths with self
use std::io::{self, Write};  // Imports std::io and std::io::Write

// ✅ Glob (use sparingly!)
use std::collections::*;

// ✅ Rename
use std::io::Result as IoResult;
use std::fmt::Result as FmtResult;

// ✅ Idiomatic: bring module, not function (for clarity)
use std::collections::HashMap;
let map = HashMap::new();  // Clear it's from HashMap

// ❌ Less clear
use std::collections::HashMap::new;
let map = new();  // What type?
```

### Visibility

```rust
// Private by default
fn private_function() { }
struct PrivateStruct { }

// ✅ Public
pub fn public_function() { }
pub struct PublicStruct { }

// ✅ Public struct, private fields
pub struct Person {
    pub name: String,  // Public
    age: u32,          // Private
}

// ✅ Public enum, all variants automatically public
pub enum Status {
    Active,    // Public
    Inactive,  // Public
}

// ✅ Crate-visible (internal API)
pub(crate) fn internal_api() { }

// ✅ Parent module visible
pub(super) fn parent_only() { }

// ✅ Specific path visible
pub(in crate::front_of_house) fn specific_module_only() { }
```

**Idiom**:
- Organize code into modules by feature/domain
- Use `pub` judiciously - keep internals private
- Use `pub(crate)` for internal APIs
- Re-export at crate root for better ergonomics
- Prefer `use module::Type` over `use module::Type::function`

---

## Async/Await

**Reference**: [Async Book](https://rust-lang.github.io/async-book/)

### Async Basics

```rust
// ✅ Async function
async fn hello() -> String {
    String::from("Hello")
}

// ✅ Await async function
async fn greet() {
    let message = hello().await;
    println!("{}", message);
}

// ✅ Async block
let future = async {
    let result = some_async_function().await;
    result + 1
};

// ✅ Async main with Tokio
#[tokio::main]
async fn main() {
    greet().await;
}

// Or manually
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(greet());
}
```

### Tokio Patterns

```rust
use tokio;

// ✅ Spawn task (runs concurrently)
tokio::spawn(async {
    println!("Running in background");
});

// ✅ Join multiple futures (run concurrently)
let (result1, result2) = tokio::join!(
    async_function1(),
    async_function2(),
);

// ✅ Select (race futures, first to complete wins)
tokio::select! {
    val = async_function1() => println!("Got {}", val),
    val = async_function2() => println!("Got {}", val),
}

// ✅ Timeout
use tokio::time::{timeout, Duration};

match timeout(Duration::from_secs(5), long_operation()).await {
    Ok(result) => println!("Completed: {:?}", result),
    Err(_) => println!("Timeout!"),
}

// ✅ Sleep
use tokio::time::{sleep, Duration};

sleep(Duration::from_secs(1)).await;

// ✅ Spawn blocking (for CPU-heavy or blocking operations)
let result = tokio::task::spawn_blocking(|| {
    // Heavy computation or blocking I/O
    expensive_operation()
}).await?;
```

### Async Traits and Error Handling

```rust
// ✅ Async function with Result
async fn fetch_data() -> Result<String, reqwest::Error> {
    let response = reqwest::get("https://example.com").await?;
    let body = response.text().await?;
    Ok(body)
}

// ✅ Using async in traits (requires async-trait crate)
use async_trait::async_trait;

#[async_trait]
trait DataFetcher {
    async fn fetch(&self) -> Result<String, Error>;
}

#[async_trait]
impl DataFetcher for MyFetcher {
    async fn fetch(&self) -> Result<String, Error> {
        // Implementation
    }
}
```

### When to Use Async

| Use Async When | Use Sync When |
|----------------|---------------|
| I/O-bound (network, disk) | CPU-bound (computation) |
| Many concurrent operations | Sequential operations |
| Need high concurrency | Simple, short operations |
| Web servers, APIs | Scripts, CLI tools |

**Idiom**:
- Use `tokio::spawn` for background tasks
- Use `tokio::join!` to run futures concurrently
- Use `tokio::select!` for racing futures
- Use `spawn_blocking` for CPU-intensive work
- Don't mix blocking and async code without `spawn_blocking`

---

## Common Patterns

### Iterator Patterns

**Reference**: [Rust Book - Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)

```rust
let vec = vec![1, 2, 3, 4, 5];

// ✅ Map (transform each element)
let doubled: Vec<i32> = vec.iter().map(|x| x * 2).collect();

// ✅ Filter (keep elements matching predicate)
let evens: Vec<&i32> = vec.iter().filter(|&&x| x % 2 == 0).collect();

// ✅ Filter_map (filter and map in one step)
let results: Vec<i32> = vec.iter()
    .filter_map(|&x| if x > 2 { Some(x * 2) } else { None })
    .collect();

// ✅ Fold (reduce to single value)
let sum: i32 = vec.iter().fold(0, |acc, &x| acc + x);

// ✅ Sum (common fold operation)
let sum: i32 = vec.iter().sum();

// ✅ Chain (concatenate iterators)
let chain: Vec<_> = vec.iter()
    .chain(vec2.iter())
    .collect();

// ✅ Zip (pair elements from two iterators)
let pairs: Vec<_> = vec.iter().zip(vec2.iter()).collect();

// ✅ Enumerate (add index)
for (i, &value) in vec.iter().enumerate() {
    println!("{}: {}", i, value);
}

// ✅ Take/Skip
let first_three: Vec<_> = vec.iter().take(3).collect();
let after_two: Vec<_> = vec.iter().skip(2).collect();

// ✅ Find (first matching element)
let found = vec.iter().find(|&&x| x > 3);

// ✅ Any/All
let has_even = vec.iter().any(|&x| x % 2 == 0);
let all_positive = vec.iter().all(|&x| x > 0);

// ✅ Partition (split into two collections)
let (evens, odds): (Vec<_>, Vec<_>) = vec.iter()
    .partition(|&&x| x % 2 == 0);
```

### Builder Pattern

```rust
// ✅ Builder with consuming methods
struct Request {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

struct RequestBuilder {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl RequestBuilder {
    fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: "GET".to_string(),
            headers: vec![],
            body: None,
        }
    }
    
    fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }
    
    fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
    
    fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
    
    fn build(self) -> Request {
        Request {
            url: self.url,
            method: self.method,
            headers: self.headers,
            body: self.body,
        }
    }
}

// Usage
let request = RequestBuilder::new("https://example.com")
    .method("POST")
    .header("Content-Type", "application/json")
    .body("{\"key\": \"value\"}")
    .build();
```

### Newtype Pattern

```rust
// ✅ Wrap existing type for type safety
struct Meters(f64);
struct Kilometers(f64);

impl Meters {
    fn to_kilometers(&self) -> Kilometers {
        Kilometers(self.0 / 1000.0)
    }
}

// ❌ Can't accidentally mix up
fn distance_in_meters(m: Meters) -> f64 {
    m.0
}

// Compile error:
// let km = Kilometers(5.0);
// distance_in_meters(km); // Error: expected Meters, got Kilometers

// ✅ Wrap to implement trait on external type
struct Wrapper(Vec<String>);

impl Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join(", "))
    }
}
```

### State Pattern with Type States

```rust
// ✅ Encode state in types (compile-time state machine)
struct Locked;
struct Unlocked;

struct Door<State> {
    state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        Self { state: PhantomData }
    }
    
    fn unlock(self, key: &Key) -> Door<Unlocked> {
        Door { state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        Door { state: PhantomData }
    }
    
    fn open(&self) {
        println!("Door opened");
    }
}

// Usage
let door = Door::new();           // Locked
// door.open();                   // ❌ Compile error!
let door = door.unlock(&key);     // Unlocked
door.open();                      // ✅ OK
```

---

## Idiomatic Rust

### Naming Conventions

**Reference**: [API Guidelines - Naming](https://rust-lang.github.io/api-guidelines/naming.html)

```rust
// ✅ Types: UpperCamelCase
struct UserAccount { }
enum HttpStatus { }
trait Drawable { }

// ✅ Functions, methods, variables: snake_case
fn calculate_total() { }
let user_name = "Terry";

// ✅ Constants, statics: SCREAMING_SNAKE_CASE
const MAX_POINTS: u32 = 100_000;
static GLOBAL_STATE: AtomicUsize = AtomicUsize::new(0);

// ✅ Lifetimes: short lowercase
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { }

// ✅ Type parameters: Single capital letter or UpperCamelCase
fn generic<T>(item: T) { }
fn generic<TEntity>(entity: TEntity) { }

// ✅ Feature names: kebab-case
// In Cargo.toml:
[features]
database-support = []

// ✅ Crate names: kebab-case (snake_case in code)
// Crate: my-awesome-crate
// In code: use my_awesome_crate;
```

### Method Naming Conventions

```rust
impl User {
    // ✅ Constructors: new (most common), default, with_*
    fn new(name: String) -> Self { }
    fn default() -> Self { }
    fn with_capacity(cap: usize) -> Self { }
    
    // ✅ Conversions
    fn as_str(&self) -> &str { }        // Cheap borrow
    fn to_string(&self) -> String { }   // Expensive conversion
    fn into_string(self) -> String { }  // Take ownership
    
    // ✅ Boolean checks: is_*, has_*, can_*
    fn is_admin(&self) -> bool { }
    fn has_permission(&self) -> bool { }
    fn can_edit(&self) -> bool { }
    
    // ✅ Getters: no get_ prefix (unless getting by computation)
    fn name(&self) -> &str { }          // Simple field access
    fn get_computed_value(&self) -> i32 { }  // Computation involved
    
    // ✅ Setters: set_*
    fn set_name(&mut self, name: String) { }
    
    // ✅ Builders: with_* (consume and return Self)
    fn with_age(mut self, age: u32) -> Self { }
    
    // ✅ Try operations: try_*
    fn try_parse(&self) -> Result<Data, Error> { }
}
```

### Error Handling Idioms

```rust
// ✅ Use ? operator
fn process() -> Result<Data, Error> {
    let file = read_file()?;
    let parsed = parse_data(&file)?;
    Ok(parsed)
}

// ✅ Provide context with .context()
use anyhow::Context;

fn process() -> Result<Data> {
    let file = read_file()
        .context("Failed to read config file")?;
    Ok(file)
}

// ✅ Early returns for error conditions
fn divide(x: i32, y: i32) -> Result<i32, String> {
    if y == 0 {
        return Err("division by zero".to_string());
    }
    Ok(x / y)
}

// ✅ Custom error types with thiserror
#[derive(Error, Debug)]
enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
}
```

### Option Handling Idioms

```rust
// ✅ Use combinators instead of match
let result = maybe_value
    .map(|x| x * 2)
    .filter(|&x| x > 10)
    .unwrap_or(0);

// ❌ Verbose match
let result = match maybe_value {
    Some(x) => {
        let doubled = x * 2;
        if doubled > 10 {
            doubled
        } else {
            0
        }
    }
    None => 0,
};

// ✅ Use if let for single pattern
if let Some(value) = maybe_value {
    println!("Got: {}", value);
}

// ✅ Use unwrap_or for defaults
let value = maybe_value.unwrap_or(DEFAULT);

// ✅ Use ? for early returns
fn get_value() -> Option<i32> {
    let x = maybe_value?;
    Some(x * 2)
}
```

### Collection Idioms

```rust
// ✅ Use iterator methods instead of loops
let sum: i32 = vec.iter().sum();

// ❌ Manual loop
let mut sum = 0;
for &x in &vec {
    sum += x;
}

// ✅ Collect into specific type
let doubled: Vec<_> = vec.iter().map(|x| x * 2).collect();
let set: HashSet<_> = vec.into_iter().collect();

// ✅ Chain operations
let result: Vec<_> = vec.iter()
    .filter(|&&x| x > 0)
    .map(|&x| x * 2)
    .take(10)
    .collect();

// ✅ Use entry API for HashMap
let count = map.entry(key).or_insert(0);
*count += 1;

// ❌ Check then insert
if !map.contains_key(&key) {
    map.insert(key, 0);
}
let count = map.get_mut(&key).unwrap();
*count += 1;
```

### Type Conversion Idioms

```rust
// ✅ Implement From (gets Into for free)
impl From<i32> for MyType {
    fn from(value: i32) -> Self {
        MyType { value }
    }
}

let my_type: MyType = 42.into();  // Into is automatic!

// ✅ Use Into for flexible parameters
fn takes_string(s: impl Into<String>) {
    let s: String = s.into();
}

takes_string("string slice");
takes_string(String::from("owned"));

// ✅ Use AsRef for borrowing conversions
fn takes_str(s: impl AsRef<str>) {
    let s: &str = s.as_ref();
}

takes_str("string slice");
takes_str(&String::from("owned"));
```

### Documentation Idioms

**Reference**: [API Guidelines - Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)

```rust
/// Brief one-line description
///
/// More detailed description after blank line.
///
/// # Examples
///
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 84);
/// ```
///
/// # Errors
///
/// Returns `MyError::Invalid` if input is negative.
///
/// # Panics
///
/// Panics if input is zero.
///
/// # Safety
///
/// (For unsafe functions) Caller must ensure...
pub fn my_function(x: i32) -> Result<i32, MyError> {
    // Implementation
}

/// Module-level documentation
//! This module contains utilities for parsing.
//!
//! # Examples
//! ```
//! use my_crate::parser;
//! ```
```

---

## When to Use What

### Choosing Between Types

| Scenario | Use | Why |
|----------|-----|-----|
| Function parameter (string) | `&str` | Most flexible, borrows |
| Return new string | `String` | Caller owns data |
| Store owned string | `String` | Need to own data |
| String constants | `&'static str` | Embedded in binary |
| Collection of unknown size | `Vec<T>` | Dynamic, efficient |
| Collection of fixed size | `[T; N]` | Stack allocated, fast |
| Key-value mapping | `HashMap<K, V>` | Fast lookups |
| Ordered key-value | `BTreeMap<K, V>` | Sorted keys |
| Unique values | `HashSet<T>` | Fast membership test |
| Optional value | `Option<T>` | May or may not exist |
| Fallible operation | `Result<T, E>` | Can return error |
| Shared ownership | `Rc<T>` | Single-threaded |
| Shared ownership (thread-safe) | `Arc<T>` | Multi-threaded |
| Interior mutability | `RefCell<T>` | Single-threaded |
| Interior mutability (thread-safe) | `Mutex<T>`, `RwLock<T>` | Multi-threaded |

### Choosing Between Patterns

| Goal | Pattern | Example |
|------|---------|---------|
| Handle optional value | `Option<T>` | `Some(x)`, `None` |
| Handle errors | `Result<T, E>` | `Ok(x)`, `Err(e)` |
| Multiple related constants | `enum` | `Status::Active` |
| Multiple named fields | `struct` | `User { name, age }` |
| 2-4 unnamed fields | tuple | `(x, y)` |
| Type with behavior | trait | `trait Draw { fn draw(&self); }` |
| Share code across types | trait with default impl | Common methods |
| Complex construction | Builder pattern | `User::new().with_age(25)` |
| Type safety wrapper | Newtype pattern | `struct Meters(f64)` |
| Iterate over collection | Iterator methods | `.map()`, `.filter()` |
| State machine | Type states or enum | Compile-time safety |

### Performance Considerations

| Choice | Use When | Performance |
|--------|----------|-------------|
| `String` vs `&str` | Need ownership vs borrow | Allocation vs stack |
| `Vec` vs `[T; N]` | Dynamic vs fixed size | Heap vs stack |
| `clone()` vs borrow | Need copy vs shared access | Expensive vs cheap |
| Static dispatch | Types known at compile time | Fast, no overhead |
| Dynamic dispatch | Types unknown at compile time | Virtual call overhead |
| `Rc` vs `Arc` | Single vs multi-threaded | Cheaper vs thread-safe |
| `RefCell` vs `Mutex` | Single vs multi-threaded | Cheaper vs thread-safe |
| Iterator chains | Transforming collections | Zero-cost abstractions |
| Manual loops | Simple iteration | Often same as iterators |

---

## Cargo Commands

**Reference**: [Cargo Book - Commands](https://doc.rust-lang.org/cargo/commands/index.html)

### Project Management

```bash
# Create new project
cargo new my-project              # Binary (executable)
cargo new --lib my-library        # Library

# Initialize in existing directory
cargo init                        # Binary
cargo init --lib                  # Library

# Build
cargo build                       # Debug build (target/debug/)
cargo build --release             # Optimized build (target/release/)
cargo build -p package-name       # Build specific workspace package

# Run
cargo run                         # Build and run
cargo run --release               # Run release build
cargo run -p package-name         # Run specific package
cargo run --bin binary-name       # Run specific binary
cargo run -- arg1 arg2            # Pass arguments to program

# Check
cargo check                       # Fast compile check (no executable)
cargo check --workspace           # Check all workspace packages
cargo check --all-targets         # Check tests, benches, examples too
```

### Testing

```bash
# Run tests
cargo test                        # Run all tests
cargo test test_name              # Run specific test
cargo test --lib                  # Only library tests
cargo test --bin binary-name      # Binary tests
cargo test --test integration     # Specific integration test
cargo test --doc                  # Doc tests only
cargo test -- --nocapture         # Show println! output
cargo test -- --test-threads=1    # Run serially

# Benchmarks
cargo bench                       # Run benchmarks
```

### Documentation

```bash
# Generate and view docs
cargo doc                         # Generate documentation
cargo doc --open                  # Generate and open in browser
cargo doc --no-deps               # Don't document dependencies
cargo doc --document-private-items # Include private items

# Search standard library docs
cargo doc --open std              # Open std docs
```

### Code Quality

```bash
# Format code
cargo fmt                         # Format all code
cargo fmt --check                 # Check if formatted (CI)

# Lint
cargo clippy                      # Run linter
cargo clippy --fix                # Auto-fix issues
cargo clippy -- -D warnings       # Treat warnings as errors

# Security audit
cargo audit                       # Check for vulnerable dependencies
```

### Dependencies

```bash
# Update dependencies
cargo update                      # Update to latest compatible
cargo update -p package-name      # Update specific package

# Add dependency
cargo add serde                   # Add latest version
cargo add serde@1.0               # Add specific version
cargo add serde --features derive # With features

# Remove dependency
cargo rm serde

# Show dependency tree
cargo tree                        # Show all dependencies
cargo tree -p package-name        # For specific package
cargo tree --duplicates           # Show duplicate dependencies
```

### Workspace Commands

```bash
# Build/test all packages
cargo build --workspace
cargo test --workspace
cargo check --workspace

# Build/test specific package
cargo build -p package-name
cargo test -p package-name
```

### Publishing

```bash
# Publish to crates.io
cargo login <token>               # Authenticate
cargo publish                     # Publish crate
cargo publish --dry-run           # Test publish

# Package
cargo package                     # Create .crate file
cargo package --list              # List files to be packaged
```

### Other Useful Commands

```bash
# Clean build artifacts
cargo clean                       # Remove target/ directory

# Search crates
cargo search serde                # Search crates.io

# Show crate info
cargo info serde                  # Show crate information

# Verify project
cargo verify-project              # Check Cargo.toml

# Configuration
cargo config get                  # Show configuration
```

---

## Quick Reference Tables

### Common Methods

| Type | Method | Returns | Description |
|------|--------|---------|-------------|
| `Option<T>` | `.unwrap()` | `T` | Panics if None |
| `Option<T>` | `.unwrap_or(default)` | `T` | Returns default if None |
| `Option<T>` | `.map(f)` | `Option<U>` | Transform Some value |
| `Option<T>` | `.and_then(f)` | `Option<U>` | Chain operations |
| `Option<T>` | `.is_some()` | `bool` | Check if Some |
| `Result<T, E>` | `.unwrap()` | `T` | Panics if Err |
| `Result<T, E>` | `.expect(msg)` | `T` | Panics with message |
| `Result<T, E>` | `.unwrap_or(default)` | `T` | Returns default if Err |
| `Result<T, E>` | `.map(f)` | `Result<U, E>` | Transform Ok value |
| `Result<T, E>` | `.map_err(f)` | `Result<T, F>` | Transform Err value |
| `Vec<T>` | `.push(item)` | `()` | Add to end |
| `Vec<T>` | `.pop()` | `Option<T>` | Remove from end |
| `Vec<T>` | `.len()` | `usize` | Number of elements |
| `Vec<T>` | `.is_empty()` | `bool` | Check if empty |
| `Vec<T>` | `.get(i)` | `Option<&T>` | Safe index access |
| `String` | `.len()` | `usize` | Bytes (not chars!) |
| `String` | `.push_str(s)` | `()` | Append &str |
| `String` | `.push(c)` | `()` | Append char |
| `HashMap<K,V>` | `.insert(k, v)` | `Option<V>` | Insert/update |
| `HashMap<K,V>` | `.get(k)` | `Option<&V>` | Get value |
| `HashMap<K,V>` | `.entry(k)` | `Entry<K,V>` | For update patterns |

### Common Traits to Derive

```rust
#[derive(Debug)]           // {:?} formatting
#[derive(Clone)]           // .clone() method
#[derive(Copy)]            // Implicit copying (requires Clone)
#[derive(PartialEq)]       // == and !=
#[derive(Eq)]              // Full equality (requires PartialEq)
#[derive(PartialOrd)]      // <, >, <=, >=
#[derive(Ord)]             // Full ordering (requires PartialOrd, Eq)
#[derive(Hash)]            // For HashMap/HashSet keys
#[derive(Default)]         // ::default() method
#[derive(Serialize, Deserialize)] // serde support
```

### Common Imports

```rust
// Standard library
use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::fmt::{self, Display};

// For web services (add to Cargo.toml)
use axum::{Router, routing::get};
use tokio;
use serde::{Serialize, Deserialize};
use sqlx::{PgPool, FromRow};

// For error handling
use anyhow::{Result, Context};
use thiserror::Error;
```

---

## Learning Resources

### Official Documentation
- **The Rust Book**: https://doc.rust-lang.org/book/ - Start here!
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **Standard Library**: https://doc.rust-lang.org/std/
- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **Rustonomicon (unsafe Rust)**: https://doc.rust-lang.org/nomicon/
- **Async Book**: https://rust-lang.github.io/async-book/
- **API Guidelines**: https://rust-lang.github.io/api-guidelines/

### Interactive Learning
- **Rustlings**: https://github.com/rust-lang/rustlings
- **Rust Playground**: https://play.rust-lang.org/
- **Exercism Rust Track**: https://exercism.org/tracks/rust

### Community Resources
- **This Week in Rust**: https://this-week-in-rust.org/
- **Awesome Rust**: https://github.com/rust-unofficial/awesome-rust
- **Rust Patterns**: https://rust-unofficial.github.io/patterns/
- **Rust Design Patterns**: https://rust-unofficial.github.io/patterns/
- **r/rust**: https://reddit.com/r/rust
- **Rust Users Forum**: https://users.rust-lang.org/

### Books
- **Programming Rust** (O'Reilly)
- **Rust in Action** (Manning)
- **Zero To Production In Rust** (web services)

---

This cheatsheet covers the most common Rust patterns and idioms you'll use in daily development. Refer back to it as you build RustMart!
