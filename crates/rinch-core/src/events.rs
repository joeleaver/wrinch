//! Event handling infrastructure for rinch.
//!
//! This module provides the event handler registry that maps element IDs
//! to Rust callbacks, enabling reactive event handling in the UI.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Escape HTML special characters in a string.
///
/// This is used at runtime for dynamic content in RSX.
pub fn html_escape_string(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Unique identifier for an event handler.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EventHandlerId(pub usize);

impl std::fmt::Display for EventHandlerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type alias for event handler callbacks.
pub type EventCallback = Box<dyn Fn() + 'static>;

/// Global counter for generating unique event handler IDs.
static NEXT_HANDLER_ID: AtomicUsize = AtomicUsize::new(0);

/// Generate a new unique event handler ID.
pub fn next_handler_id() -> EventHandlerId {
    EventHandlerId(NEXT_HANDLER_ID.fetch_add(1, Ordering::SeqCst))
}

/// Reset the handler ID counter (useful for testing or re-rendering).
pub fn reset_handler_ids() {
    NEXT_HANDLER_ID.store(0, Ordering::SeqCst);
}

// Thread-local event handler registry.
thread_local! {
    static EVENT_REGISTRY: RefCell<EventRegistry> = RefCell::new(EventRegistry::new());
}

/// Registry that maps event handler IDs to callbacks.
pub struct EventRegistry {
    handlers: HashMap<EventHandlerId, EventCallback>,
}

impl EventRegistry {
    fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
}

/// Register an event handler and return its ID.
///
/// The handler will be called when an element with the corresponding
/// `data-rid` attribute is clicked.
///
/// # Example
///
/// ```ignore
/// let id = register_handler(Box::new(|| {
///     println!("Button clicked!");
/// }));
/// // The element should have: data-rid="{id}"
/// ```
pub fn register_handler(callback: EventCallback) -> EventHandlerId {
    let id = next_handler_id();
    EVENT_REGISTRY.with(|registry| {
        registry.borrow_mut().handlers.insert(id, callback);
    });
    id
}

/// Dispatch an event to the handler with the given ID.
///
/// Returns `true` if a handler was found and called, `false` otherwise.
pub fn dispatch_event(id: EventHandlerId) -> bool {
    EVENT_REGISTRY.with(|registry| {
        if let Some(handler) = registry.borrow().handlers.get(&id) {
            handler();
            true
        } else {
            false
        }
    })
}

/// Clear all registered event handlers.
///
/// This should be called before re-rendering to avoid stale handlers.
pub fn clear_handlers() {
    EVENT_REGISTRY.with(|registry| {
        registry.borrow_mut().handlers.clear();
    });
    reset_handler_ids();
}

/// Get the number of registered handlers (for debugging).
pub fn handler_count() -> usize {
    EVENT_REGISTRY.with(|registry| registry.borrow().handlers.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_register_and_dispatch() {
        clear_handlers();

        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        let id = register_handler(Box::new(move || {
            called_clone.set(true);
        }));

        assert!(!called.get());
        assert!(dispatch_event(id));
        assert!(called.get());
    }

    #[test]
    fn test_dispatch_unknown_id() {
        clear_handlers();

        let unknown_id = EventHandlerId(99999);
        assert!(!dispatch_event(unknown_id));
    }

    #[test]
    fn test_clear_handlers() {
        clear_handlers();

        let id = register_handler(Box::new(|| {}));
        assert_eq!(handler_count(), 1);

        clear_handlers();
        assert_eq!(handler_count(), 0);
        assert!(!dispatch_event(id));
    }
}
