//! Common re-exports for widget extension authors.
//!
//! ```ignore
//! use julep_core::prelude::*;
//! ```

// Core extension types
pub use crate::extensions::{
    EventResult, ExtensionCaches, ExtensionDispatcher, GenerationCounter, RenderContext, WidgetEnv,
    WidgetExtension,
};

// App builder
pub use crate::app::JulepAppBuilder;

// Wire types
pub use crate::message::Message;
pub use crate::protocol::{OutgoingEvent, TreeNode};

// Renderer types extensions may need
pub use crate::image_registry::ImageRegistry;

// Prop helpers
pub use crate::prop_helpers::*;

// Commonly needed iced widget constructors (sourced via crate::iced re-export)
pub use crate::iced::widget::{
    button, canvas, checkbox, column, container, image, pick_list, progress_bar, row, rule,
    scrollable, slider, space, stack, text, toggler, tooltip,
};
pub use crate::iced::{Color, Element, Font, Length, Padding, Pixels, Theme};

// JSON (extensions parse props from serde_json::Value)
pub use serde_json::Value;
