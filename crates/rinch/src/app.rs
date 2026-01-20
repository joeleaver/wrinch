//! Application lifecycle and entry point.

/// Main application container.
pub struct App {
    // TODO: Application state
}

impl App {
    /// Create a new application.
    pub fn new() -> Self {
        Self {}
    }

    /// Run the application event loop.
    pub fn run(self) {
        // TODO: Initialize winit event loop
        // TODO: Create window with blitz renderer
        // TODO: Run event loop
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
