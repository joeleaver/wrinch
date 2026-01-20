//! Menu module - native menu support via muda.

use muda::{
    accelerator::Accelerator, Menu, MenuEvent, MenuEventReceiver, MenuItem, PredefinedMenuItem,
    Submenu,
};
use rinch_core::element::{Element, MenuItemCallback, MenuItemProps};
use std::collections::HashMap;
use std::str::FromStr;

/// Manages native menus for the application.
pub struct MenuManager {
    /// The root menu.
    menu: Option<Menu>,
    /// Map from menu item IDs to callback indices.
    item_callbacks: HashMap<muda::MenuId, usize>,
    /// Stored callbacks (indices into this vec).
    callbacks: Vec<MenuCallback>,
}

/// Stores menu item information and callback.
pub struct MenuCallback {
    pub label: String,
    /// The callback to invoke when this menu item is activated.
    pub callback: Option<MenuItemCallback>,
}

impl MenuManager {
    pub fn new() -> Self {
        Self {
            menu: None,
            item_callbacks: HashMap::new(),
            callbacks: Vec::new(),
        }
    }

    /// Build native menu from AppMenu element.
    pub fn build_from_element(&mut self, element: &Element) -> Option<&Menu> {
        let Element::AppMenu(props, children) = element else {
            return None;
        };

        // Only build native menu if native: true
        if !props.native {
            return None;
        }

        let menu = Menu::new();

        for child in children {
            if let Some(submenu) = self.build_submenu(child) {
                let _ = menu.append(&submenu);
            }
        }

        self.menu = Some(menu);
        self.menu.as_ref()
    }

    /// Build a Submenu from a Menu element.
    fn build_submenu(&mut self, element: &Element) -> Option<Submenu> {
        let Element::Menu(props, children) = element else {
            return None;
        };

        let submenu = Submenu::new(&props.label, true);

        for child in children {
            match child {
                Element::MenuItem(item_props) => {
                    let menu_item = self.build_menu_item(item_props);
                    let _ = submenu.append(&menu_item);
                }
                Element::MenuSeparator => {
                    let _ = submenu.append(&PredefinedMenuItem::separator());
                }
                Element::Menu(_, _) => {
                    // Nested submenu
                    if let Some(nested) = self.build_submenu(child) {
                        let _ = submenu.append(&nested);
                    }
                }
                _ => {}
            }
        }

        Some(submenu)
    }

    /// Build a MenuItem from MenuItemProps.
    fn build_menu_item(&mut self, props: &MenuItemProps) -> MenuItem {
        // Parse accelerator from shortcut string
        let accelerator = props
            .shortcut
            .as_ref()
            .and_then(|s| parse_shortcut(s));

        let item = MenuItem::new(&props.label, props.enabled, accelerator);

        // Store callback mapping
        let callback_idx = self.callbacks.len();
        self.callbacks.push(MenuCallback {
            label: props.label.clone(),
            callback: props.onclick.clone(),
        });
        self.item_callbacks.insert(item.id().clone(), callback_idx);

        item
    }

    /// Get the menu for platform initialization.
    pub fn menu(&self) -> Option<&Menu> {
        self.menu.as_ref()
    }

    /// Initialize menu for a window (Windows/Linux).
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    pub fn init_for_window(&self, window: &winit::window::Window) {
        use winit::raw_window_handle::HasWindowHandle;

        if let Some(menu) = &self.menu {
            #[cfg(target_os = "windows")]
            {
                if let Ok(handle) = window.window_handle() {
                    if let winit::raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
                        let hwnd = win32.hwnd.get() as isize;
                        let _ = menu.init_for_hwnd(hwnd);
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                // Linux requires GTK integration - for now skip
                // TODO: Implement GTK menu integration
            }
        }
    }

    /// Initialize menu for the application (macOS).
    #[cfg(target_os = "macos")]
    pub fn init_for_app(&self) {
        if let Some(menu) = &self.menu {
            menu.init_for_nsapp();
        }
    }

    /// Handle a menu event, invoking the callback if one exists.
    ///
    /// Returns `true` if a callback was invoked (indicating state may have changed
    /// and a re-render may be needed), `false` otherwise.
    pub fn handle_event(&self, event: &MenuEvent) -> bool {
        if let Some(&callback_idx) = self.item_callbacks.get(event.id()) {
            if let Some(stored) = self.callbacks.get(callback_idx) {
                tracing::info!("Menu item activated: {}", stored.label);
                if let Some(cb) = &stored.callback {
                    cb.invoke();
                    return true;
                }
            }
        }
        false
    }

    /// Get the label of a menu item by event, for logging purposes.
    pub fn get_label(&self, event: &MenuEvent) -> Option<&str> {
        if let Some(&callback_idx) = self.item_callbacks.get(event.id()) {
            if let Some(stored) = self.callbacks.get(callback_idx) {
                return Some(&stored.label);
            }
        }
        None
    }

    /// Get the menu event receiver for polling.
    pub fn event_receiver() -> &'static MenuEventReceiver {
        MenuEvent::receiver()
    }
}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a shortcut string like "Cmd+N" or "Ctrl+Shift+S" into an Accelerator.
fn parse_shortcut(shortcut: &str) -> Option<Accelerator> {
    // Convert common shortcuts to muda format
    // muda uses: "CmdOrCtrl+N", "Shift+CmdOrCtrl+S", etc.
    let normalized = shortcut
        .replace("Cmd+", "CmdOrCtrl+")
        .replace("Ctrl+", "CmdOrCtrl+")
        .replace("Meta+", "CmdOrCtrl+");

    Accelerator::from_str(&normalized).ok()
}
