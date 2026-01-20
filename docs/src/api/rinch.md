# rinch

The main rinch crate provides the entry point and re-exports commonly used types.

## Entry Point

### `rinch::run`

Runs a rinch application with the given root component.

```rust
use rinch::prelude::*;

fn app() -> Element {
    rsx! {
        Window { title: "My App", width: 800, height: 600,
            html { body { "Hello!" } }
        }
    }
}

fn main() {
    rinch::run(app);
}
```

## Prelude

Import commonly used types with the prelude:

```rust
use rinch::prelude::*;
```

This includes:
- `Element` - RSX node type
- `Signal`, `Effect`, `Memo` - Reactive primitives
- `batch`, `derived`, `untracked` - Reactive utilities
- `rsx!` - RSX macro
- Element types: `WindowProps`, `MenuProps`, etc.

## Re-exports

### Element Types

```rust
pub use rinch_core::element::{
    AppMenuProps,
    Children,
    Element,
    MenuItemProps,
    MenuProps,
    WindowProps,
};
```

### Reactive Primitives

```rust
pub use rinch_core::{
    batch,
    derived,
    untracked,
    Effect,
    Memo,
    Scope,
    Signal,
};
```

### Macros

```rust
pub use rinch_macros::rsx;
```

### Sub-crates

```rust
pub use rinch_core as core;
pub use rinch_renderer as renderer;
```

## Modules

### `rinch::shell`

Application runtime and event loop:
- `Runtime` - Main application runtime
- `run()` - Entry point function

### `rinch::menu`

Menu management:
- `MenuManager` - Builds native menus from Elements

### `rinch::window`

Window utilities (currently minimal, window management is in shell).

### `rinch::app`

Application-level types (reserved for future use).
