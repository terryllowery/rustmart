# Lesson 4: Error Handling in Rust

Understanding Rust's approach to error handling with `Result`, `Option`, and error types.

## Official Documentation References

- **Rust Book - Error Handling**: https://doc.rust-lang.org/book/ch09-00-error-handling.html
- **Rust Book - Recoverable Errors**: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
- **Rust Book - To panic! or Not to panic!**: https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html
- **thiserror crate**: https://docs.rs/thiserror/
- **anyhow crate**: https://docs.rs/anyhow/
- **Error Handling Survey**: https://blog.burntsushi.net/rust-error-handling/

---

## The Philosophy

**Rust has no exceptions!** Instead, errors are values that you must handle explicitly.

**Two types of errors:**
1. **Recoverable** - Use `Result<T, E>` (file not found, parse failed)
2. **Unrecoverable** - Use `panic!` (broken invariants, bugs)

**Key principle:** Errors are part of your function's contract. If a function can fail, its signature shows it.

---

## Option<T> - When a Value Might Not Exist

**Reference**: [Rust Book - Option](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values)

### The Problem with Null

In many languages:
```javascript
// JavaScript
let user = findUser(id);
user.name; // Might crash if user is null!
```

**Rust's solution:** Make "might not exist" explicit with `Option<T>`.

### Option Definition

```rust
enum Option<T> {
    Some(T),    // Has a value
    None,       // No value
}
```

### Using Option

```rust
// Function that might not find something
fn find_user(id: u32) -> Option<User> {
    if id == 1 {
        Some(User { name: "Alice".to_string() })
    } else {
        None  // Not found
    }
}

// Using the result
match find_user(1) {
    Some(user) => println!("Found: {}", user.name),
    None => println!("User not found"),
}
```

### Option Methods (Idiomatic)

```rust
let maybe_user = find_user(1);

// ✅ Check if Some
if maybe_user.is_some() { }
if maybe_user.is_none() { }

// ✅ Get value or default
let user = maybe_user.unwrap_or(default_user);
let user = maybe_user.unwrap_or_else(|| create_default());

// ✅ Transform Some value (map)
let maybe_name = maybe_user.map(|user| user.name);
// If Some(user) -> Some(user.name)
// If None -> None

// ✅ Chain operations (and_then / flat_map)
let maybe_email = maybe_user
    .and_then(|user| user.email);  // Returns Option<String>

// ✅ Filter
let adult = maybe_user.filter(|user| user.age >= 18);

// ✅ if let (single pattern match)
if let Some(user) = maybe_user {
    println!("User: {}", user.name);
}

// ❌ Avoid unwrap in production (panics if None!)
let user = maybe_user.unwrap();

// ✅ Better: unwrap with message
let user = maybe_user.expect("User must exist at this point");
```

### Common Patterns

```rust
// ✅ Early return with ?
fn get_user_email(id: u32) -> Option<String> {
    let user = find_user(id)?;  // Returns None if find_user returns None
    Some(user.email)
}

// ✅ Combining Options
let result = option1
    .and_then(|x| option2.map(|y| x + y));

// ✅ Convert Option to Result
let result: Result<User, String> = maybe_user
    .ok_or("User not found".to_string());
```

---

## Result<T, E> - When Operations Can Fail

**Reference**: [Rust Book - Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)

### Result Definition

```rust
enum Result<T, E> {
    Ok(T),      // Success with value T
    Err(E),     // Failure with error E
}
```

### Basic Usage

```rust
use std::fs::File;
use std::io::Error;

// Function that can fail
fn open_file(path: &str) -> Result<File, Error> {
    File::open(path)  // Returns Result
}

// Using the result
match open_file("data.txt") {
    Ok(file) => println!("File opened: {:?}", file),
    Err(error) => println!("Failed to open file: {}", error),
}
```

### Result Methods

```rust
let result = divide(10, 2);

// ✅ Check success/failure
if result.is_ok() { }
if result.is_err() { }

// ✅ Get value or default
let value = result.unwrap_or(0);
let value = result.unwrap_or_else(|err| {
    println!("Error: {}", err);
    0
});

// ✅ Transform Ok value (map)
let doubled = result.map(|n| n * 2);
// Ok(5) -> Ok(10)
// Err(e) -> Err(e)

// ✅ Transform Err value (map_err)
let result = result.map_err(|e| format!("Error: {}", e));

// ✅ Chain operations (and_then)
let result = divide(10, 2)
    .and_then(|n| divide(n, 2));  // Returns Result

// ✅ if let Ok (single pattern)
if let Ok(value) = result {
    println!("Success: {}", value);
}

// ❌ Avoid unwrap (panics if Err!)
let value = result.unwrap();

// ✅ Better: expect with message
let value = result.expect("Division should not fail");
```

