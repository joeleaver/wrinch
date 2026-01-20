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

A clickable menu item with optional shortcut and callback:

```rust
MenuItem { label: "Save", shortcut: "Cmd+S" }
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `label` | `&str` | Required. The menu item text. |
| `shortcut` | `&str` | Optional. Keyboard shortcut (see below). |
| `enabled` | `bool` | Optional. Whether the item is clickable. Default: `true`. |
| `checked` | `bool` | Optional. Shows a checkmark next to the item. |
| `onclick` | `Fn()` | Optional. Callback invoked when clicked or shortcut pressed. |

#### Menu Callbacks

Use `onclick` to handle menu item activation:

```rust
let count = use_signal(|| 0);
let count_reset = count.clone();

rsx! {
    AppMenu { native: true,
        Menu { label: "Edit",
            MenuItem {
                label: "Reset Counter",
                onclick: move || {
                    count_reset.set(0);
                    println!("Counter reset!");
                }
            }
        }
    }
}
```

Callbacks are triggered both when:
- The user clicks the menu item
- The user presses the keyboard shortcut

### MenuSeparator

A visual separator between menu items:

```rust
MenuSeparator {}
```

## Keyboard Shortcuts

Shortcuts are specified as strings combining modifiers and a key, separated by `+`.

### Modifiers

| Modifier | macOS | Windows/Linux |
|----------|-------|---------------|
| `Cmd` | Command (⌘) | Ctrl |
| `Ctrl` | Control (⌃) | Ctrl |
| `Alt` | Option (⌥) | Alt |
| `Shift` | Shift (⇧) | Shift |

### Supported Keys

**Letters:** `A` through `Z`

**Numbers:** `0` through `9`

**Function keys:** `F1` through `F12`

**Special keys:**
- `Enter`, `Return`
- `Escape`, `Esc`
- `Backspace`
- `Tab`
- `Space`
- `Delete`, `Del`

**Navigation:**
- `Home`, `End`
- `PageUp`, `PageDown`
- `Up`, `Down`, `Left`, `Right` (arrow keys)

**Symbols:**
- `=`, `Equal`, `Plus`
- `-`, `Minus`

### Examples

```rust
MenuItem { label: "New", shortcut: "Cmd+N" }           // Ctrl+N on Windows
MenuItem { label: "Save As", shortcut: "Cmd+Shift+S" } // Ctrl+Shift+S
MenuItem { label: "Redo", shortcut: "Cmd+Shift+Z" }
MenuItem { label: "Exit", shortcut: "Alt+F4" }
MenuItem { label: "Zoom In", shortcut: "Cmd+=" }
MenuItem { label: "Find Next", shortcut: "F3" }
```

Shortcuts work across platforms - `Cmd` is automatically mapped to `Ctrl` on Windows and Linux.

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
    // State for the application
    let file_path = use_signal(|| None::<String>);
    let show_about = use_signal(|| false);

    // Clones for menu callbacks
    let file_new = file_path.clone();
    let file_open = file_path.clone();
    let about_toggle = show_about.clone();

    rsx! {
        Fragment {
            AppMenu { native: true,
                Menu { label: "File",
                    MenuItem {
                        label: "New",
                        shortcut: "Cmd+N",
                        onclick: move || {
                            file_new.set(None);
                            println!("New file created");
                        }
                    }
                    MenuItem {
                        label: "Open...",
                        shortcut: "Cmd+O",
                        onclick: move || {
                            // In a real app, show file picker
                            file_open.set(Some("example.txt".into()));
                            println!("Opening file...");
                        }
                    }
                    MenuSeparator {}
                    MenuItem {
                        label: "Save",
                        shortcut: "Cmd+S",
                        onclick: || println!("Saving...")
                    }
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
                    MenuItem {
                        label: "About",
                        onclick: move || about_toggle.update(|v| *v = !*v)
                    }
                }
            }

            Window { title: "My App", width: 800, height: 600,
                html {
                    body {
                        h1 { "Application with Menus" }
                        p {
                            "Current file: "
                            {file_path.get().unwrap_or_else(|| "Untitled".into())}
                        }
                        // About dialog
                        div {
                            style: if show_about.get() { "display: block" } else { "display: none" },
                            h2 { "About My App" }
                            p { "Built with Rinch" }
                        }
                    }
                }
            }
        }
    }
}
```
