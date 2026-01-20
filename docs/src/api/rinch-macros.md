# rinch-macros

Procedural macros for rinch, primarily the `rsx!` macro.

## `rsx!`

A JSX-like macro for building UI elements.

### Basic Usage

```rust
use rinch::prelude::*;

let element = rsx! {
    div { class: "container",
        h1 { "Hello, World!" }
        p { "Welcome to rinch." }
    }
};
```

### Syntax

#### Elements

HTML elements use lowercase names:

```rust
rsx! {
    div { }
    span { }
    button { }
    input { }
}
```

Rinch components use PascalCase:

```rust
rsx! {
    Window { }
    AppMenu { }
    Menu { }
    MenuItem { }
    Fragment { }
}
```

#### Attributes

Attributes are key-value pairs before children:

```rust
rsx! {
    div { class: "container", id: "main",
        // children
    }
}
```

#### Text Content

Text is included directly:

```rust
rsx! {
    p { "Hello, World!" }
    span { "Multiple " "strings " "work" }
}
```

#### Expressions

Rust expressions in curly braces:

```rust
let name = "World";
rsx! {
    p { "Hello, " {name} "!" }
}
```

#### Event Handlers

Events use `onevent: handler` syntax:

```rust
rsx! {
    button {
        onclick: |_| println!("Clicked!"),
        "Click me"
    }
}
```

### Expansion

The macro expands to `Element` enum variants:

```rust
// This:
rsx! {
    Window { title: "App", width: 800, height: 600,
        html { body { "Content" } }
    }
}

// Expands to approximately:
Element::Window(
    WindowProps {
        title: "App".into(),
        width: 800,
        height: 600,
        decorations: true,
    },
    vec![
        Element::Html("<html><body>Content</body></html>".into())
    ]
)
```

### Component Mapping

| RSX Component | Element Variant |
|---------------|-----------------|
| `Window` | `Element::Window` |
| `AppMenu` | `Element::AppMenu` |
| `Menu` | `Element::Menu` |
| `MenuItem` | `Element::MenuItem` |
| `MenuSeparator` | `Element::MenuSeparator` |
| `Fragment` | `Element::Fragment` |
| `html`, `div`, etc. | `Element::Html` |

### HTML Generation

HTML elements are converted to HTML strings:

```rust
// This:
rsx! {
    div { class: "wrapper",
        p { "Hello" }
        span { id: "name", "World" }
    }
}

// Generates:
Element::Html(
    "<div class=\"wrapper\"><p>Hello</p><span id=\"name\">World</span></div>".into()
)
```

### Notes

- HTML elements are rendered as a single string for efficiency
- Component props use default values where not specified
- The macro is compile-time, so syntax errors appear at build time
