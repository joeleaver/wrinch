//! Procedural macros for rinch - RSX syntax.
//!
//! Provides the `rsx!` macro for declarative UI definition.

mod prop_schema;
mod suggestions;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, token, Expr, Ident, LitStr, Result, Token};

use prop_schema::{get_prop_names, get_required_props, is_valid_prop};
use suggestions::{format_missing_prop_error, format_unknown_prop_error};

/// The main RSX macro for building UI.
///
/// # Example
///
/// ```ignore
/// use rinch::prelude::*;
///
/// fn app() -> Element {
///     rsx! {
///         Window { title: "My App", width: 800, height: 600,
///             div {
///                 h1 { "Hello, Rinch!" }
///             }
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let node = syn::parse_macro_input!(input as RsxNode);
    node.to_element().into()
}

/// A node in the RSX tree.
enum RsxNode {
    /// A component or HTML element with optional props and children.
    Element(RsxElement),
    /// A text literal.
    Text(LitStr),
    /// A Rust expression in braces.
    Expr(Expr),
}

impl Parse for RsxNode {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            Ok(RsxNode::Text(input.parse()?))
        } else if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            Ok(RsxNode::Expr(content.parse()?))
        } else {
            Ok(RsxNode::Element(input.parse()?))
        }
    }
}