---

## The ? Operator - Error Propagation

**Reference**: [Rust Book - ? Operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator)

The `?` operator is the **idiomatic** way to handle errors in Rust.

### Without ? Operator (Verbose)

```rust
fn read_username_from_file() -> Result<String, std::io::Error> {
    let file = match File::open("username.txt") {
        Ok(f) => f,
        Err(e) => return Err(e),  // Early return on error
    };
    
    let mut username = String::new();
    match file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}
```

### With ? Operator (Idiomatic!)

```rust
fn read_username_from_file() -> Result<String, std::io::Error> {
    let mut file = File::open("username.txt")?;  // Returns Err if fails
    let mut username = String::new();
    file.read_to_string(&mut username)?;  // Returns Err if fails
    Ok(username)
}
```

**What `?` does:**
- If `Ok(value)` → unwraps and continues
- If `Err(error)` → **returns early** with the error

### ? with Option

```rust
fn get_first_char(text: &str) -> Option<char> {
    let first = text.chars().next()?;  // Returns None if no chars
    Some(first.to_uppercase().next()?)
}
```

### ? Rules

1. ✅ Can only use in functions returning `Result` or `Option`
2. ✅ Error types must be compatible (or convertible)
3. ✅ Makes error handling concise and readable

```rust
// ❌ Can't use ? in functions returning ()
fn main() {
    let file = File::open("data.txt")?;  // Error: main returns ()
}

// ✅ Can use in functions returning Result
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("data.txt")?;  // OK
    Ok(())
}
```

---

## Custom Error Types

### Simple Custom Error (Enum)

```rust
#[derive(Debug)]
enum MathError {
    DivisionByZero,
    NegativeNumber,
}

fn divide(x: i32, y: i32) -> Result<i32, MathError> {
    if y == 0 {
        return Err(MathError::DivisionByZero);
    }
    Ok(x / y)
}

fn sqrt(x: i32) -> Result<i32, MathError> {
    if x < 0 {
        return Err(MathError::NegativeNumber);
    }
    Ok((x as f64).sqrt() as i32)
}
```

### Using Custom Errors

```rust
match divide(10, 0) {
    Ok(result) => println!("Result: {}", result),
    Err(MathError::DivisionByZero) => println!("Cannot divide by zero!"),
    Err(MathError::NegativeNumber) => println!("Negative number!"),
}
```

---

## Error Trait

**Reference**: [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html)

For errors to work with `?` operator and be displayed properly, they should implement `Error` trait:

```rust
use std::fmt;
use std::error::Error;

#[derive(Debug)]
enum MyError {
    NotFound(String),
    InvalidInput(String),
}

// Implement Display (required)
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::NotFound(msg) => write!(f, "Not found: {}", msg),
            MyError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

// Implement Error trait
impl Error for MyError {}
```

**This is boilerplate!** That's where `thiserror` helps...

---

## thiserror - Easy Custom Errors

