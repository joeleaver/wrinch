# RSX Syntax

RSX is a JSX-like syntax for building UI in Rust. It lets you write HTML-like markup directly in your Rust code.

## Basic Syntax

```rust
use rinch::prelude::*;

fn example() -> Element {
    rsx! {
        div {
            h1 { "Hello, World!" }
            p { "This is a paragraph." }
        }
    }
}
```

## HTML Elements

Standard HTML elements are written in lowercase:

```rust
rsx! {
    div {
        span { "Text content" }
        button { "Click me" }
        input { type: "text", placeholder: "Enter text..." }
    }
}
```

## Attributes

Attributes are specified after the element name:

```rust
rsx! {
    div { class: "container", id: "main",
        a { href: "https://example.com", "Link text" }
        img { src: "image.png", alt: "Description" }
    }
}
```

## Rinch Components

Rinch-specific components are written in PascalCase:

```rust
rsx! {
    Window { title: "My App", width: 800, height: 600,
        // window content
    }

    AppMenu { native: true,
        Menu { label: "File",
            MenuItem { label: "New", shortcut: "Cmd+N" }
        }
    }
}
```

## Fragments

Use `Fragment` to group multiple elements without a wrapper:

```rust
rsx! {
    Fragment {
        Window { /* ... */ }
        Window { /* ... */ }
    }
}
```

## Text Content

Text can be included directly in elements:

```rust
rsx! {
    p { "This is text content" }
    span { "Multiple " "strings " "concatenated" }
}
```

## Expressions

Rust expressions can be embedded in curly braces:

```rust
let name = "World";
let count = 42;

rsx! {
    p { "Hello, " {name} "!" }
    p { "Count: " {count} }
}
```

## Event Handlers

Events use the `onevent: handler` syntax:

```rust
let count = Signal::new(0);

rsx! {
    button {
        onclick: move |_| count.update(|n| *n += 1),
        "Increment"
    }
}
```

## Styling

Inline styles and CSS classes work like regular HTML:

```rust
rsx! {
    html {
        head {
            style {
                "
                .container { padding: 20px; }
                .highlight { color: red; }
                "
            }
        }
        body {
            div { class: "container",
                span { class: "highlight", "Styled text" }
            }
        }
    }
}
```
