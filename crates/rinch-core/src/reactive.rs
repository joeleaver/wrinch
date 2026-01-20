//! Reactive primitives: signals, effects, and memos.
//!
//! This module provides fine-grained reactivity similar to Solid.js and Leptos.
//!
//! # Core Concepts
//!
//! - **Signal**: A reactive container that holds a value and notifies subscribers when it changes
//! - **Effect**: A side-effect that re-runs when its dependencies change
//! - **Memo**: A cached computed value that only recomputes when dependencies change
//!
//! # Example
//!
//! ```ignore
//! use rinch_core::reactive::*;
//!
//! let count = Signal::new(0);
//!
//! Effect::new(move || {
//!     println!("Count is: {}", count.get());
//! });
//!
//! count.set(1); // Prints: "Count is: 1"
//! ```

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

// ============================================================================
// Runtime Context
// ============================================================================

// Global runtime state for tracking reactive subscriptions.
//
// The runtime maintains:
// - A stack of observers (effects/memos currently being computed)
// - A queue of pending effects to run
// - Batching state
thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

struct Runtime {
    /// Stack of currently executing observers
    observer_stack: Vec<ObserverId>,

    /// Effects that need to run
    pending_effects: Vec<ObserverId>,

    /// Whether we're currently in a batch
    batching: bool,

    /// Counter for generating unique IDs
    next_id: usize,
}

impl Runtime {
    fn new() -> Self {
        Self {
            observer_stack: Vec::new(),
            pending_effects: Vec::new(),
            batching: false,
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

/// Unique identifier for an observer (effect or memo)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct ObserverId(usize);

// ============================================================================
// Signal
// ============================================================================

/// A reactive container that holds a value and notifies subscribers when it changes.
///
/// Signals are the foundational reactive primitive. Reading a signal inside an effect
/// automatically subscribes that effect to the signal. Setting a signal notifies
/// all subscribers to re-run.
///
/// # Example
///
/// ```ignore
/// let count = Signal::new(0);
///
/// // Read the value
/// let value = count.get();
///
/// // Update the value (triggers subscribers)
/// count.set(5);
///
/// // Update based on current value
/// count.update(|n| *n += 1);
/// ```
pub struct Signal<T> {
    inner: Rc<SignalInner<T>>,
}

struct SignalInner<T> {
    value: RefCell<T>,
    subscribers: RefCell<HashSet<ObserverId>>,
}

impl<T> Signal<T> {
    /// Create a new signal with the given initial value.
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(SignalInner {
                value: RefCell::new(value),
                subscribers: RefCell::new(HashSet::new()),
            }),
        }
    }

    /// Subscribe the current observer (if any) to this signal.
    fn track(&self) {
        RUNTIME.with(|rt| {
            let rt = rt.borrow();
            if let Some(&observer) = rt.observer_stack.last() {
                self.inner.subscribers.borrow_mut().insert(observer);
            }
        });
    }

    /// Notify all subscribers that the value has changed.
    fn notify(&self) {
        let subscribers: Vec<_> = self.inner.subscribers.borrow().iter().copied().collect();

        RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            for observer in subscribers {
                if !rt.pending_effects.contains(&observer) {
                    rt.pending_effects.push(observer);
                }
            }

            // If not batching, flush immediately
            if !rt.batching {
                drop(rt);
                flush_effects();
            }
        });
    }
}

impl<T: Clone> Signal<T> {
    /// Get the current value of the signal.
    ///
    /// If called inside an effect, this automatically subscribes the effect
    /// to this signal.
    pub fn get(&self) -> T {
        self.track();
        self.inner.value.borrow().clone()
    }
}

impl<T> Signal<T> {
    /// Get a reference to the current value without cloning.
    ///
    /// If called inside an effect, this automatically subscribes the effect
    /// to this signal.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.track();
        f(&*self.inner.value.borrow())
    }

    /// Set the signal to a new value.
    ///
    /// This will notify all subscribers to re-run.
    pub fn set(&self, value: T) {
        *self.inner.value.borrow_mut() = value;
        self.notify();
    }

    /// Update the signal's value using a function.
    ///
    /// This will notify all subscribers to re-run.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut *self.inner.value.borrow_mut());
        self.notify();
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signal")
            .field("value", &*self.inner.value.borrow())
            .finish()
    }
}

