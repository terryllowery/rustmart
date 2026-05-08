# Lesson 18: Rust FFI - Integrating C Libraries

## Overview
Learn Foreign Function Interface (FFI) to integrate C code with Rust, opening up decades of C libraries and allowing you to write high-performance systems code. This lesson bridges your Rust microservices knowledge with your C learning from lowlevel.academy.

By the end of this lesson, you'll have:
- Understanding of Rust's FFI capabilities
- Ability to call C functions from Rust
- Ability to expose Rust functions to C
- Safe wrappers around unsafe FFI code
- Real-world integration examples for RustMart

## Why FFI Matters

**For your career:**
- Most system libraries are written in C (OpenSSL, libpq, etc.)
- Differentiates you from typical web developers
- Shows systems programming depth (Band 10 material)
- Leverages your C knowledge from lowlevel.academy
- Real-world: SQLx, tokio, and most Rust libs use FFI under the hood

**Use cases:**
- Using existing C libraries (no rewrite needed)
- Performance-critical code in C
- Interfacing with hardware/kernel
- Gradual migration from C to Rust
- Cross-language microservices

## FFI Safety

```rust
// FFI is ALWAYS unsafe in Rust
unsafe {
    let result = c_function(ptr);
}
```

**Why?** Rust can't verify:
- C pointer validity
- C memory management
- C data race freedom
- C type correctness

**Your job:** Create safe Rust wrappers around unsafe FFI.

## Step 1: Calling C from Rust - Simple Example

Create `ffi-examples/simple/`:

```bash
mkdir -p ffi-examples/simple
cd ffi-examples/simple
```

Create `hello.c`:

```c
#include <stdio.h>

void say_hello(const char* name) {
    printf("Hello from C, %s!\n", name);
}

int add(int a, int b) {
    return a + b;
}

typedef struct {
    int x;
    int y;
} Point;

Point create_point(int x, int y) {
    Point p = {x, y};
    return p;
}
```

Compile C library:

```bash
gcc -c hello.c -o hello.o
ar rcs libhello.a hello.o
```

Create Rust project:

```bash
cargo init --name ffi-simple
```

Create `build.rs`:

```rust
fn main() {
    println!("cargo:rustc-link-search=native=.");
    println!("cargo:rustc-link-lib=static=hello");
}
```

Create `src/main.rs`:

```rust
use std::ffi::CString;
use std::os::raw::c_char;

// Declare C functions
extern "C" {
    fn say_hello(name: *const c_char);
    fn add(a: i32, b: i32) -> i32;
}

// Declare C struct
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

extern "C" {
    fn create_point(x: i32, y: i32) -> Point;
}

// Safe Rust wrapper
fn safe_say_hello(name: &str) {
    // Convert Rust string to C string
    let c_name = CString::new(name).expect("CString conversion failed");
    
    unsafe {
        say_hello(c_name.as_ptr());
    }
}

fn safe_add(a: i32, b: i32) -> i32 {
    unsafe { add(a, b) }
}

fn main() {
    safe_say_hello("Rust Developer");
    
    let result = safe_add(5, 3);
    println!("5 + 3 = {}", result);
    
    let point = unsafe { create_point(10, 20) };
    println!("Point: ({}, {})", point.x, point.y);
}
```

Build and run:

```bash
cargo build
cargo run
```

**Key concepts:**
- `extern "C"`: Declares C functions
- `#[repr(C)]`: Use C memory layout for structs
- `CString`: Rust string → C string conversion
- `unsafe`: All FFI calls must be in unsafe blocks

## Step 2: Using bindgen for Automatic Bindings

Instead of manually declaring C functions, use `bindgen` to auto-generate bindings.

Install bindgen:

```bash
cargo install bindgen-cli
```

Create `wrapper.h`:

```c
#include "hello.h"
```

Generate bindings:

```bash
bindgen wrapper.h -o src/bindings.rs
```

Use generated bindings:

```rust
mod bindings;

use bindings::*;
use std::ffi::CString;

fn main() {
    let name = CString::new("Auto-generated").unwrap();
    
    unsafe {
        say_hello(name.as_ptr());
    }
}
```

## Step 3: Real-World Example - JSON Parser in C

Let's integrate a fast C JSON parser into RustMart.

Create `ffi-examples/json-parser/json.c`:

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Simplified JSON parser (in reality, use cJSON or similar)
typedef struct {
    char* name;
    double price;
    int inventory_count;
} Product;

