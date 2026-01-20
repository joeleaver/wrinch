# Rinch

A lightweight cross-platform GUI library for Rust, built on top of [blitz](https://github.com/DioxusLabs/blitz).

Rinch provides a reactive GUI framework using HTML/CSS for layout with a Vello-based GPU renderer.

## Features

- **Declarative UI** - React-style component model with hooks API
- **HTML/CSS Rendering** - Full HTML/CSS support via Stylo and Taffy
- **GPU Accelerated** - Fast 2D rendering via Vello and wgpu
- **Native Menus** - Cross-platform menu support via muda
- **DevTools** - Built-in developer tools for debugging

## Quick Start

```rust
use rinch::prelude::*;

fn app() -> Element {
    let count = use_signal(|| 0);
    let count_inc = count.clone();

    rsx! {
        Window { title: "Counter", width: 400, height: 300,
            div {
                h1 { "Count: " {count.get()} }
                button { onclick: move || count_inc.update(|n| *n += 1),
                    "Increment"
                }
            }
        }
    }
}

fn main() {
    rinch::run(app);
}
```

## Documentation

- [**Getting Started Guide**](https://joeleaver.github.io/wrinch/guide/getting-started.html)
- [**API Reference**](https://joeleaver.github.io/wrinch/api/rinch/)

## Development Setup

```bash
# Clone the repository
git clone git@github.com:joeleaver/wrinch.git
cd wrinch

# Install git hooks (validates docs on commit)
./scripts/setup-hooks.sh

# Build
cargo build

# Run the example editor
cargo run -p smyeditor

# Build documentation locally
cargo doc --open
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `F12` | Toggle DevTools window |
| `Alt+D` | Toggle layout debug overlay |
| `Alt+I` | Toggle inspect mode |
| `Alt+T` | Print Taffy layout tree |
| `Ctrl/Cmd + +/-/0` | Zoom in/out/reset |

## License

MIT