impl<T: fmt::Display> fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.inner.value.borrow(), f)
    }
}

// ============================================================================
// Effect
// ============================================================================

// Storage for all effects (needed because effects reference themselves)
thread_local! {
    static EFFECTS: RefCell<Vec<Option<Rc<EffectInner>>>> = RefCell::new(Vec::new());
}

/// A side-effect that re-runs when its dependencies change.
///
/// Effects automatically track which signals they read and re-run when
/// any of those signals change.
///
/// # Example
///
/// ```ignore
/// let count = Signal::new(0);
///
/// Effect::new(move || {
///     println!("Count is now: {}", count.get());
/// });
///
/// count.set(1); // Prints: "Count is now: 1"
/// count.set(2); // Prints: "Count is now: 2"
/// ```
pub struct Effect {
    id: ObserverId,
}

struct EffectInner {
    #[allow(dead_code)] // Used for debugging/tracking purposes
    id: ObserverId,
    f: RefCell<Box<dyn FnMut()>>,
    disposed: Cell<bool>,
}

impl Effect {
    /// Create a new effect that runs immediately and re-runs when dependencies change.
    pub fn new<F: FnMut() + 'static>(f: F) -> Self {
        let id = RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            ObserverId(rt.next_id())
        });

        let inner = Rc::new(EffectInner {
            id,
            f: RefCell::new(Box::new(f)),
            disposed: Cell::new(false),
        });

        // Store the effect
        EFFECTS.with(|effects| {
            let mut effects = effects.borrow_mut();
            let idx = id.0;
            if idx >= effects.len() {
                effects.resize(idx + 1, None);
            }
            effects[idx] = Some(Rc::clone(&inner));
        });

        // Run the effect immediately
        run_effect(id);

        Effect { id }
    }

    /// Create an effect that doesn't run immediately.
    pub fn new_deferred<F: FnMut() + 'static>(f: F) -> Self {
        let id = RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            ObserverId(rt.next_id())
        });

        let inner = Rc::new(EffectInner {
            id,
            f: RefCell::new(Box::new(f)),
            disposed: Cell::new(false),
        });

        EFFECTS.with(|effects| {
            let mut effects = effects.borrow_mut();
            let idx = id.0;
            if idx >= effects.len() {
                effects.resize(idx + 1, None);
            }
            effects[idx] = Some(inner);
        });

        Effect { id }
    }

    /// Manually trigger this effect to run.
    pub fn run(&self) {
        run_effect(self.id);
    }

    /// Dispose of this effect, preventing it from running again.
    pub fn dispose(&self) {
        EFFECTS.with(|effects| {
            if let Some(Some(inner)) = effects.borrow().get(self.id.0) {
                inner.disposed.set(true);
            }
        });
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        // Note: We don't automatically dispose here to allow effects to outlive
        // their handles. Use dispose() explicitly if needed.
    }
}

/// Run a specific effect by ID
fn run_effect(id: ObserverId) {
    let effect = EFFECTS.with(|effects| {
        effects.borrow().get(id.0).and_then(|e| e.clone())
    });

    if let Some(inner) = effect {
        if inner.disposed.get() {
            return;
        }

        // Push this effect as the current observer
        RUNTIME.with(|rt| {
            rt.borrow_mut().observer_stack.push(id);
        });

        // Run the effect
        (inner.f.borrow_mut())();

        // Pop the observer
        RUNTIME.with(|rt| {
            rt.borrow_mut().observer_stack.pop();
        });
    }
}

/// Flush all pending effects
fn flush_effects() {
    loop {
        let effect_id = RUNTIME.with(|rt| {
            rt.borrow_mut().pending_effects.pop()
        });

        match effect_id {
            Some(id) => run_effect(id),
            None => break,
        }
    }
}

// ============================================================================
// Memo
// ============================================================================

/// A cached computed value that only recomputes when dependencies change.
///
/// Memos are lazily evaluated and cache their result until one of their
/// dependencies changes.
///
/// # Example
///
/// ```ignore
/// let count = Signal::new(2);
/// let doubled = Memo::new(move || count.get() * 2);
///
/// doubled.get(); // Returns 4
/// count.set(3);
/// doubled.get(); // Returns 6 (recomputed)
/// doubled.get(); // Returns 6 (cached)
/// ```
pub struct Memo<T> {
    inner: Rc<MemoInner<T>>,
}

