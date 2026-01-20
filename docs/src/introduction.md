# Rinch

Rinch is a lightweight, cross-platform GUI library for Rust that combines the power of web technologies with native performance.

## Philosophy

- **Declarative UI** - Define your UI as a function of state using RSX syntax
- **Fine-grained Reactivity** - Only update what changed, not the entire UI
- **Web Standards** - Use HTML/CSS for layout, familiar to web developers
- **Native Performance** - GPU-accelerated rendering via Vello, native menus via muda
- **Cross-platform** - Windows, macOS, and Linux from a single codebase

## Quick Example

```rust
use rinch::prelude::*;

fn app() -> Element {
    let count = Signal::new(0);

    rsx! {
        Window { title: "Counter",
            button { onclick: move |_| count.set(count.get() + 1),
                "Count: " {count}
            }
        }
    }
}

fn main() {
    rinch::run(app);
}
```

## Features

- **RSX Macro** - JSX-like syntax for building UI
- **Reactive Signals** - Automatic UI updates when state changes
- **Native Menus** - Platform-native menu bars via muda
- **HTML/CSS Rendering** - Full CSS support via Stylo (Firefox's engine)
- **GPU Rendering** - Fast 2D rendering via Vello and wgpu

## Architecture

Rinch is built on top of several excellent Rust crates:

- [blitz](https://github.com/DioxusLabs/blitz) - HTML/CSS rendering engine
- [vello](https://github.com/linebender/vello) - GPU 2D rendering
- [winit](https://github.com/rust-windowing/winit) - Cross-platform windowing
- [muda](https://github.com/tauri-apps/muda) - Native menu support
