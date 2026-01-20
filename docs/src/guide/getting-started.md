# Getting Started

This guide will walk you through creating your first rinch application.

## Prerequisites

- Rust 1.75 or later
- A C++ compiler (for native dependencies)

## Create a New Project

```bash
cargo new my-app
cd my-app
```

## Add Dependencies

Add rinch to your `Cargo.toml`:

```toml
[dependencies]
rinch = { path = "../path/to/rinch" }
```

## Write Your First App

Replace the contents of `src/main.rs`:

```rust
use rinch::prelude::*;

fn app() -> Element {
    rsx! {
        Window { title: "My First Rinch App", width: 800, height: 600,
            html {
                body {
                    h1 { "Hello, Rinch!" }
                    p { "Welcome to your first rinch application." }
                }
            }
        }
    }
}

fn main() {
    rinch::run(app);
}
```

## Run Your App

```bash
cargo run
```

You should see a window appear with your content rendered inside.

## What's Next?

- Learn about [RSX Syntax](./rsx-syntax.md) for building UI
- Explore [Windows](./windows.md) for multi-window support
- Add [Menus](./menus.md) to your application
- Understand [Reactivity](./reactivity.md) for dynamic state
