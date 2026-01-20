# rinch-core

Core types and traits for rinch, including elements and reactive primitives.

## Element Types

### `Element`

The fundamental building block of rinch UI.

```rust
pub enum Element {
    Window(WindowProps, Children),
    AppMenu(AppMenuProps, Children),
    Menu(MenuProps, Children),
    MenuItem(MenuItemProps),
    MenuSeparator,
    Html(String),
    Component(Box<dyn AnyComponent>),
    Fragment(Children),
}
```

### `WindowProps`

Configuration for a window:

```rust
pub struct WindowProps {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub decorations: bool,
}
```

### `AppMenuProps`

Configuration for an application menu:

```rust
pub struct AppMenuProps {
    pub native: bool,
}
```

### `MenuProps`

Configuration for a menu:

```rust
pub struct MenuProps {
    pub label: String,
}
```

### `MenuItemProps`

Configuration for a menu item:

```rust
pub struct MenuItemProps {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub checked: Option<bool>,
}
```

## Reactive Module

### `Signal<T>`

A reactive container for mutable state.

```rust
impl<T> Signal<T> {
    pub fn new(value: T) -> Self;
    pub fn set(&self, value: T);
    pub fn update(&self, f: impl FnOnce(&mut T));
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R;
}

impl<T: Clone> Signal<T> {
    pub fn get(&self) -> T;
}
```

### `Effect`

A side-effect that tracks dependencies.

```rust
impl Effect {
    pub fn new<F: FnMut() + 'static>(f: F) -> Self;
    pub fn new_deferred<F: FnMut() + 'static>(f: F) -> Self;
    pub fn run(&self);
    pub fn dispose(&self);
}
```

### `Memo<T>`

A cached computed value.

```rust
impl<T: Clone + 'static> Memo<T> {
    pub fn new<F: Fn() -> T + 'static>(f: F) -> Self;
    pub fn get(&self) -> T;
}
```

### `Scope`

Manages the lifetime of reactive primitives.

```rust
impl Scope {
    pub fn new() -> Self;
    pub fn run<R>(&self, f: impl FnOnce() -> R) -> R;
    pub fn add_effect(&self, effect: Effect);
    pub fn dispose(&self);
}
```

## Utility Functions

### `batch`

Batch multiple signal updates:

```rust
pub fn batch<R>(f: impl FnOnce() -> R) -> R;
```

### `derived`

Create a memo (convenience function):

```rust
pub fn derived<T: Clone + 'static>(f: impl Fn() -> T + 'static) -> Memo<T>;
```

### `untracked`

Read signals without tracking:

```rust
pub fn untracked<R>(f: impl FnOnce() -> R) -> R;
```

## Event Module

### `RinchEvent`

Events processed by the rinch runtime:

```rust
pub enum RinchEvent {
    Poll { window_id: WindowId },
    MenuEvent(muda::MenuId),
}
```