// Parse JSON string into Product
int parse_product_json(const char* json, Product* product) {
    // Simplified parsing (real code would use proper JSON parser)
    char name_buf[256];
    
    int result = sscanf(json, 
        "{\"name\":\"%[^\"]\",\"price\":%lf,\"inventory_count\":%d}",
        name_buf,
        &product->price,
        &product->inventory_count
    );
    
    if (result == 3) {
        product->name = strdup(name_buf);
        return 0;  // Success
    }
    
    return -1;  // Error
}

// Free product memory
void free_product(Product* product) {
    if (product->name) {
        free(product->name);
        product->name = NULL;
    }
}
```

Rust wrapper `src/json_parser.rs`:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
struct CProduct {
    name: *mut c_char,
    price: f64,
    inventory_count: i32,
}

extern "C" {
    fn parse_product_json(json: *const c_char, product: *mut CProduct) -> i32;
    fn free_product(product: *mut CProduct);
}

pub struct Product {
    pub name: String,
    pub price: f64,
    pub inventory_count: i32,
}

pub fn parse_json(json: &str) -> Result<Product, String> {
    let c_json = CString::new(json).map_err(|e| e.to_string())?;
    let mut c_product = CProduct {
        name: std::ptr::null_mut(),
        price: 0.0,
        inventory_count: 0,
    };
    
    let result = unsafe {
        parse_product_json(c_json.as_ptr(), &mut c_product as *mut CProduct)
    };
    
    if result != 0 {
        return Err("Failed to parse JSON".to_string());
    }
    
    // Convert C strings to Rust strings
    let name = unsafe {
        CStr::from_ptr(c_product.name)
            .to_string_lossy()
            .into_owned()
    };
    
    let product = Product {
        name,
        price: c_product.price,
        inventory_count: c_product.inventory_count,
    };
    
    // Clean up C memory
    unsafe {
        free_product(&mut c_product as *mut CProduct);
    }
    
    Ok(product)
}
```

## Step 4: Exposing Rust to C

Create a Rust library that C code can call:

Create `lib.rs`:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn rust_validate_price(price: f64) -> bool {
    price >= 0.0 && price <= 1000000.0
}

#[no_mangle]
pub extern "C" fn rust_format_currency(amount: f64) -> *mut c_char {
    let formatted = format!("${:.2}", amount);
    
    // Allocate C string (caller must free!)
    match CString::new(formatted) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rust_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            // Take ownership and drop
            let _ = CString::from_raw(ptr);
        }
    }
}

// Complex type - pass struct from Rust to C
#[repr(C)]
pub struct RustProduct {
    pub id: u64,
    pub price: f64,
    pub in_stock: bool,
}

#[no_mangle]
pub extern "C" fn rust_create_product(id: u64, price: f64, in_stock: bool) -> RustProduct {
    RustProduct { id, price, in_stock }
}
```

Build as C library:

```toml
[lib]
crate-type = ["cdylib"]  # C dynamic library
```

Generate C header:

```bash
cbindgen --output rustmart.h
```

C code calling Rust:

```c
#include "rustmart.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Call Rust validation
    bool valid = rust_validate_price(99.99);
    printf("Price valid: %d\n", valid);
    
    // Call Rust string formatting
    char* formatted = rust_format_currency(1234.56);
    printf("Formatted: %s\n", formatted);
    rust_free_string(formatted);  // Must free!
    
    // Use Rust struct
    RustProduct product = rust_create_product(1, 49.99, true);
    printf("Product ID: %lu, Price: %.2f\n", product.id, product.price);
    
    return 0;
}
```

Compile:

```bash
cargo build --release
gcc main.c -L target/release -lrustmart -o main
./main
```

## Step 5: Real Integration - Custom Allocator

Build a memory allocator in C for Rust (advanced):

`allocator.c`:

```c
#include <stdlib.h>
#include <stddef.h>

void* custom_alloc(size_t size) {
    void* ptr = malloc(size);
    // Could add tracking, debugging, etc.
    return ptr;
}

void custom_free(void* ptr, size_t size) {
    // Could verify size, track deallocations, etc.
    free(ptr);
}
```

`src/allocator.rs`:

```rust
use std::alloc::{GlobalAlloc, Layout};
use std::os::raw::c_void;

extern "C" {
    fn custom_alloc(size: usize) -> *mut c_void;
    fn custom_free(ptr: *mut c_void, size: usize);
}

struct CustomAllocator;

unsafe impl GlobalAlloc for CustomAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        custom_alloc(layout.size()) as *mut u8
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        custom_free(ptr as *mut c_void, layout.size());
    }
}

#[global_allocator]
static ALLOCATOR: CustomAllocator = CustomAllocator;
```

## Step 6: FFI Best Practices

### 1. Always Create Safe Wrappers

```rust
// ❌ BAD: Expose unsafe FFI directly
pub fn add(a: i32, b: i32) -> i32 {
    unsafe { c_add(a, b) }
}

