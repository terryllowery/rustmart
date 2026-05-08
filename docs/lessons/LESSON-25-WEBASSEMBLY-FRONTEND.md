# Lesson 25: WebAssembly Frontend

## Overview
Build a high-performance web frontend for RustMart using Rust compiled to WebAssembly with Yew framework, demonstrating full-stack Rust development.

## Why This Matters
WebAssembly (Wasm) enables:
- **Performance** - Near-native speed in the browser
- **Type Safety** - Rust's type system prevents entire classes of bugs
- **Code Sharing** - Share types and logic between frontend/backend
- **Small Bundle Size** - Optimized Wasm binaries (with proper tooling)

Companies using Wasm: Figma (performance-critical rendering), Cloudflare Workers, Google Earth.

## Setting Up Yew Project

### Install Tools

```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Install trunk (build tool)
cargo install trunk

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli
```

### Create Project

```bash
cargo new rustmart-frontend
cd rustmart-frontend

# Add dependencies to Cargo.toml
```

**Cargo.toml**:
```toml
[package]
name = "rustmart-frontend"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-router = "0.18"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = "0.3"
js-sys = "0.3"
gloo-net = "0.5"
uuid = { version = "1.0", features = ["serde", "js"] }

[profile.release]
lto = true
opt-level = 'z'
codegen-units = 1
```

### Project Structure

```
rustmart-frontend/
├── Cargo.toml
├── index.html
├── src/
│   ├── main.rs
│   ├── components/
│   │   ├── mod.rs
│   │   ├── product_list.rs
│   │   ├── product_card.rs
│   │   └── nav.rs
│   ├── api/
│   │   ├── mod.rs
│   │   └── products.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   └── home.rs
│   └── types.rs
└── Trunk.toml
```

## Basic Yew Application

### index.html

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>RustMart</title>
    <link data-trunk rel="css" href="styles.css" />
  </head>
  <body></body>
</html>
```

### Main Component (src/main.rs)

```rust
use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod routes;
mod api;
mod types;

use routes::{Route, switch};

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <div class="app">
                <components::Nav />
                <main>
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```

### Routing (src/routes/mod.rs)

```rust
use yew::prelude::*;
use yew_router::prelude::*;

pub mod home;
pub mod product_detail;
pub mod cart;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/products/:id")]
    ProductDetail { id: String },
    #[at("/cart")]
    Cart,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <home::Home /> },
        Route::ProductDetail { id } => html! { <product_detail::ProductDetail {id} /> },
        Route::Cart => html! { <cart::Cart /> },
        Route::NotFound => html! { <h1>{"404 - Page Not Found"}</h1> },
    }
}
```

## Shared Types

### src/types.rs

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub inventory_count: i32,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub price: f64,
}
```

## API Client

### src/api/products.rs

```rust
use crate::types::Product;
use gloo_net::http::Request;
use uuid::Uuid;

const API_BASE: &str = "http://localhost:8000/api";

pub async fn fetch_products() -> Result<Vec<Product>, String> {
    let url = format!("{}/products", API_BASE);
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<Product>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_product(id: Uuid) -> Result<Product, String> {
    let url = format!("{}/products/{}", API_BASE, id);
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Product>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_product(product: &Product) -> Result<Product, String> {
    let url = format!("{}/products", API_BASE);
    
    Request::post(&url)
        .json(product)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Product>()
        .await
        .map_err(|e| e.to_string())
}
```

## Components

### Product List (src/components/product_list.rs)

```rust
use yew::prelude::*;
use crate::{types::Product, api};
use crate::components::ProductCard;

#[function_component(ProductList)]
pub fn product_list() -> Html {
    let products = use_state(|| Vec::<Product>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    
    {
        let products = products.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::products::fetch_products().await {
                    Ok(data) => {
                        products.set(data);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }
    
    if *loading {
        return html! {
            <div class="loading">{"Loading products..."}</div>
        };
    }
    
    if let Some(err) = (*error).as_ref() {
        return html! {
            <div class="error">{format!("Error: {}", err)}</div>
        };
    }
    
    html! {
        <div class="product-grid">
            { for products.iter().map(|product| {
                html! { <ProductCard product={product.clone()} /> }
            })}
        </div>
    }
}
```

