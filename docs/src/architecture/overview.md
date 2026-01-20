# Architecture Overview

Rinch is built as a layered architecture with clear separation of concerns.

## Crate Structure

```
rinch/
├── crates/
│   ├── rinch/           # Main application crate
│   ├── rinch-core/      # Core types and reactive primitives
│   ├── rinch-macros/    # RSX proc macro
│   └── rinch-renderer/  # Rendering abstraction
└── examples/
    └── smyeditor/       # Example application
```

## Layer Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│  (your app, smyeditor, etc.)                                │
├─────────────────────────────────────────────────────────────┤
│                        rinch                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Shell     │  │   Menu      │  │   Window    │         │
│  │  (runtime)  │  │  (muda)     │  │  (winit)    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│                      rinch-core                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  Element    │  │  Reactive   │  │   Event     │         │
│  │  (RSX types)│  │  (signals)  │  │  (input)    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│                     External Crates                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   blitz     │  │   vello     │  │   winit     │         │
│  │  (HTML/CSS) │  │  (GPU 2D)   │  │ (windowing) │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

## Key Components

### rinch-core

The foundation layer containing:
- **Element types** - RSX node representations (Window, AppMenu, Html, etc.)
- **Reactive primitives** - Signal, Effect, Memo for state management
- **Event types** - Input and window events

### rinch-macros

The `rsx!` proc macro that transforms JSX-like syntax into Rust code:

```rust
// This RSX syntax:
rsx! {
    div { class: "container",
        p { "Hello" }
    }
}

// Becomes:
Element::Html("<div class=\"container\"><p>Hello</p></div>".into())
```

### rinch

The main crate that ties everything together:
- **Shell** - Application runtime, event loop integration
- **Window Manager** - Window creation and rendering
- **Menu Manager** - Native menu support via muda

### rinch-renderer

Rendering abstraction layer (currently using Vello).

## External Dependencies

| Crate | Purpose |
|-------|---------|
| blitz-dom | HTML DOM implementation |
| blitz-html | HTML parsing |
| blitz-paint | CSS painting |
| vello | GPU 2D rendering |
| wgpu | GPU abstraction |
| winit | Cross-platform windowing |
| muda | Native menu support |

## Data Flow

```
User RSX code
     │
     ▼
┌─────────────┐
│ rsx! macro  │  Compile time
└─────────────┘
     │
     ▼
┌─────────────┐
│  Element    │  Runtime
│  tree       │
└─────────────┘
     │
     ├─────────────────────────────┐
     ▼                             ▼
┌─────────────┐            ┌─────────────┐
│   Window    │            │    Menu     │
│   Manager   │            │   Manager   │
└─────────────┘            └─────────────┘
     │                             │
     ▼                             ▼
┌─────────────┐            ┌─────────────┐
│ blitz DOM   │            │    muda     │
│ + painting  │            │   (native)  │
└─────────────┘            └─────────────┘
     │
     ▼
┌─────────────┐
│   Vello     │
│  (GPU)      │
└─────────────┘
     │
     ▼
  Display
```
