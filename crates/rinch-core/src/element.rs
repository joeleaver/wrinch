//! Element types and component traits.

use std::any::Any;
use std::rc::Rc;

/// A node in the UI tree.
pub enum Element {
    /// A window element - creates a native OS window.
    Window(WindowProps, Children),
    /// Application menu - renders as native or HTML.
    AppMenu(AppMenuProps, Children),
    /// A menu within AppMenu.
    Menu(MenuProps, Children),
    /// A clickable menu item.
    MenuItem(MenuItemProps),
    /// A separator line in menus.
    MenuSeparator,
    /// Raw HTML content to be rendered by blitz.
    Html(String),
    /// A user-defined component.
    Component(Box<dyn AnyComponent>),
    /// A fragment containing multiple children.
    Fragment(Children),
}

pub type Children = Vec<Element>;

/// Properties for the Window component.
#[derive(Debug, Clone)]
pub struct WindowProps {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub borderless: bool,
    pub resizable: bool,
    pub transparent: bool,
    pub always_on_top: bool,
    pub visible: bool,
}

impl Default for WindowProps {
    fn default() -> Self {
        Self {
            title: String::from("Rinch Window"),
            width: 800,
            height: 600,
            x: None,
            y: None,
            borderless: false,
            resizable: true,
            transparent: false,
            always_on_top: false,
            visible: true,
        }
    }
}

/// Properties for the AppMenu component.
#[derive(Debug, Clone)]
pub struct AppMenuProps {
    /// If true, render as native OS menu. If false, render as HTML.
    pub native: bool,
}

impl Default for AppMenuProps {
    fn default() -> Self {
        Self { native: true }
    }
}

/// Properties for a Menu (dropdown) within AppMenu.
#[derive(Debug, Clone)]
pub struct MenuProps {
    pub label: String,
}

/// Callback type for menu items.
///
/// Uses `Rc` for `Clone` support, allowing callbacks to be stored and invoked.
#[derive(Clone)]
pub struct MenuItemCallback(pub Rc<dyn Fn()>);

impl MenuItemCallback {
    /// Create a new menu item callback from a function.
    pub fn new<F: Fn() + 'static>(f: F) -> Self {
        Self(Rc::new(f))
    }

    /// Invoke the callback.
    pub fn invoke(&self) {
        (self.0)()
    }
}

impl std::fmt::Debug for MenuItemCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MenuItemCallback(...)")
    }
}

/// Properties for a MenuItem.
#[derive(Debug, Clone)]
pub struct MenuItemProps {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub checked: Option<bool>,
    /// Callback to invoke when the menu item is activated.
    pub onclick: Option<MenuItemCallback>,
}

impl Default for MenuItemProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            enabled: true,
            checked: None,
            onclick: None,
        }
    }
}

/// Trait for user-defined components.
pub trait Component: 'static {
    type Props;

    fn render(&self, props: &Self::Props) -> Element;
}

/// Type-erased component for storage in Element tree.
pub trait AnyComponent: Any {
    fn render_any(&self) -> Element;
    fn as_any(&self) -> &dyn Any;
}
