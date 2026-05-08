# Lesson 25: WebAssembly Frontend (Outline)

## Overview
Build a high-performance web frontend for RustMart using Rust compiled to WebAssembly with Yew framework, demonstrating full-stack Rust development.

## Core Topics

### 1. WebAssembly Fundamentals
- What is WebAssembly (Wasm)?
- Wasm vs JavaScript performance
- Browser support and compatibility
- Security model (sandboxed execution)
- Use cases for Wasm in web development

### 2. Rust to WebAssembly Toolchain

#### wasm-pack
- Build Rust for WebAssembly
- Generate JavaScript bindings
- npm package generation
- Optimization flags

#### wasm-bindgen
- JavaScript interop from Rust
- Import JS functions into Rust
- Export Rust functions to JS
- Type conversions between Rust and JS

#### web-sys & js-sys
- Browser APIs in Rust
- DOM manipulation
- Fetch API, WebSockets
- Console, timers, storage

### 3. Yew Framework

#### Component Model
```rust
use yew::prelude::*;

#[function_component(ProductList)]
fn product_list() -> Html {
    let products = use_state(|| vec![]);
    
    html! {
        <div class="product-list">
            { for products.iter().map(|p| html! {
                <ProductCard product={p.clone()} />
            })}
        </div>
    }
}
```

#### Component Lifecycle
- Hooks (use_state, use_effect, use_reducer)
- Props and callbacks
- Component communication
- Lifecycle methods

#### Routing
- yew-router for SPA navigation
- Route definitions
- Route parameters
- Navigation guards

### 4. State Management

#### Local State with Hooks
- use_state for component state
- use_reducer for complex state
- use_context for global state

#### State Management Libraries
- yewdux (Redux-like for Yew)
- Bounce (reactive state management)
- Global state patterns

### 5. API Integration

#### Fetch API with Reqwest
```rust
use reqwest;

async fn fetch_products() -> Result<Vec<Product>, Error> {
    let response = reqwest::get("https://api.rustmart.com/products")
        .await?
        .json::<Vec<Product>>()
        .await?;
    
    Ok(response)
}
```

#### Error Handling
- Result types in async contexts
- User-friendly error messages
- Retry logic
- Loading states

### 6. Styling & UI

#### CSS Integration
- Inline styles
- CSS modules
- Tailwind CSS with Trunk
- Styled components

#### UI Component Libraries
- yew-material
- PatternFly Yew
- Custom component library

### 7. Forms & Validation

#### Form Handling
```rust
#[function_component(ProductForm)]
fn product_form() -> Html {
    let name = use_state(|| String::new());
    let price = use_state(|| 0.0);
    
    let onsubmit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        // Submit form
    });
    
    html! {
        <form {onsubmit}>
            <input 
                type="text" 
                value={(*name).clone()}
                onchange={/* update name */}
            />
            <button type="submit">{"Submit"}</button>
        </form>
    }
}
```

#### Validation
- Client-side validation
- Custom validators
- Error display patterns

### 8. Performance Optimization

#### Code Splitting
- Lazy loading components
- Route-based splitting
- Dynamic imports

#### Wasm Binary Optimization
- `wasm-opt` for size reduction
- `wee_alloc` for smaller allocator
- Strip debug symbols
- Link-time optimization (LTO)

#### Bundle Size Analysis
- wasm-pack size profiling
- twiggy for code analysis
- Minimizing dependencies

### 9. Build & Development Tools

#### Trunk
- Development server with hot reload
- Asset pipeline (CSS, images)
- Build optimization
- Proxy for API calls during development

**Trunk.toml**:
```toml
[build]
target = "index.html"

[watch]
ignore = ["target", "dist"]

[serve]
address = "127.0.0.1"
port = 8080

[[proxy]]
backend = "http://localhost:8000/api"
```

### 10. Testing Wasm Frontend

#### Unit Tests
- wasm-bindgen-test
- Testing components in isolation
- Mocking API calls

#### Integration Tests
- wasm-pack test in headless browsers
- Cypress or Playwright for E2E tests

### 11. Deployment

#### Static Hosting
- Deploy to Netlify, Vercel, Cloudflare Pages
- GitHub Pages deployment
- CDN configuration

#### CI/CD Pipeline
```yaml
# Build Wasm frontend
- name: Build frontend
  run: |
    cargo install trunk
    trunk build --release
    
- name: Deploy
  uses: peaceiris/actions-gh-pages@v3
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    publish_dir: ./dist
```

### 12. Advanced Topics

#### Web Workers with Wasm
- Offload heavy computation
- Parallel processing
- SharedArrayBuffer

#### WebGL with Wasm
- 3D product visualization
- High-performance graphics

#### Progressive Web App (PWA)
- Service workers
- Offline support
- App manifest

## Tools & Libraries

- **Yew**: Frontend framework (React-like)
- **wasm-pack**: Rust to Wasm build tool
- **Trunk**: Build tool and dev server
- **yew-router**: SPA routing
- **yewdux**: State management
- **reqwest**: HTTP client
- **web-sys**: Browser APIs
- **wasm-bindgen**: JS interop

## Hands-on Exercises

1. Create "Hello World" Yew app
2. Build product listing page with API integration
3. Implement shopping cart with state management
4. Add routing for multi-page app
5. Create product form with validation
6. Optimize bundle size and measure performance
7. Deploy to Netlify/Vercel

## Best Practices

- Keep Wasm bundle size small (<500KB)
- Use code splitting for large apps
- Minimize JS interop overhead
- Cache API responses when appropriate
- Use Rust's type safety for correctness
- Profile and optimize hot paths
- Provide loading states for async operations
- Handle errors gracefully with user feedback

## Resources

- [Yew Documentation](https://yew.rs/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [Trunk Documentation](https://trunkrs.dev/)
- [WebAssembly.org](https://webassembly.org/)
