//! Shared renderer logic for toddy.
//!
//! This crate contains the platform-independent rendering engine that
//! processes incoming messages, dispatches iced updates, and emits
//! outgoing events. It compiles to both native and wasm32 targets.
//!
//! Platform-specific behavior (I/O, effects, sleep) is injected via
//! traits and cfg-gated dependencies. The `toddy` binary crate and
//! `toddy-web` WASM crate each provide their own implementations.

mod app;
mod apply;
pub mod constants;
mod emitter;
pub mod emitters;
mod events;
pub mod message_processing;
pub mod scripting;
mod subscriptions;
mod update;
mod view;
mod widget_ops;
mod window_map;
mod window_ops;

pub mod effects;

pub use app::App;
pub use effects::EffectHandler;
