//! Wire protocol types for host-renderer communication.
//!
//! The renderer reads [`IncomingMessage`]s from stdin and writes
//! [`OutgoingEvent`]s (plus response structs) to stdout.

mod incoming;
mod outgoing;
mod types;

pub use incoming::{
    EffectResponse, ExtensionCommandItem, IncomingMessage, InteractResponse, PROTOCOL_VERSION,
    QueryResponse, ResetResponse, ScreenshotResponseEmpty, SnapshotCaptureResponse,
    emit_screenshot_response,
};
pub use outgoing::{KeyModifiers, OutgoingEvent};
pub use types::{PatchOp, TreeNode};