### Product Card (src/components/product_card.rs)

```rust
use yew::prelude::*;
use yew_router::prelude::*;
use crate::types::Product;
use crate::routes::Route;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub product: Product,
}

#[function_component(ProductCard)]
pub fn product_card(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let product = &props.product;
    
    let onclick = {
        let navigator = navigator.clone();
        let id = product.id.to_string();
        Callback::from(move |_| {
            navigator.push(&Route::ProductDetail { id: id.clone() });
        })
    };
    
    html! {
        <div class="product-card" {onclick}>
            if let Some(img) = &product.image_url {
                <img src={img.clone()} alt={product.name.clone()} />
            } else {
                <div class="placeholder-image">{"No Image"}</div>
            }
            <h3>{&product.name}</h3>
            <p class="description">{&product.description}</p>
            <div class="price-stock">
                <span class="price">{format!("${:.2}", product.price)}</span>
                <span class="stock">
                    {if product.inventory_count > 0 {
                        format!("{} in stock", product.inventory_count)
                    } else {
                        "Out of stock".to_string()
                    }}
                </span>
            </div>
        </div>
    }
}
```

## State Management with use_reducer

### Shopping Cart State

```rust
use std::collections::HashMap;
use uuid::Uuid;
use yew::prelude::*;
use crate::types::CartItem;

#[derive(Debug, Clone, PartialEq)]
pub struct CartState {
    pub items: HashMap<Uuid, CartItem>,
}

impl Default for CartState {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}

pub enum CartAction {
    AddItem(CartItem),
    RemoveItem(Uuid),
    UpdateQuantity(Uuid, i32),
    Clear,
}

impl Reducible for CartState {
    type Action = CartAction;
    
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new_state = (*self).clone();
        
        match action {
            CartAction::AddItem(item) => {
                if let Some(existing) = new_state.items.get_mut(&item.product_id) {
                    existing.quantity += item.quantity;
                } else {
                    new_state.items.insert(item.product_id, item);
                }
            }
            CartAction::RemoveItem(id) => {
                new_state.items.remove(&id);
            }
            CartAction::UpdateQuantity(id, qty) => {
                if let Some(item) = new_state.items.get_mut(&id) {
                    item.quantity = qty;
                }
            }
            CartAction::Clear => {
                new_state.items.clear();
            }
        }
        
        Rc::new(new_state)
    }
}

// Context Provider
#[derive(Properties, PartialEq)]
pub struct CartProviderProps {
    pub children: Children,
}

#[function_component(CartProvider)]
pub fn cart_provider(props: &CartProviderProps) -> Html {
    let cart = use_reducer(CartState::default);
    
    html! {
        <ContextProvider<UseReducerHandle<CartState>> context={cart}>
            {props.children.clone()}
        </ContextProvider<UseReducerHandle<CartState>>>
    }
}
```

### Using Cart Context

```rust
#[function_component(ProductDetail)]
pub fn product_detail(props: &Props) -> Html {
    let cart = use_context::<UseReducerHandle<CartState>>().unwrap();
    let product = use_state(|| None::<Product>);
    
    // Fetch product...
    
    let add_to_cart = {
        let cart = cart.clone();
        let product = product.clone();
        
        Callback::from(move |_| {
            if let Some(p) = (*product).as_ref() {
                cart.dispatch(CartAction::AddItem(CartItem {
                    product_id: p.id,
                    product_name: p.name.clone(),
                    quantity: 1,
                    price: p.price,
                }));
            }
        })
    };
    
    html! {
        <div class="product-detail">
            <button onclick={add_to_cart}>{"Add to Cart"}</button>
        </div>
    }
}
```

## Forms and Validation

### Product Form