impl RsxNode {
    fn to_element(&self) -> TokenStream2 {
        match self {
            RsxNode::Element(el) => el.to_element(),
            RsxNode::Text(lit) => {
                let text = lit.value();
                quote! { Element::Html(#text.into()) }
            }
            RsxNode::Expr(expr) => {
                // Wrap expressions in a ToString call for display
                quote! { Element::Html(::std::string::ToString::to_string(&#expr).into()) }
            }
        }
    }

    fn to_html_tokens(&self) -> TokenStream2 {
        match self {
            RsxNode::Element(el) => el.to_html_tokens(),
            RsxNode::Text(lit) => {
                let text = html_escape(&lit.value());
                quote! { #text }
            }
            RsxNode::Expr(expr) => {
                // Dynamic expression - needs runtime string conversion
                quote! { &::rinch::core::events::html_escape_string(&::std::string::ToString::to_string(&#expr)) }
            }
        }
    }

    fn is_rinch_component(&self) -> bool {
        match self {
            RsxNode::Element(el) => el.is_rinch_component(),
            _ => false,
        }
    }

    fn has_dynamic_content(&self) -> bool {
        match self {
            RsxNode::Element(el) => el.has_dynamic_content(),
            RsxNode::Text(_) => false,
            RsxNode::Expr(_) => true,
        }
    }
}

/// An element in RSX (component or HTML tag).
struct RsxElement {
    name: Ident,
    props: Vec<RsxProp>,
    children: Vec<RsxNode>,
}

impl Parse for RsxElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        let content;
        braced!(content in input);

        let mut props = Vec::new();
        let mut children = Vec::new();

        while !content.is_empty() {
            // Try to parse as a prop (name: value)
            if content.peek(Ident) && content.peek2(Token![:]) && !content.peek2(Token![::]) {
                let prop: RsxProp = content.parse()?;
                props.push(prop);

                // Consume trailing comma if present
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            } else {
                // Parse as a child node
                let child: RsxNode = content.parse()?;
                children.push(child);

                // Consume trailing comma if present
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
        }

        Ok(RsxElement {
            name,
            props,
            children,
        })
    }
}

impl RsxElement {
    fn is_rinch_component(&self) -> bool {
        let name = self.name.to_string();
        matches!(
            name.as_str(),
            "Window" | "AppMenu" | "Menu" | "MenuItem" | "MenuSeparator" | "Fragment"
        )
    }

    /// Validate props for a component and return a compile_error! if invalid.
    /// Returns None if validation passes, Some(error_tokens) otherwise.
    fn validate_props(&self) -> Option<TokenStream2> {
        let component_name = self.name.to_string();

        // Skip validation for HTML elements (not rinch components)
        if !self.is_rinch_component() {
            return None;
        }

        // MenuSeparator and Fragment don't have props
        if component_name == "MenuSeparator" || component_name == "Fragment" {
            return None;
        }

        let valid_props = get_prop_names(&component_name);
        let required_props = get_required_props(&component_name);

        // Check for unknown props
        for prop in &self.props {
            let prop_name = prop.name.to_string();
            if !is_valid_prop(&component_name, &prop_name) {
                let error_msg = format_unknown_prop_error(&component_name, &prop_name, &valid_props);
                return Some(syn::Error::new_spanned(&prop.name, error_msg).to_compile_error());
            }
        }

        // Check for missing required props
        let provided_props: Vec<String> = self.props.iter().map(|p| p.name.to_string()).collect();
        for required in required_props {
            if !provided_props.iter().any(|p| p == required) {
                let error_msg = format_missing_prop_error(&component_name, required);
                return Some(syn::Error::new_spanned(&self.name, error_msg).to_compile_error());
            }
        }

        None
    }

    /// Check if this element or any children have event handlers or dynamic expressions.
    fn has_dynamic_content(&self) -> bool {
        // Check for event handlers
        if self.props.iter().any(|p| is_event_prop(&p.name.to_string())) {
            return true;
        }

        // Check for non-literal prop values (dynamic attributes)
        if self.props.iter().any(|p| !is_literal_expr(&p.value)) {
            return true;
        }

        // Check children
        self.children.iter().any(|c| c.has_dynamic_content())
    }

    fn to_element(&self) -> TokenStream2 {
        // Validate props first - return compile error if invalid
        if let Some(error) = self.validate_props() {
            return error;
        }

        let name_str = self.name.to_string();

        match name_str.as_str() {
            "Window" => self.gen_window(),
            "AppMenu" => self.gen_app_menu(),
            "Menu" => self.gen_menu(),
            "MenuItem" => self.gen_menu_item(),
            "MenuSeparator" => quote! { Element::MenuSeparator },
            "Fragment" => self.gen_fragment(),
            _ => self.gen_html_element(),
        }
    }

    fn gen_window(&self) -> TokenStream2 {
        let props = self.gen_window_props();
        let children = self.gen_children_as_elements();

        quote! {
            Element::Window(#props, #children)
        }
    }

    fn gen_window_props(&self) -> TokenStream2 {
        let mut title = quote! { String::from("Rinch Window") };
        let mut width = quote! { 800 };
        let mut height = quote! { 600 };
        let mut x = quote! { None };
        let mut y = quote! { None };
        let mut borderless = quote! { false };
        let mut resizable = quote! { true };
        let mut transparent = quote! { false };
        let mut always_on_top = quote! { false };
        let mut visible = quote! { true };

        for prop in &self.props {
            let name = prop.name.to_string();
            let value = &prop.value;

            match name.as_str() {
                "title" => title = quote! { String::from(#value) },
                "width" => width = quote! { #value },
                "height" => height = quote! { #value },
                "x" => x = quote! { Some(#value) },
                "y" => y = quote! { Some(#value) },
                "borderless" => borderless = quote! { #value },
                "resizable" => resizable = quote! { #value },
                "transparent" => transparent = quote! { #value },
                "always_on_top" => always_on_top = quote! { #value },
                "visible" => visible = quote! { #value },
                _ => {}
            }
        }

        quote! {
            WindowProps {
                title: #title,
                width: #width,
                height: #height,
                x: #x,
                y: #y,
                borderless: #borderless,
                resizable: #resizable,
                transparent: #transparent,
                always_on_top: #always_on_top,
                visible: #visible,
            }
        }
    }

    fn gen_app_menu(&self) -> TokenStream2 {
        let mut native = quote! { true };

        for prop in &self.props {
            if prop.name == "native" {
                let value = &prop.value;
                native = quote! { #value };
            }
        }

        let children = self.gen_children_as_elements();

        quote! {
            Element::AppMenu(
                AppMenuProps { native: #native },
                #children
            )
        }
    }

    fn gen_menu(&self) -> TokenStream2 {
        let mut label = quote! { String::new() };

        for prop in &self.props {
            if prop.name == "label" {
                let value = &prop.value;
                label = quote! { String::from(#value) };
            }
        }

        let children = self.gen_children_as_elements();

        quote! {
            Element::Menu(
                MenuProps { label: #label },
                #children
            )
        }
    }

    fn gen_menu_item(&self) -> TokenStream2 {
        let mut label = quote! { String::new() };
        let mut shortcut = quote! { None };
        let mut enabled = quote! { true };
        let mut checked = quote! { None };
        let mut onclick = quote! { None };

        for prop in &self.props {
            let name = prop.name.to_string();
            let value = &prop.value;

            match name.as_str() {
                "label" => label = quote! { String::from(#value) },
                "shortcut" => shortcut = quote! { Some(String::from(#value)) },
                "enabled" => enabled = quote! { #value },
                "checked" => checked = quote! { Some(#value) },
                "onclick" => onclick = quote! { Some(MenuItemCallback::new(#value)) },
                _ => {}
            }
        }

        quote! {
            Element::MenuItem(MenuItemProps {
                label: #label,
                shortcut: #shortcut,
                enabled: #enabled,
                checked: #checked,
                onclick: #onclick,
            })
        }
    }

    fn gen_fragment(&self) -> TokenStream2 {
        let children = self.gen_children_as_elements();
        quote! { Element::Fragment(#children) }
    }

    fn gen_children_as_elements(&self) -> TokenStream2 {
        if self.children.is_empty() {
            return quote! { vec![] };
        }

        // Check if all children are HTML elements (can be combined into one HTML string)
        let all_html = self.children.iter().all(|c| !c.is_rinch_component());

        if all_html {
            // Check if we need dynamic HTML generation
            let has_dynamic = self.children.iter().any(|c| c.has_dynamic_content());

            if has_dynamic {
                // Generate runtime HTML building
                let html_parts: Vec<TokenStream2> =
                    self.children.iter().map(|c| c.to_html_tokens()).collect();

                quote! {
                    vec![Element::Html({
                        let mut __html = String::new();
                        #( __html.push_str(#html_parts); )*
                        __html
                    })]
                }
            } else {
                // Static HTML string
                let html: String = self.children.iter().map(|c| node_to_static_html(c)).collect();
                quote! { vec![Element::Html(#html.into())] }
            }
        } else {
            // Mix of components and HTML - generate each separately
            let children: Vec<TokenStream2> = self
                .children
                .iter()
                .map(|c| {
                    if c.is_rinch_component() {
                        c.to_element()
                    } else if c.has_dynamic_content() {
                        let html_tokens = c.to_html_tokens();
                        quote! { Element::Html(#html_tokens.into()) }
                    } else {
                        let html = node_to_static_html(c);
                        quote! { Element::Html(#html.into()) }
                    }
                })
                .collect();

            quote! { vec![#(#children),*] }
        }
    }

    fn gen_html_element(&self) -> TokenStream2 {
        if self.has_dynamic_content() {
            self.gen_dynamic_html_element()
        } else {
            let html = self.to_static_html();
            quote! { Element::Html(#html.into()) }
        }
    }

    fn gen_dynamic_html_element(&self) -> TokenStream2 {
        let tag = self.name.to_string();

        // Separate event handlers from regular attributes
        let (event_props, attr_props): (Vec<_>, Vec<_>) = self
            .props
            .iter()
            .partition(|p| is_event_prop(&p.name.to_string()));

        // Build attribute string
        let attr_parts: Vec<TokenStream2> = attr_props
            .iter()
            .map(|p| {
                let name = p.name.to_string();
                let value = &p.value;
                if is_literal_expr(value) {
                    let val_str = expr_to_string(value);
                    let escaped = html_escape(&val_str);
                    let attr = format!(" {}=\"{}\"", name, escaped);
                    quote! { #attr }
                } else {
                    // Dynamic attribute value
                    quote! {
                        &format!(" {}=\"{}\"", #name, ::rinch::core::events::html_escape_string(&::std::string::ToString::to_string(&#value)))
                    }
                }
            })
            .collect();

        // Generate event handler registration
        let event_registrations: Vec<TokenStream2> = event_props
            .iter()
            .map(|p| {
                let handler = &p.value;
                quote! {
                    let __handler_id = ::rinch::core::register_handler(Box::new(#handler));
                }
            })
            .collect();

        // Build the data-rid attribute if we have event handlers
        let rid_attr = if !event_props.is_empty() {
            quote! { &format!(" data-rid=\"{}\"", __handler_id) }
        } else {
            quote! { "" }
        };

        // Build children HTML
        let children_tokens: Vec<TokenStream2> =
            self.children.iter().map(|c| c.to_html_tokens()).collect();

        if is_void_element(&tag) {
            quote! {
                {
                    #(#event_registrations)*
                    Element::Html({
                        let mut __html = String::new();
                        __html.push_str("<");
                        __html.push_str(#tag);
                        #( __html.push_str(#attr_parts); )*
                        __html.push_str(#rid_attr);
                        __html.push_str(" />");
                        __html
                    })
                }
            }
        } else {
            quote! {
                {
                    #(#event_registrations)*
                    Element::Html({
                        let mut __html = String::new();
                        __html.push_str("<");
                        __html.push_str(#tag);
                        #( __html.push_str(#attr_parts); )*
                        __html.push_str(#rid_attr);
                        __html.push_str(">");
                        #( __html.push_str(#children_tokens); )*
                        __html.push_str("</");
                        __html.push_str(#tag);
                        __html.push_str(">");
                        __html
                    })
                }
            }
        }
    }

    fn to_html_tokens(&self) -> TokenStream2 {
        if self.has_dynamic_content() {
            self.gen_dynamic_html_tokens()
        } else {
            let html = self.to_static_html();
            quote! { #html }
        }
    }

    fn gen_dynamic_html_tokens(&self) -> TokenStream2 {
        let tag = self.name.to_string();

        // Separate event handlers from regular attributes
        let (event_props, attr_props): (Vec<_>, Vec<_>) = self
            .props
            .iter()
            .partition(|p| is_event_prop(&p.name.to_string()));

        // Build attribute parts
        let attr_parts: Vec<TokenStream2> = attr_props
            .iter()
            .map(|p| {
                let name = p.name.to_string();
                let value = &p.value;
                if is_literal_expr(value) {
                    let val_str = expr_to_string(value);
                    let escaped = html_escape(&val_str);
                    let attr = format!(" {}=\"{}\"", name, escaped);
                    quote! { __html.push_str(#attr); }
                } else {
                    quote! {
                        __html.push_str(&format!(" {}=\"{}\"", #name, ::rinch::core::events::html_escape_string(&::std::string::ToString::to_string(&#value))));
                    }
                }
            })
            .collect();

        // Event handler registrations
        let event_registrations: Vec<TokenStream2> = event_props
            .iter()
            .map(|p| {
                let handler = &p.value;
                quote! {
                    let __handler_id = ::rinch::core::register_handler(Box::new(#handler));
                }
            })
            .collect();

        // data-rid attribute
        let rid_attr = if !event_props.is_empty() {
            quote! { __html.push_str(&format!(" data-rid=\"{}\"", __handler_id)); }
        } else {
            quote! {}
        };

        // Children
        let children_tokens: Vec<TokenStream2> = self
            .children
            .iter()
            .map(|c| {
                let tokens = c.to_html_tokens();
                quote! { __html.push_str(#tokens); }
            })
            .collect();

        if is_void_element(&tag) {
            quote! {
                &{
                    #(#event_registrations)*
                    let mut __html = String::new();
                    __html.push_str("<");
                    __html.push_str(#tag);
                    #( #attr_parts )*
                    #rid_attr
                    __html.push_str(" />");
                    __html
                }
            }
        } else {
            quote! {
                &{
                    #(#event_registrations)*
                    let mut __html = String::new();
                    __html.push_str("<");
                    __html.push_str(#tag);
                    #( #attr_parts )*
                    #rid_attr
                    __html.push_str(">");
                    #( #children_tokens )*
                    __html.push_str("</");
                    __html.push_str(#tag);
                    __html.push_str(">");
                    __html
                }
            }
        }
    }

    fn to_static_html(&self) -> String {
        let tag = self.name.to_string();

        // Build attributes (skip event handlers)
        let attrs: String = self
            .props
            .iter()
            .filter(|p| !is_event_prop(&p.name.to_string()))
            .map(|p| {
                let name = p.name.to_string();
                let value = expr_to_string(&p.value);
                format!(" {}=\"{}\"", name, html_escape(&value))
            })
            .collect();

        // Self-closing tags
        if is_void_element(&tag) {
            return format!("<{}{} />", tag, attrs);
        }

        // Build children
        let children: String = self.children.iter().map(|c| node_to_static_html(c)).collect();

        format!("<{}{}>{}</{}>", tag, attrs, children, tag)
    }
}

/// A property in RSX (name: value).
struct RsxProp {
    name: Ident,
    value: Expr,
}

impl Parse for RsxProp {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: Expr = input.parse()?;
        Ok(RsxProp { name, value })
    }
}

/// Check if a property name is an event handler.
fn is_event_prop(name: &str) -> bool {
    name.starts_with("on")
}

/// Check if an expression is a literal (can be evaluated at compile time).
fn is_literal_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(_))
}

/// Convert an expression to a string (for HTML attribute values).
fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => s.value(),
            syn::Lit::Int(i) => i.base10_digits().to_string(),
            syn::Lit::Float(f) => f.base10_digits().to_string(),
            syn::Lit::Bool(b) => b.value.to_string(),
            _ => expr.to_token_stream().to_string(),
        },
        _ => expr.to_token_stream().to_string(),
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Check if an HTML tag is a void element (self-closing).
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "param" | "source" | "track" | "wbr"
    )
}

/// Convert an RSX node to static HTML (for compile-time generation).
fn node_to_static_html(node: &RsxNode) -> String {
    match node {
        RsxNode::Element(el) => el.to_static_html(),
        RsxNode::Text(lit) => html_escape(&lit.value()),
        RsxNode::Expr(_) => String::new(), // Expressions can't be static
    }
}
