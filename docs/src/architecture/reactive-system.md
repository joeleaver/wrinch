# Reactive System Architecture

Rinch uses a **fine-grained reactivity** model inspired by Solid.js and Leptos. This document describes the architecture and design decisions.

## Core Concepts

### Signals

A **Signal** is a reactive container that holds a value and notifies subscribers when it changes.

```rust
let count = Signal::new(0);

// Read the value
let value = count.get();

// Update the value (triggers subscribers)
count.set(5);

// Update based on current value
count.update(|n| *n += 1);
```

**Key properties:**
- Reading a signal inside an effect automatically subscribes to it
- Setting a signal schedules dependent effects to re-run
- Signals are `Clone` and can be shared across closures

### Effects

An **Effect** is a side-effect that re-runs when its dependencies change.

```rust
let count = Signal::new(0);

Effect::new(move || {
    println!("Count is now: {}", count.get());
});

count.set(1); // Prints: "Count is now: 1"
count.set(2); // Prints: "Count is now: 2"
```

**Key properties:**
- Dependencies are tracked automatically (no dependency arrays)
- Effects run immediately when created, then re-run when dependencies change
- Effects are cleaned up when their scope is disposed

### Memos

A **Memo** is a cached computed value that only recomputes when dependencies change.

```rust
let count = Signal::new(2);
let doubled = Memo::new(move || count.get() * 2);

doubled.get(); // Returns 4
count.set(3);
doubled.get(); // Returns 6 (recomputed)
doubled.get(); // Returns 6 (cached)
```

**Key properties:**
- Lazily evaluated (only computes when read)
- Caches the result until dependencies change
- Can be read inside effects (creates a subscription)

## Dependency Tracking

Rinch uses **automatic dependency tracking** at runtime:

1. When an effect runs, it registers itself as the "current observer"
2. When a signal is read, it checks for a current observer and subscribes it
3. When a signal changes, it notifies all subscribers to re-run

```
┌─────────────┐     read      ┌─────────────┐
│   Effect    │ ───────────── │   Signal    │
│             │               │             │
│  observer   │ ◄──subscribe──│ subscribers │
└─────────────┘               └─────────────┘
                                    │
                                    │ set()
                                    ▼
                              notify observers
```

## Runtime Architecture

```
┌────────────────────────────────────────────────────┐
│                    Runtime                          │
├────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐               │
│  │ Observer     │  │ Pending      │               │
│  │ Stack        │  │ Effects      │               │
│  └──────────────┘  └──────────────┘               │
│                                                    │
│  ┌──────────────────────────────────────────────┐ │
│  │              Signal Storage                   │ │
│  │  ┌────────┐ ┌────────┐ ┌────────┐           │ │
│  │  │Signal 1│ │Signal 2│ │Signal 3│  ...      │ │
│  │  └────────┘ └────────┘ └────────┘           │ │
│  └──────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────┘
```

### Observer Stack

The runtime maintains a stack of "current observers":
- When an effect starts, it pushes itself onto the stack
- When a signal is read, it subscribes the top of the stack
- When an effect ends, it pops itself

This allows nested effects to work correctly.

### Batching

Multiple signal updates can be batched to avoid redundant effect runs:

```rust
batch(|| {
    count.set(1);
    name.set("Alice");
    // Effects only run once, after the batch
});
```

### Scheduling

Effects are scheduled to run after the current synchronous code completes:
1. Signal is set → effect is marked as "dirty"
2. Dirty effects are queued for execution
3. After current execution, queued effects run
4. This prevents infinite loops and ensures consistent state

## Memory Management

### Scopes

A **Scope** manages the lifetime of reactive primitives:

```rust
let scope = Scope::new();

scope.run(|| {
    let signal = Signal::new(0);
    Effect::new(|| { /* ... */ });
    // signal and effect belong to this scope
});

scope.dispose(); // Cleans up signal and effect
```

### Ownership

- Signals are reference-counted (`Rc<RefCell<T>>`)
- Effects hold strong references to their closures
- Disposing a scope drops all its primitives

## Integration with UI

The reactive system integrates with the rendering pipeline:

1. **Component functions** run inside a scope
2. **RSX expressions** can include signal reads: `{count.get()}`
3. **When signals change**, affected parts of the DOM are updated
4. **Fine-grained updates** - only changed nodes are re-rendered

```rust
fn counter() -> Element {
    let count = Signal::new(0);

    rsx! {
        div {
            // This text updates when count changes
            "Count: " {count.get()}

            button { onclick: move |_| count.update(|n| *n += 1),
                "Increment"
            }
        }
    }
}
```

## Thread Safety

The current implementation uses `Rc<RefCell<T>>` for single-threaded use. For multi-threaded scenarios, we provide:

- `Signal::new_sync()` - Uses `Arc<RwLock<T>>`
- Thread-local runtime by default
- Optional shared runtime for async contexts

## Comparison with Other Systems

| Feature | Rinch | React | Solid.js | Leptos |
|---------|-------|-------|----------|--------|
| Reactivity | Fine-grained | Coarse (VDOM) | Fine-grained | Fine-grained |
| Tracking | Automatic | Manual (deps array) | Automatic | Automatic |
| Scheduling | Batched | Batched | Synchronous | Batched |
| Memory | Scoped | GC | Scoped | Scoped |