```rust
use web_sys::HtmlInputElement;

#[function_component(ProductForm)]
pub fn product_form() -> Html {
    let name = use_state(|| String::new());
    let price = use_state(|| String::new());
    let description = use_state(|| String::new());
    let error = use_state(|| None::<String>);
    
    let onsubmit = {
        let name = name.clone();
        let price = price.clone();
        let description = description.clone();
        let error = error.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let name_val = (*name).clone();
            let price_val = (*price).clone();
            let desc_val = (*description).clone();
            
            // Validate
            if name_val.is_empty() {
                error.set(Some("Name is required".to_string()));
                return;
            }
            
            let price_num = match price_val.parse::<f64>() {
                Ok(p) if p > 0.0 => p,
                _ => {
                    error.set(Some("Price must be a positive number".to_string()));
                    return;
                }
            };
            
            // Submit
            let product = Product {
                id: Uuid::new_v4(),
                name: name_val,
                description: desc_val,
                price: price_num,
                inventory_count: 0,
                image_url: None,
            };
            
            wasm_bindgen_futures::spawn_local(async move {
                match api::products::create_product(&product).await {
                    Ok(_) => {
                        // Success - navigate or show message
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
            });
        })
    };
    
    let oninput_name = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            name.set(input.value());
        })
    };
    
    let oninput_price = {
        let price = price.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            price.set(input.value());
        })
    };
    
    html! {
        <form {onsubmit} class="product-form">
            {if let Some(err) = (*error).as_ref() {
                html! { <div class="error">{err}</div> }
            } else {
                html! {}
            }}
            
            <div class="form-group">
                <label>{"Product Name"}</label>
                <input
                    type="text"
                    value={(*name).clone()}
                    oninput={oninput_name}
                    required=true
                />
            </div>
            
            <div class="form-group">
                <label>{"Price"}</label>
                <input
                    type="number"
                    step="0.01"
                    value={(*price).clone()}
                    oninput={oninput_price}
                    required=true
                />
            </div>
            
            <button type="submit">{"Create Product"}</button>
        </form>
    }
}
```

## Building and Optimization

### Trunk.toml

```toml
[build]
target = "index.html"
release = true

[watch]
ignore = ["target", "dist"]

[serve]
address = "127.0.0.1"
port = 8080

[[proxy]]
backend = "http://localhost:8000/api"
rewrite = "/api"
```

### Build Commands

```bash
# Development build with hot reload
trunk serve

# Production build
trunk build --release

# Optimize with wasm-opt
wasm-opt dist/*.wasm -O3 -o dist/optimized.wasm
```

### Bundle Size Optimization

```toml
# In Cargo.toml
[profile.release]
lto = true                  # Link-time optimization
opt-level = 'z'            # Optimize for size
codegen-units = 1          # Better optimization
strip = true               # Strip symbols
panic = 'abort'            # Smaller panic handler
```

## Testing

### Component Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use yew::platform::pinned::oneshot;
    
    #[test]
    fn test_product_card_renders() {
        let product = Product {
            id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            description: "Test".to_string(),
            price: 9.99,
            inventory_count: 10,
            image_url: None,
        };
        
        let props = Props { product };
        let html = yew::ServerRenderer::<ProductCard>::with_props(|| props)
            .render()
            .await;
        
        assert!(html.contains("Test Product"));
        assert!(html.contains("$9.99"));
    }
}
```

## Deployment

### GitHub Pages

```bash
# Build for production
trunk build --release --public-url /rustmart-frontend/

# Deploy dist/ folder to gh-pages branch
```

### Netlify/Vercel

**netlify.toml**:
```toml
[build]
  command = "trunk build --release"
  publish = "dist"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

## Best Practices

- **Keep Wasm Bundle Small** - Use `opt-level='z'`, enable LTO
- **Lazy Load Routes** - Split bundles by route
- **Cache API Responses** - Use `use_memo` for expensive computations
- **Handle Loading States** - Always show loading/error UI
- **Type Safety** - Share types between frontend/backend
- **Test Components** - Unit test component logic
- **Optimize Re-renders** - Use `PartialEq` on Props

## Official Documentation

- [Yew Documentation](https://yew.rs/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [Trunk Documentation](https://trunkrs.dev/)
- [WebAssembly.org](https://webassembly.org/)
