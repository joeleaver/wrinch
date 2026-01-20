//! Property schemas for rinch components.
//!
//! Defines which properties are valid for each component type,
//! enabling compile-time validation and helpful error messages.

/// A property schema describing a component's valid properties.
#[derive(Debug, Clone, Copy)]
pub struct PropSchema {
    /// The property name.
    pub name: &'static str,
    /// Whether this property is required.
    pub required: bool,
}

impl PropSchema {
    const fn new(name: &'static str, required: bool) -> Self {
        Self { name, required }
    }

    const fn optional(name: &'static str) -> Self {
        Self::new(name, false)
    }

    const fn required(name: &'static str) -> Self {
        Self::new(name, true)
    }
}

/// Window component properties.
static WINDOW_PROPS: &[PropSchema] = &[
    PropSchema::optional("title"),
    PropSchema::optional("width"),
    PropSchema::optional("height"),
    PropSchema::optional("x"),
    PropSchema::optional("y"),
    PropSchema::optional("borderless"),
    PropSchema::optional("resizable"),
    PropSchema::optional("transparent"),
    PropSchema::optional("always_on_top"),
    PropSchema::optional("visible"),
];

/// AppMenu component properties.
static APP_MENU_PROPS: &[PropSchema] = &[PropSchema::optional("native")];

/// Menu component properties.
static MENU_PROPS: &[PropSchema] = &[PropSchema::required("label")];

/// MenuItem component properties.
static MENU_ITEM_PROPS: &[PropSchema] = &[
    PropSchema::required("label"),
    PropSchema::optional("shortcut"),
    PropSchema::optional("enabled"),
    PropSchema::optional("checked"),
    PropSchema::optional("onclick"),
];

/// Get valid property names for a component.
pub fn get_valid_props(component: &str) -> Option<&'static [PropSchema]> {
    match component {
        "Window" => Some(WINDOW_PROPS),
        "AppMenu" => Some(APP_MENU_PROPS),
        "Menu" => Some(MENU_PROPS),
        "MenuItem" => Some(MENU_ITEM_PROPS),
        _ => None,
    }
}

/// Get the list of required property names for a component.
pub fn get_required_props(component: &str) -> Vec<&'static str> {
    get_valid_props(component)
        .map(|props| {
            props
                .iter()
                .filter(|p| p.required)
                .map(|p| p.name)
                .collect()
        })
        .unwrap_or_default()
}

/// Check if a property name is valid for a component.
pub fn is_valid_prop(component: &str, prop_name: &str) -> bool {
    get_valid_props(component)
        .map(|props| props.iter().any(|p| p.name == prop_name))
        .unwrap_or(true) // Unknown components allow any props
}

/// Get all valid property names for a component.
pub fn get_prop_names(component: &str) -> Vec<&'static str> {
    get_valid_props(component)
        .map(|props| props.iter().map(|p| p.name).collect())
        .unwrap_or_default()
}