struct MemoInner<T> {
    id: ObserverId,
    value: RefCell<Option<T>>,
    f: RefCell<Box<dyn Fn() -> T>>,
    dirty: Cell<bool>,
    subscribers: RefCell<HashSet<ObserverId>>,
}

impl<T: Clone + 'static> Memo<T> {
    /// Create a new memo with the given computation function.
    pub fn new<F: Fn() -> T + 'static>(f: F) -> Self {
        let id = RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            ObserverId(rt.next_id())
        });

        let inner = Rc::new(MemoInner {
            id,
            value: RefCell::new(None),
            f: RefCell::new(Box::new(f)),
            dirty: Cell::new(true),
            subscribers: RefCell::new(HashSet::new()),
        });

        // Store memo as an effect so it can be notified
        let inner_clone = Rc::clone(&inner);
        EFFECTS.with(|effects| {
            let mut effects = effects.borrow_mut();
            let idx = id.0;
            if idx >= effects.len() {
                effects.resize(idx + 1, None);
            }
            // We store a "marker" effect that marks the memo as dirty
            let memo_inner = inner_clone;
            effects[idx] = Some(Rc::new(EffectInner {
                id,
                f: RefCell::new(Box::new(move || {
                    memo_inner.dirty.set(true);
                    // Notify memo's subscribers
                    let subscribers: Vec<_> = memo_inner.subscribers.borrow().iter().copied().collect();
                    RUNTIME.with(|rt| {
                        let mut rt = rt.borrow_mut();
                        for observer in subscribers {
                            if !rt.pending_effects.contains(&observer) {
                                rt.pending_effects.push(observer);
                            }
                        }
                    });
                })),
                disposed: Cell::new(false),
            }));
        });

        Self { inner }
    }

    /// Get the current value, recomputing if necessary.
    pub fn get(&self) -> T {
        // Subscribe current observer to this memo
        RUNTIME.with(|rt| {
            let rt = rt.borrow();
            if let Some(&observer) = rt.observer_stack.last() {
                self.inner.subscribers.borrow_mut().insert(observer);
            }
        });

        // Recompute if dirty
        if self.inner.dirty.get() {
            // Push memo as observer while computing
            RUNTIME.with(|rt| {
                rt.borrow_mut().observer_stack.push(self.inner.id);
            });

            let value = (self.inner.f.borrow())();
            *self.inner.value.borrow_mut() = Some(value);
            self.inner.dirty.set(false);

            RUNTIME.with(|rt| {
                rt.borrow_mut().observer_stack.pop();
            });
        }

        self.inner.value.borrow().clone().expect("memo should have value after get")
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: fmt::Debug + Clone + 'static> fmt::Debug for Memo<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memo")
            .field("value", &*self.inner.value.borrow())
            .field("dirty", &self.inner.dirty.get())
            .finish()
    }
}

// ============================================================================
// Batching
// ============================================================================

/// Batch multiple signal updates to avoid redundant effect runs.
///
/// Effects will only run once after the batch completes, even if multiple
/// signals they depend on are updated.
///
/// # Example
///
/// ```ignore
/// let count = Signal::new(0);
/// let name = Signal::new("".to_string());
///
/// batch(|| {
///     count.set(1);
///     name.set("Alice".to_string());
///     // Effects only run once, after this batch
/// });
/// ```
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    RUNTIME.with(|rt| {
        rt.borrow_mut().batching = true;
    });

    let result = f();

    RUNTIME.with(|rt| {
        rt.borrow_mut().batching = false;
    });

    flush_effects();

    result
}

// ============================================================================
// Scope (for memory management)
// ============================================================================

/// A scope that manages the lifetime of reactive primitives.
///
/// When a scope is disposed, all effects created within it are cleaned up.
///
/// # Example
///
/// ```ignore
/// let scope = Scope::new();
///
/// scope.run(|| {
///     let signal = Signal::new(0);
///     Effect::new(|| { /* ... */ });
///     // signal and effect belong to this scope
/// });
///
/// scope.dispose(); // Cleans up signal and effect
/// ```
pub struct Scope {
    effects: RefCell<Vec<Effect>>,
}

impl Scope {
    /// Create a new scope.
    pub fn new() -> Self {
        Self {
            effects: RefCell::new(Vec::new()),
        }
    }

