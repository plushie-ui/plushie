//! Subscription event handlers for keyboard, mouse, touch, IME, window
//! lifecycle, and pane grid events. Each handler checks whether the host
//! subscribed to the event type before emitting it.

use std::io;

use iced::{Point, Task, window};

use plushie_ext::message::{
    KeyEventData, Message, serialize_modifiers, serialize_mouse_button, serialize_scroll_delta,
};
use plushie_ext::protocol::OutgoingEvent;

use crate::App;
use crate::constants::*;
use crate::emitters::emit_event;

/// Convert a file path to a UTF-8 string, using lossy conversion if
/// the path contains non-UTF-8 bytes (rare on modern systems, but
/// possible on Linux with legacy filenames).
fn path_to_string(path: std::path::PathBuf) -> String {
    match path.to_str() {
        Some(s) => s.to_string(),
        None => {
            log::warn!(
                "file path contains non-UTF-8 bytes, using lossy conversion: {}",
                path.display()
            );
            path.to_string_lossy().into_owned()
        }
    }
}

/// Attach window_id to an outgoing event if the id is non-empty.
/// When the iced window::Id can't be resolved (shouldn't happen for
/// known windows), the event is emitted without a window_id and a
/// warning is logged.
fn maybe_with_window_id(event: OutgoingEvent, window_id: &str) -> OutgoingEvent {
    if window_id.is_empty() {
        event
    } else {
        event.with_window_id(window_id)
    }
}

impl App {
    /// Resolve an iced window::Id to a string window_id, logging a
    /// warning if the window is unknown. Returns an empty string for
    /// unresolved windows so handlers can proceed without panicking.
    fn resolve_window_id(&self, iced_id: &window::Id) -> String {
        let id = self.windows.window_id_for(iced_id);
        if id.is_empty() {
            log::warn!(
                "subscription event for unknown iced window {:?}, emitting without window_id",
                iced_id
            );
        }
        id
    }