**Reference**: [thiserror docs](https://docs.rs/thiserror/)

`thiserror` is a crate that makes custom errors trivial:

### Add to Cargo.toml

```toml
[dependencies]
thiserror = "1.0"
```

### Define Errors (Simple!)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {field} must be {requirement}")]
    InvalidInput {
        field: String,
        requirement: String,
    },
    
    #[error("Database error")]
    Database(#[from] sqlx::Error),  // Auto-converts from sqlx::Error
    
    #[error("IO error")]
    Io(#[from] std::io::Error),  // Auto-converts from io::Error
}
```

**What you get:**
- ✅ `Display` implementation (from `#[error(...)]` strings)
- ✅ `Error` trait implementation
- ✅ `From` conversions with `#[from]`
- ✅ Works with `?` operator

### Using thiserror Errors

```rust
fn load_config() -> Result<Config, MyError> {
    // This works! io::Error auto-converts to MyError::Io
    let contents = std::fs::read_to_string("config.toml")?;
    
    // Manual error
    if contents.is_empty() {
        return Err(MyError::InvalidInput {
            field: "config".to_string(),
            requirement: "non-empty".to_string(),
        });
    }
    
    Ok(parse_config(&contents))
}
```

---

## anyhow - Easy Error Handling in Applications

**Reference**: [anyhow docs](https://docs.rs/anyhow/)

`anyhow` is for **applications** (not libraries). It provides a catch-all error type.

### When to Use What

| Use Case | Use | Why |
|----------|-----|-----|
| **Library** | `thiserror` | Users need to handle specific errors |
| **Application** | `anyhow` | You just need to propagate and display errors |
| **Binary** | `anyhow` | Simplifies main() and error handling |

### Add to Cargo.toml

```toml
[dependencies]
anyhow = "1.0"
```

### Using anyhow

```rust
use anyhow::{Result, Context};

fn load_config() -> Result<Config> {  // Result from anyhow
    let contents = std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?;  // Add context
    
    let config = toml::from_str(&contents)
        .context("Failed to parse config")?;
    
    Ok(config)
}

fn main() -> Result<()> {  // anyhow::Result
    let config = load_config()?;
    run_app(config)?;
    Ok(())
}
```

**Benefits:**
- ✅ Works with any error type
- ✅ `.context()` adds helpful messages
- ✅ Great for prototyping
- ✅ Easy to use in `main()`

**Drawback:**
- ❌ Loses specific error type information
- ❌ Can't match on specific error variants
- ❌ Not suitable for libraries

---

## Error Handling Patterns

### Pattern 1: Early Returns

```rust
fn process_user(id: u32) -> Result<(), MyError> {
    let user = find_user(id)?;  // Return early if not found
    
    if user.age < 18 {
        return Err(MyError::InvalidInput {
            field: "age".to_string(),
            requirement: "must be 18+".to_string(),
        });
    }
    
    save_user(&user)?;  // Return early if save fails
    Ok(())
}
```

### Pattern 2: Match for Different Handling

```rust
match load_config() {
    Ok(config) => run_app(config),
    Err(MyError::NotFound(_)) => {
        println!("Creating default config...");
        create_default_config()
    },
    Err(e) => {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}
```

### Pattern 3: Collect Results

```rust
// Process multiple items, stop on first error
fn process_all(ids: Vec<u32>) -> Result<Vec<User>, MyError> {
    ids.into_iter()
        .map(|id| find_user(id))
        .collect()  // Collects Result<Vec<_>, _>
}

// Or handle each error individually
fn process_all_lenient(ids: Vec<u32>) -> Vec<Result<User, MyError>> {
    ids.into_iter()
        .map(|id| find_user(id))
        .collect()
}
```

### Pattern 4: Combining Multiple Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    
    #[error("HTTP error")]
    Http(#[from] reqwest::Error),
    
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

// Now all three error types convert automatically!
fn complex_operation() -> Result<(), AppError> {
    let data = std::fs::read_to_string("data.json")?;  // io::Error -> AppError
    let parsed = serde_json::from_str(&data)?;  // serde error -> ?
    let response = reqwest::get("https://api.example.com").await?;  // reqwest::Error -> AppError
    let saved = save_to_db(&parsed).await?;  // sqlx::Error -> AppError
    Ok(())
}
```

---

## panic! - Unrecoverable Errors

**Reference**: [Rust Book - panic!](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html)

### When to panic!

Use `panic!` for:
- ✅ Bugs / broken invariants
- ✅ Conditions that "should never happen"
- ✅ Prototyping / learning
- ✅ Test failures

**Don't use for:**
- ❌ Expected errors (file not found, invalid input)
- ❌ Recoverable conditions
- ❌ Library code (let caller decide)

### panic! Examples

```rust
// Direct panic
panic!("Something went terribly wrong!");

// Assert (panics if false)
assert!(x > 0, "x must be positive, got {}", x);
assert_eq!(x, y, "x and y must be equal");
assert_ne!(x, y, "x and y must differ");

// Unwrap (panics if None/Err) - use sparingly!
let value = some_option.unwrap();  // Panics if None
let value = some_result.unwrap();  // Panics if Err

// Expect (like unwrap, but with message)
let value = some_option.expect("This should never be None");
```

### unreachable! and unimplemented!

```rust
// Mark code that should never execute
match value {
    Some(x) => println!("{}", x),
    None => unreachable!("We checked this earlier!"),
}

// Mark code you haven't written yet
fn future_feature() {
    unimplemented!("Will implement this later");
}

// Mark placeholder code
fn todo_implement_this() {
    todo!("Need to implement this");
}
```

---

## Best Practices

### ✅ Do

1. **Use Result for recoverable errors**
   ```rust
   fn load_file(path: &str) -> Result<String, IoError>
   ```

2. **Use ? operator for propagation**
   ```rust
   let data = read_file()?;
   ```

3. **Use thiserror for library errors**
   ```rust
   #[derive(Error, Debug)]
   enum MyLibError { ... }
   ```

4. **Use anyhow for application errors**
   ```rust
   fn main() -> anyhow::Result<()> { ... }
   ```

5. **Add context to errors**
   ```rust
   .context("Failed to load config")?
   ```

6. **Match when you need different handling**
   ```rust
   match result {
       Ok(v) => handle_success(v),
       Err(MyError::NotFound) => create_default(),
       Err(e) => return Err(e),
   }
   ```

### ❌ Don't

1. **Don't use unwrap in production**
   ```rust
   let x = result.unwrap();  // Bad! Panics on error
   ```

2. **Don't ignore errors**
   ```rust
   let _ = might_fail();  // Bad! Silent failure
   ```

3. **Don't panic for expected errors**
   ```rust
   if file_not_found {
       panic!("File not found");  // Bad! Use Result
   }
   ```

4. **Don't use String as error type**
   ```rust
   Result<T, String>  // Bad! No type safety
   ```

5. **Don't mix anyhow in libraries**
   ```rust
   // Library crate - Bad!
   pub fn my_lib_fn() -> anyhow::Result<T>
   
   // Library crate - Good!
   pub fn my_lib_fn() -> Result<T, MyLibError>
   ```

---

## Decision Tree

```
Can the operation fail?
├─ No → Don't use Result
└─ Yes → Use Result
    │
    ├─ Is this a library?
    │  ├─ Yes → Use thiserror for custom errors
    │  └─ No → Use anyhow for easy error handling
    │
    ├─ Can you handle the error here?
    │  ├─ Yes → Use match or if let
    │  └─ No → Use ? to propagate
    │
    └─ Is this a bug/invariant violation?
       ├─ Yes → panic! or assert!
       └─ No → Use Result
```

---

## Quick Reference

### Option<T>

| Method | Returns | Use When |
|--------|---------|----------|
| `.is_some()` | `bool` | Check if has value |
| `.is_none()` | `bool` | Check if no value |
| `.unwrap()` | `T` | Get value (panics if None) |
| `.expect(msg)` | `T` | Get value with message |
| `.unwrap_or(default)` | `T` | Get value or default |
| `.map(f)` | `Option<U>` | Transform Some value |
| `.and_then(f)` | `Option<U>` | Chain operations |
| `.ok_or(err)` | `Result<T, E>` | Convert to Result |
| `?` | Early return | Propagate None |

### Result<T, E>

| Method | Returns | Use When |
|--------|---------|----------|
| `.is_ok()` | `bool` | Check if success |
| `.is_err()` | `bool` | Check if error |
| `.unwrap()` | `T` | Get value (panics if Err) |
| `.expect(msg)` | `T` | Get value with message |
| `.unwrap_or(default)` | `T` | Get value or default |
| `.map(f)` | `Result<U, E>` | Transform Ok value |
| `.map_err(f)` | `Result<T, F>` | Transform Err value |
| `.and_then(f)` | `Result<U, E>` | Chain operations |
| `?` | Early return | Propagate error |

---

## For RustMart

We'll use:
- **thiserror** for shared error types (library)
- **Custom errors** for each service
- **?** operator everywhere
- **anyhow** in main() for services

Example structure:
```rust
// shared/src/error.rs
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

// product-service/src/main.rs
fn main() -> anyhow::Result<()> {
    // ... easy error handling
}
```

---

## Summary

✅ **No exceptions** - Errors are values (Result, Option)
✅ **Explicit handling** - Compiler forces you to handle errors
✅ **? operator** - Idiomatic error propagation
✅ **thiserror** - Easy custom errors for libraries
✅ **anyhow** - Easy error handling for applications
✅ **panic!** - Only for unrecoverable errors

**Key takeaway:** Rust makes error handling explicit and type-safe. It's more verbose initially, but prevents entire classes of bugs!

## Next Steps

Now let's apply this to build the shared library's error types!