    /// Run a function within this scope, capturing any effects created.
    pub fn run<R>(&self, f: impl FnOnce() -> R) -> R {
        // TODO: Implement scope tracking so effects created within
        // are automatically registered to this scope
        f()
    }

    /// Register an effect with this scope.
    pub fn add_effect(&self, effect: Effect) {
        self.effects.borrow_mut().push(effect);
    }

    /// Dispose of all effects in this scope.
    pub fn dispose(&self) {
        for effect in self.effects.borrow().iter() {
            effect.dispose();
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.dispose();
    }
}

// ============================================================================
// Utility functions
// ============================================================================

/// Create a derived signal from a computation.
///
/// This is a convenience function that creates a memo and returns it
/// as a signal-like value.
pub fn derived<T: Clone + 'static>(f: impl Fn() -> T + 'static) -> Memo<T> {
    Memo::new(f)
}

/// Run a function without tracking any signal reads.
///
/// Useful for reading signals without creating subscriptions.
pub fn untracked<R>(f: impl FnOnce() -> R) -> R {
    // Temporarily remove the current observer
    let observer = RUNTIME.with(|rt| {
        rt.borrow_mut().observer_stack.pop()
    });

    let result = f();

    // Restore the observer
    if let Some(obs) = observer {
        RUNTIME.with(|rt| {
            rt.borrow_mut().observer_stack.push(obs);
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn signal_basic() {
        let signal = Signal::new(0);
        assert_eq!(signal.get(), 0);

        signal.set(5);
        assert_eq!(signal.get(), 5);

        signal.update(|n| *n += 1);
        assert_eq!(signal.get(), 6);
    }

    #[test]
    fn effect_tracks_signals() {
        let count = Signal::new(0);
        let run_count = Rc::new(Cell::new(0));

        let run_count_clone = Rc::clone(&run_count);
        let count_clone = count.clone();
        Effect::new(move || {
            let _ = count_clone.get();
            run_count_clone.set(run_count_clone.get() + 1);
        });

        // Effect runs immediately
        assert_eq!(run_count.get(), 1);

        // Effect runs when signal changes
        count.set(1);
        assert_eq!(run_count.get(), 2);

        count.set(2);
        assert_eq!(run_count.get(), 3);
    }

    #[test]
    fn memo_caches_value() {
        let count = Signal::new(2);
        let compute_count = Rc::new(Cell::new(0));

        let compute_count_clone = Rc::clone(&compute_count);
        let count_clone = count.clone();
        let doubled = Memo::new(move || {
            compute_count_clone.set(compute_count_clone.get() + 1);
            count_clone.get() * 2
        });

        // First access computes
        assert_eq!(doubled.get(), 4);
        assert_eq!(compute_count.get(), 1);

        // Second access uses cache
        assert_eq!(doubled.get(), 4);
        assert_eq!(compute_count.get(), 1);

        // Update signal
        count.set(3);

        // Next access recomputes
        assert_eq!(doubled.get(), 6);
        assert_eq!(compute_count.get(), 2);
    }

    #[test]
    fn batch_prevents_multiple_runs() {
        let count = Signal::new(0);
        let run_count = Rc::new(Cell::new(0));

        let run_count_clone = Rc::clone(&run_count);
        let count_clone = count.clone();
        Effect::new(move || {
            let _ = count_clone.get();
            run_count_clone.set(run_count_clone.get() + 1);
        });

        // Effect runs immediately
        assert_eq!(run_count.get(), 1);

        // Batch multiple updates
        batch(|| {
            count.set(1);
            count.set(2);
            count.set(3);
        });

        // Effect only ran once more
        assert_eq!(run_count.get(), 2);
        assert_eq!(count.get(), 3);
    }

    #[test]
    fn untracked_prevents_subscription() {
        let count = Signal::new(0);
        let run_count = Rc::new(Cell::new(0));

        let run_count_clone = Rc::clone(&run_count);
        let count_clone = count.clone();
        Effect::new(move || {
            untracked(|| {
                let _ = count_clone.get();
            });
            run_count_clone.set(run_count_clone.get() + 1);
        });

        // Effect runs immediately
        assert_eq!(run_count.get(), 1);

        // Effect does NOT run when signal changes (untracked)
        count.set(1);
        assert_eq!(run_count.get(), 1);
    }
}
