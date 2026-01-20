//! Core types and traits for rinch.

pub mod element;
pub mod event;
pub mod events;
pub mod hooks;
pub mod reactive;

// Re-export reactive types for convenience
pub use reactive::{batch, derived, untracked, Effect, Memo, Scope, Signal};

// Re-export hooks for ergonomic state management
pub use hooks::{
    begin_render, clear_hooks, create_context, end_render, get_hooks_debug_info, use_callback,
    use_context, use_derived, use_effect, use_effect_cleanup, use_memo, use_mount, use_ref,
    use_signal, use_state, HookMeta, RefHandle,
};

// Re-export event handling types
pub use events::{
    clear_handlers, dispatch_event, register_handler, EventCallback, EventHandlerId,
};
