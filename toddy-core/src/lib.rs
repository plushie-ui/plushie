//! # toddy-core
//!
//! The public SDK for toddy. Extension authors depend on this crate to
//! implement the [`WidgetExtension`](extensions::WidgetExtension) trait
//! and build custom native widgets. The [`prelude`] module re-exports
//! everything an extension needs; [`iced`] is re-exported so extensions
//! don't need a direct iced dependency.
//!
//! This crate also provides the rendering engine, wire protocol, and
//! widget infrastructure used internally by the `toddy` binary.
//!
//! ## Module guide
//!
//! **Core engine:**
//! - [`engine`] -- `Core` struct: pure state management decoupled from iced runtime
//! - [`tree`] -- tree data structure, patch application, window discovery
//! - [`message`] -- `Message` enum, keyboard/mouse serialization helpers
//!
//! **Widgets:**
//! - [`widgets`] -- tree node to iced widget rendering (all widget types)
//! - [`widgets::overlay`] -- custom `Widget` + `Overlay` impl for positioned overlays
//!
//! **Protocol:**
//! - [`protocol`] -- wire message parsing and event serialization
//! - [`codec`] -- wire codec: JSON + MessagePack encode/decode/framing
//!
//! **Platform:**
//! - [`theming`] -- theme resolution, custom palette parsing, hex colors
//! - [`image_registry`] -- in-memory image handle storage
//!
//! **Extension SDK:**
//! - [`extensions`] -- `WidgetExtension` trait, `ExtensionDispatcher`, `ExtensionCaches`
//! - [`app`] -- `ToddyAppBuilder` for registering extensions
//! - [`prelude`] -- common re-exports for extension authors
//! - [`prop_helpers`] -- public prop extraction helpers for extension authors
//! - [`testing`] -- test factory helpers for extension authors

#![deny(warnings)]

// Ensure catch_unwind works: extension panic isolation requires unwinding.
// If this fails, remove `panic = "abort"` from your Cargo profile.
#[cfg(all(not(test), panic = "abort"))]
compile_error!(
    "toddy-core requires panic=\"unwind\" (the default). \
     Extension panic isolation via catch_unwind is a no-op with panic=\"abort\"."
);

// -- Public SDK modules (stable API for extension authors) --
pub mod app;
pub mod extensions;
pub mod prelude;
pub mod prop_helpers;
pub mod testing;

// -- Internal modules used by the toddy binary --
//
// These are public so the binary crate can access them, but they are
// NOT part of the stable extension SDK. Extension authors should use
// the prelude and `toddy_core::iced::*` instead.
#[doc(hidden)]
pub mod codec;
#[doc(hidden)]
pub mod engine;
#[doc(hidden)]
pub mod image_registry;
#[doc(hidden)]
pub mod message;
#[doc(hidden)]
pub mod protocol;
#[doc(hidden)]
pub mod theming;
#[doc(hidden)]
pub mod tree;
#[doc(hidden)]
pub mod widgets;

// Re-export iced so extension crates can use `toddy_core::iced::*` without
// adding a direct iced dependency. This avoids version conflicts when
// toddy-core bumps its iced version -- extensions that use only
// `toddy_core::prelude::*` and `toddy_core::iced::*` get the upgrade
// automatically.
pub use iced;