    pub fn handle_key_pressed(&self, data: KeyEventData, iced_id: window::Id) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_KEY_PRESS, data.captured, |tag| {
            maybe_with_window_id(OutgoingEvent::key_press(tag, &data), &window_id)
        })
    }

    pub fn handle_key_released(&self, data: KeyEventData, iced_id: window::Id) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_KEY_RELEASE, data.captured, |tag| {
            maybe_with_window_id(OutgoingEvent::key_release(tag, &data), &window_id)
        })
    }

    pub fn handle_modifiers_changed(
        &mut self,
        mods: iced::keyboard::Modifiers,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.coalesce_subscription(SUB_MODIFIERS_CHANGED, captured, |tag| {
            maybe_with_window_id(
                OutgoingEvent::modifiers_changed(tag, serialize_modifiers(mods)),
                &window_id,
            )
        })
    }

    pub fn handle_cursor_moved(
        &mut self,
        pos: Point,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.coalesce_subscription(SUB_MOUSE_MOVE, captured, |tag| {
            maybe_with_window_id(OutgoingEvent::cursor_moved(tag, pos.x, pos.y), &window_id)
        })
    }

    pub fn handle_cursor_entered(&self, iced_id: window::Id, captured: bool) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_MOUSE_MOVE, captured, |tag| {
            maybe_with_window_id(OutgoingEvent::cursor_entered(tag), &window_id)
        })
    }

    pub fn handle_cursor_left(&self, iced_id: window::Id, captured: bool) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_MOUSE_MOVE, captured, |tag| {
            maybe_with_window_id(OutgoingEvent::cursor_left(tag), &window_id)
        })
    }

    pub fn handle_mouse_button_pressed(
        &self,
        button: iced::mouse::Button,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_MOUSE_BUTTON, captured, |tag| {
            maybe_with_window_id(
                OutgoingEvent::button_pressed(tag, serialize_mouse_button(&button)),
                &window_id,
            )
        })
    }

    pub fn handle_mouse_button_released(
        &self,
        button: iced::mouse::Button,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_MOUSE_BUTTON, captured, |tag| {
            maybe_with_window_id(
                OutgoingEvent::button_released(tag, serialize_mouse_button(&button)),
                &window_id,
            )
        })
    }

    pub fn handle_wheel_scrolled(
        &mut self,
        delta: iced::mouse::ScrollDelta,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.coalesce_subscription(SUB_MOUSE_SCROLL, captured, |tag| {
            let (dx, dy, unit) = serialize_scroll_delta(&delta);
            maybe_with_window_id(OutgoingEvent::wheel_scrolled(tag, dx, dy, unit), &window_id)
        })
    }

    pub fn handle_finger_pressed(
        &self,
        finger: iced::touch::Finger,
        pos: Point,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_TOUCH, captured, |tag| {
            maybe_with_window_id(
                OutgoingEvent::finger_pressed(tag, finger.0, pos.x, pos.y),
                &window_id,
            )
        })
    }

    pub fn handle_finger_moved(
        &mut self,
        finger: iced::touch::Finger,
        pos: Point,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.coalesce_subscription(SUB_TOUCH, captured, |tag| {
            maybe_with_window_id(
                OutgoingEvent::finger_moved(tag, finger.0, pos.x, pos.y),
                &window_id,
            )
        })
    }

    pub fn handle_finger_lifted(
        &self,
        finger: iced::touch::Finger,
        pos: Point,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_TOUCH, captured, |tag| {
            maybe_with_window_id(
                OutgoingEvent::finger_lifted(tag, finger.0, pos.x, pos.y),
                &window_id,
            )
        })
    }

    pub fn handle_finger_lost(
        &self,
        finger: iced::touch::Finger,
        pos: Point,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_TOUCH, captured, |tag| {
            maybe_with_window_id(
                OutgoingEvent::finger_lost(tag, finger.0, pos.x, pos.y),
                &window_id,
            )
        })
    }

    // IME (Input Method Editor) events for CJK and complex input.
    //
    // Platform support: Windows (Microsoft IME, Google Japanese, etc.),
    // macOS (built-in input methods), Linux/X11 (XIM/IBus), Linux/Wayland
    // (text-input-v3 protocol -- compositor support varies). The preedit
    // cursor range may be None on some older X11 IME implementations.
    pub fn handle_ime_opened(&self, iced_id: window::Id, captured: bool) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_IME, captured, |tag| {
            maybe_with_window_id(OutgoingEvent::ime_opened(tag), &window_id)
        })
    }

    pub fn handle_ime_preedit(
        &self,
        text: String,
        cursor: Option<std::ops::Range<usize>>,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_IME, captured, |tag| {
            maybe_with_window_id(OutgoingEvent::ime_preedit(tag, text, cursor), &window_id)
        })
    }

    pub fn handle_ime_commit(
        &self,
        text: String,
        iced_id: window::Id,
        captured: bool,
    ) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_IME, captured, |tag| {
            maybe_with_window_id(OutgoingEvent::ime_commit(tag, text), &window_id)
        })
    }

    pub fn handle_ime_closed(&self, iced_id: window::Id, captured: bool) -> Task<Message> {
        let window_id = self.resolve_window_id(&iced_id);
        self.emit_subscription(SUB_IME, captured, |tag| {
            maybe_with_window_id(OutgoingEvent::ime_closed(tag), &window_id)
        })
    }

    /// Emit a window event to both the catch-all window subscription and
    /// the event-specific subscription (if registered).
    fn emit_window_event(
        &self,
        specific_key: Option<&str>,
        event_fn: impl Fn(String, String) -> OutgoingEvent,
        window_id: String,
    ) -> io::Result<()> {
        if let Some(tag) = self.core.active_subscriptions.get(SUB_WINDOW_EVENT) {
            emit_event(event_fn(tag.clone(), window_id.clone()))?;
        }
        if let Some(key) = specific_key
            && let Some(tag) = self.core.active_subscriptions.get(key)
        {
            emit_event(event_fn(tag.clone(), window_id))?;
        }
        Ok(())
    }

    pub fn handle_window_event(&self, iced_id: window::Id, evt: window::Event) -> Task<Message> {
        let window_id = self.windows.window_id_for(&iced_id);
        if window_id.is_empty() {
            log::warn!(
                "received window event for unknown iced window {:?}, skipping emission",
                iced_id
            );
            return Task::none();
        }
        // Helper closure: emit and propagate errors uniformly.
        let result: io::Result<()> = (|| {
            match evt {
                window::Event::Opened {
                    position,
                    size,
                    scale_factor,
                } => {
                    if let Some(tag) = self.core.active_subscriptions.get(SUB_WINDOW_EVENT) {
                        let pos = position.map(|p| (p.x, p.y));
                        emit_event(OutgoingEvent::window_opened(
                            tag.clone(),
                            window_id.clone(),
                            pos,
                            size.width,
                            size.height,
                            scale_factor,
                        ))?;
                    }
                    if let Some(tag) = self.core.active_subscriptions.get(SUB_WINDOW_OPEN) {
                        let pos = position.map(|p| (p.x, p.y));
                        emit_event(OutgoingEvent::window_opened(
                            tag.clone(),
                            window_id,
                            pos,
                            size.width,
                            size.height,
                            scale_factor,
                        ))?;
                    }
                }
                window::Event::Closed => {
                    if let Some(tag) = self.core.active_subscriptions.get(SUB_WINDOW_EVENT) {
                        emit_event(OutgoingEvent::window_closed(tag.clone(), window_id))?;
                    }
                }
                window::Event::Moved(point) => {
                    self.emit_window_event(
                        Some(SUB_WINDOW_MOVE),
                        |tag, jid| OutgoingEvent::window_moved(tag, jid, point.x, point.y),
                        window_id,
                    )?;
                }
                window::Event::Resized(size) => {
                    self.emit_window_event(
                        Some(SUB_WINDOW_RESIZE),
                        |tag, jid| OutgoingEvent::window_resized(tag, jid, size.width, size.height),
                        window_id,
                    )?;
                }
                window::Event::Rescaled(factor) => {
                    if let Some(tag) = self.core.active_subscriptions.get(SUB_WINDOW_EVENT) {
                        emit_event(OutgoingEvent::window_rescaled(
                            tag.clone(),
                            window_id,
                            factor,
                        ))?;
                    }
                }
                window::Event::Focused => {
                    self.emit_window_event(
                        Some(SUB_WINDOW_FOCUS),
                        OutgoingEvent::window_focused,
                        window_id,
                    )?;
                }
                window::Event::Unfocused => {
                    self.emit_window_event(
                        Some(SUB_WINDOW_UNFOCUS),
                        OutgoingEvent::window_unfocused,
                        window_id,
                    )?;
                }
                window::Event::FileHovered(path) => {
                    if let Some(tag) = self.core.active_subscriptions.get(SUB_FILE_DROP) {
                        let path_str = path_to_string(path);
                        emit_event(OutgoingEvent::file_hovered(
                            tag.clone(),
                            window_id,
                            path_str,
                        ))?;
                    }
                }
                window::Event::FileDropped(path) => {
                    if let Some(tag) = self.core.active_subscriptions.get(SUB_FILE_DROP) {
                        let path_str = path_to_string(path);
                        emit_event(OutgoingEvent::file_dropped(
                            tag.clone(),
                            window_id,
                            path_str,
                        ))?;
                    }
                }
                window::Event::FilesHoveredLeft => {
                    if let Some(tag) = self.core.active_subscriptions.get(SUB_FILE_DROP) {
                        emit_event(OutgoingEvent::files_hovered_left(tag.clone(), window_id))?;
                    }
                }
                window::Event::CloseRequested => {
                    // Handled via close_requests() subscription separately.
                }
                window::Event::RedrawRequested(_) => {
                    // Handled via animation_frame subscription separately.
                }
            }
            Ok(())
        })();
        if let Err(e) = result {
            log::error!("write error: {e}");
            return iced::exit();
        }
        Task::none()
    }
}
