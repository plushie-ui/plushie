//! Application builder for registering widget extensions.
//!
//! Extension packages create a [`PlushieAppBuilder`], register their
//! extensions, and pass it to `plushie::run()`. The default binary
//! passes an empty builder (no extensions).
//!
//! # Example
//!
//! ```ignore
//! use plushie_ext::prelude::*;
//!
//! fn main() -> iced::Result {
//!     plushie::run(
//!         PlushieAppBuilder::new()
//!             .extension(MyExtension::new())
//!             .extension(AnotherExtension::new())
//!     )
//! }
//! ```

use crate::extensions::{ExtensionDispatcher, WidgetExtension};

/// Builder for registering [`WidgetExtension`]s before starting the
/// renderer.
///
/// Each extension must have a unique `config_key()` and unique
/// `type_names()`. Duplicates panic at startup.
pub struct PlushieAppBuilder {
    extensions: Vec<Box<dyn WidgetExtension>>,
}

impl std::fmt::Debug for PlushieAppBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlushieAppBuilder")
            .field("extensions", &self.extensions.len())
            .finish()
    }
}

impl PlushieAppBuilder {
    /// Create an empty builder with no extensions registered.
    pub fn new() -> Self {
        Self { extensions: vec![] }
    }

    /// Register a widget extension.
    pub fn extension(mut self, ext: impl WidgetExtension + 'static) -> Self {
        self.extensions.push(Box::new(ext));
        self
    }

    /// Register a pre-boxed widget extension.
    ///
    /// Useful for dynamically loaded extensions (e.g. via `libloading`)
    /// where the concrete type is erased at the plugin boundary.
    pub fn extension_boxed(mut self, ext: Box<dyn WidgetExtension>) -> Self {
        self.extensions.push(ext);
        self
    }

    /// Return the config keys of all registered extensions.
    pub fn extension_keys(&self) -> Vec<&str> {
        self.extensions.iter().map(|e| e.config_key()).collect()
    }

    /// Consume the builder and produce an [`ExtensionDispatcher`].
    pub fn build_dispatcher(self) -> ExtensionDispatcher {
        ExtensionDispatcher::new(self.extensions)
    }
}

impl Default for PlushieAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}
