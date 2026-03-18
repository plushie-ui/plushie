//! Wire protocol types for host-renderer communication.
//!
//! [`IncomingMessage`] is deserialized from the host. [`OutgoingEvent`]
//! and response types are serialized back. The transport (stdin/stdout,
//! socket, test harness) is handled by the binary crate, not here.

mod incoming;
mod outgoing;
mod types;

/// Protocol version number. Sent in the `hello` handshake message on startup
/// and checked against the value the host embeds in Settings.
pub const PROTOCOL_VERSION: u32 = 1;

pub use incoming::{ExtensionCommandItem, IncomingMessage};
pub use outgoing::{
    EffectResponse, InteractResponse, KeyModifiers, OutgoingEvent, QueryResponse, ResetResponse,
    SnapshotCaptureResponse,
};
pub use types::{PatchOp, TreeNode};