// ✅ GOOD: Validate inputs, handle errors
pub fn add(a: i32, b: i32) -> Result<i32, Error> {
    if a == i32::MAX || b == i32::MAX {
        return Err(Error::Overflow);
    }
    
    let result = unsafe { c_add(a, b) };
    Ok(result)
}
```

### 2. Handle String Conversions Carefully

```rust
// Convert Rust → C
fn rust_to_c_string(s: &str) -> Result<CString, NulError> {
    CString::new(s)  // Fails if string contains null byte
}

// Convert C → Rust
fn c_to_rust_string(ptr: *const c_char) -> Result<String, Utf8Error> {
    if ptr.is_null() {
        return Err(...);
    }
    
    let c_str = unsafe { CStr::from_ptr(ptr) };
    Ok(c_str.to_str()?.to_owned())
}
```

### 3. Manage Memory Ownership

```rust
// Rule: Whoever allocates, deallocates
// If C allocates, C must free
// If Rust allocates, Rust must free

extern "C" {
    fn c_malloc_string() -> *mut c_char;
    fn c_free_string(ptr: *mut c_char);
}

fn use_c_string() {
    let c_str = unsafe { c_malloc_string() };
    
    // Use the string...
    
    // Must free with C's free function!
    unsafe { c_free_string(c_str); }
}
```

### 4. Use repr(C) for Structs Across FFI

```rust
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}  // Guaranteed C-compatible layout

#[repr(Rust)]  // Default - can reorder fields!
struct NotSafe {
    x: f64,
    y: f64,
}  // ❌ Don't use across FFI
```

## Step 7: RustMart Integration - High-Performance JSON

Add a C-based JSON parser to RustMart for better performance:

`Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...

[build-dependencies]
cc = "1.0"
```

`build.rs`:

```rust
fn main() {
    cc::Build::new()
        .file("src/c/json_parser.c")
        .compile("json_parser");
}
```

Use in product-service:

```rust
mod json_ffi;

use json_ffi::parse_product_json_fast;

async fn create_product_fast(
    Json(json_str): Json<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Use C parser for speed
    let product = parse_product_json_fast(&json_str)
        .map_err(|e| ApiError::BadRequest(e))?;
    
    // Rest of the logic...
}
```

## Common FFI Patterns

| Pattern | Use Case |
|---------|----------|
| Opaque pointers | Hide C struct details from Rust |
| Callbacks | C code calls Rust functions |
| Error codes | C functions return int status |
| Context/user_data | Pass Rust closures to C |
| Thread safety | Mark types with Send/Sync carefully |

## Debugging FFI

```bash
# Check symbols in library
nm -D libmylib.so

# Check if symbol is mangled
c++filt _ZN3foo3barE

# Use gdb to debug across FFI boundary
gdb ./my_program
(gdb) break c_function
(gdb) run
```

## Performance Considerations

```rust
// ❌ Slow: Convert on every call
fn call_many_times() {
    for i in 0..1000 {
        let c_str = CString::new("hello").unwrap();
        unsafe { c_function(c_str.as_ptr()); }
    }
}

// ✅ Fast: Convert once
fn call_many_times_fast() {
    let c_str = CString::new("hello").unwrap();
    let ptr = c_str.as_ptr();
    
    for i in 0..1000 {
        unsafe { c_function(ptr); }
    }
}
```

## Key Takeaways

1. **FFI is unsafe** - Always create safe wrappers
2. **Memory management** - Be explicit about ownership
3. **String conversions** - CString/CStr for C strings
4. **repr(C)** - Required for struct compatibility
5. **bindgen** - Auto-generate bindings from headers
6. **Performance** - Minimize conversions across boundary

## Challenges

1. **Wrap zlib**: Create safe Rust bindings for zlib compression
2. **Call SQLite directly**: Bypass SQLx and call libsqlite3
3. **Create plugin system**: Load C plugins dynamically (dlopen)
4. **Build a C extension**: Write performance-critical code in C
5. **Cross-language debugging**: Debug through Rust→C boundary

## Next Steps

In **Lesson 19**, you'll learn performance profiling to identify bottlenecks and optimize both Rust and C code!

## Official Documentation

- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
- [bindgen](https://rust-lang.github.io/rust-bindgen/)
- [cbindgen](https://github.com/eqrion/cbindgen)
- [cc crate](https://docs.rs/cc/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - Unsafe Rust bible

---

**This is advanced material that sets you apart!** FFI knowledge combined with systems programming makes you invaluable for low-level infrastructure work at IBM. 🔥
