# Windows

Rinch supports multi-window applications from the ground up. Each window is declared using the `Window` component in your RSX.

## Basic Window

```rust
use rinch::prelude::*;

fn app() -> Element {
    rsx! {
        Window { title: "My Application", width: 800, height: 600,
            html {
                body {
                    h1 { "Window Content" }
                }
            }
        }
    }
}
```

## Window Properties

| Property | Type | Description |
|----------|------|-------------|
| `title` | `&str` | Window title bar text |
| `width` | `u32` | Initial window width in pixels |
| `height` | `u32` | Initial window height in pixels |
| `decorations` | `bool` | Show window decorations (default: true) |

## Multiple Windows

Create multiple windows by including multiple `Window` elements:

```rust
rsx! {
    Fragment {
        Window { title: "Main Window", width: 800, height: 600,
            // Main window content
        }
        Window { title: "Secondary Window", width: 400, height: 300,
            // Secondary window content
        }
    }
}
```

## Borderless Windows

Set `decorations: false` for a borderless window:

```rust
rsx! {
    Window { title: "Borderless", width: 800, height: 600, decorations: false,
        html {
            body {
                // Custom title bar implementation
                div { class: "custom-titlebar",
                    "My Custom Title Bar"
                }
            }
        }
    }
}
```

## Window Content

Windows contain HTML content rendered by the blitz engine. The content is specified using standard HTML elements:

```rust
rsx! {
    Window { title: "Styled Window", width: 800, height: 600,
        html {
            head {
                style {
                    "
                    body {
                        font-family: system-ui;
                        background: #1e1e1e;
                        color: white;
                    }
                    "
                }
            }
            body {
                main {
                    h1 { "Welcome" }
                    p { "This content is styled with CSS." }
                }
            }
        }
    }
}
```

## GPU-Accelerated Rendering

All windows are rendered using Vello, a GPU-accelerated 2D graphics library. This provides:

- Smooth animations
- High-quality text rendering
- Efficient repaints
- Cross-platform consistency
