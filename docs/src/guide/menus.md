# Menus

Rinch provides native menu support through the `muda` library. You can create both platform-native menus and HTML-based menus.

## Native Menus

Use `AppMenu` with `native: true` for platform-native menus:

```rust
use rinch::prelude::*;

fn app() -> Element {
    rsx! {
        Fragment {
            AppMenu { native: true,
                Menu { label: "File",
                    MenuItem { label: "New", shortcut: "Cmd+N" }
                    MenuItem { label: "Open...", shortcut: "Cmd+O" }
                    MenuSeparator {}
                    MenuItem { label: "Save", shortcut: "Cmd+S" }
                    MenuSeparator {}
                    MenuItem { label: "Exit", shortcut: "Alt+F4" }
                }
                Menu { label: "Edit",
                    MenuItem { label: "Undo", shortcut: "Cmd+Z" }
                    MenuItem { label: "Redo", shortcut: "Cmd+Shift+Z" }
                    MenuSeparator {}
                    MenuItem { label: "Cut", shortcut: "Cmd+X" }
                    MenuItem { label: "Copy", shortcut: "Cmd+C" }
                    MenuItem { label: "Paste", shortcut: "Cmd+V" }
                }
            }
            Window { /* ... */ }
        }
    }
}
```

## Menu Components

### AppMenu

The root menu container. Set `native: true` for OS-native menus.

```rust
AppMenu { native: true,
    // Menu children
}
```

### Menu

A dropdown menu with a label:

```rust
Menu { label: "File",
    // MenuItem children
}
```

### MenuItem

A clickable menu item:

```rust
MenuItem { label: "Save", shortcut: "Cmd+S" }
```

### MenuSeparator

A visual separator between menu items:

```rust
MenuSeparator {}
```

## Keyboard Shortcuts

Shortcuts use a cross-platform format:

| Modifier | macOS | Windows/Linux |
|----------|-------|---------------|
| `Cmd` | Command (⌘) | Ctrl |
| `Ctrl` | Control (⌃) | Ctrl |
| `Alt` | Option (⌥) | Alt |
| `Shift` | Shift (⇧) | Shift |

Examples:
- `Cmd+S` → Save (Ctrl+S on Windows)
- `Cmd+Shift+Z` → Redo
- `Alt+F4` → Exit

## Platform Behavior

### macOS

On macOS, the menu appears in the system menu bar at the top of the screen, following Apple's Human Interface Guidelines.

### Windows

On Windows, the menu appears attached to the window's title bar.

### Linux

On Linux, the menu appears in the window (similar to Windows) unless a global menu system is available.

## Complete Example

```rust
use rinch::prelude::*;

fn app() -> Element {
    rsx! {
        Fragment {
            AppMenu { native: true,
                Menu { label: "File",
                    MenuItem { label: "New", shortcut: "Cmd+N" }
                    MenuItem { label: "Open...", shortcut: "Cmd+O" }
                    MenuSeparator {}
                    MenuItem { label: "Save", shortcut: "Cmd+S" }
                    MenuItem { label: "Save As...", shortcut: "Cmd+Shift+S" }
                    MenuSeparator {}
                    MenuItem { label: "Exit", shortcut: "Alt+F4" }
                }
                Menu { label: "Edit",
                    MenuItem { label: "Undo", shortcut: "Cmd+Z" }
                    MenuItem { label: "Redo", shortcut: "Cmd+Shift+Z" }
                    MenuSeparator {}
                    MenuItem { label: "Cut", shortcut: "Cmd+X" }
                    MenuItem { label: "Copy", shortcut: "Cmd+C" }
                    MenuItem { label: "Paste", shortcut: "Cmd+V" }
                    MenuSeparator {}
                    MenuItem { label: "Select All", shortcut: "Cmd+A" }
                }
                Menu { label: "View",
                    MenuItem { label: "Zoom In", shortcut: "Cmd+=" }
                    MenuItem { label: "Zoom Out", shortcut: "Cmd+-" }
                    MenuItem { label: "Reset Zoom", shortcut: "Cmd+0" }
                }
                Menu { label: "Help",
                    MenuItem { label: "Documentation" }
                    MenuItem { label: "About" }
                }
            }

            Window { title: "My App", width: 800, height: 600,
                html {
                    body {
                        h1 { "Application with Menus" }
                    }
                }
            }
        }
    }
}
```
