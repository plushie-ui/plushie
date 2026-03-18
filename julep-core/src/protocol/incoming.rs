use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::types::{PatchOp, TreeNode};

/// Protocol version number. Sent in the `hello` handshake message on startup
/// and checked against the value the host embeds in Settings.
pub const PROTOCOL_VERSION: u32 = 1;

/// Messages sent from the host to the renderer over stdin.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingMessage {
    Snapshot {
        tree: TreeNode,
    },
    Patch {
        ops: Vec<PatchOp>,
    },
    EffectRequest {
        id: String,
        kind: String,
        payload: Value,
    },
    WidgetOp {
        op: String,
        #[serde(default)]
        payload: Value,
    },
    SubscriptionRegister {
        kind: String,
        tag: String,
    },
    SubscriptionUnregister {
        kind: String,
    },
    WindowOp {
        op: String,
        window_id: String,
        #[serde(default)]
        settings: Value,
    },
    Settings {
        settings: Value,
    },
    /// Query the current tree or find a widget.
    Query {
        id: String,
        target: String,
        #[serde(default)]
        selector: Value,
    },
    /// Interact with a widget (click, type, etc.)
    Interact {
        id: String,
        action: String,
        #[serde(default)]
        selector: Value,
        #[serde(default)]
        payload: Value,
    },
    /// Capture a structural tree snapshot (hash of JSON tree).
    #[allow(dead_code)]
    SnapshotCapture {
        id: String,
        name: String,
        #[serde(default)]
        theme: Value,
        #[serde(default)]
        viewport: Value,
    },
    /// Capture a pixel screenshot (GPU-rendered RGBA data).
    #[allow(dead_code)]
    ScreenshotCapture {
        id: String,
        name: String,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
    },
    /// Reset the app state.
    Reset {
        id: String,
    },
    /// Image operation (create, update, delete in-memory image handles).
    ///
    /// Binary fields (`data`, `pixels`) accept either raw bytes (from msgpack)
    /// or base64-encoded strings (from JSON). The custom deserializer handles both.
    ImageOp {
        op: String,
        handle: String,
        #[serde(default, deserialize_with = "deserialize_binary_field")]
        data: Option<Vec<u8>>,
        #[serde(default, deserialize_with = "deserialize_binary_field")]
        pixels: Option<Vec<u8>>,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
    },
    /// A single extension command pushed to a native extension widget.
    /// Bypasses the normal tree update / diff / patch cycle.
    ExtensionCommand {
        node_id: String,
        op: String,
        #[serde(default)]
        payload: Value,
    },
    /// A batch of extension commands processed in one cycle.
    ExtensionCommandBatch {
        commands: Vec<ExtensionCommandItem>,
    },
    /// Advance the animation clock by one frame (headless/test mode).
    /// Emits an `animation_frame` event if `on_animation_frame` is subscribed.
    AdvanceFrame {
        timestamp: u64,
    },
}

/// A single item within an `ExtensionCommandBatch`.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionCommandItem {
    pub node_id: String,
    pub op: String,
    #[serde(default)]
    pub payload: Value,
}

/// Response to an effect request, written to stdout as JSONL.
#[derive(Debug, Serialize)]
pub struct EffectResponse {
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub id: String,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl EffectResponse {
    pub fn ok(id: String, result: Value) -> Self {
        Self {
            message_type: "effect_response",
            id,
            status: "ok",
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: String, reason: String) -> Self {
        Self {
            message_type: "effect_response",
            id,
            status: "error",
            result: None,
            error: Some(reason),
        }
    }

    pub fn unsupported(id: String) -> Self {
        Self::error(id, "unsupported".to_string())
    }

    /// The user cancelled the operation (e.g. closed a file dialog).
    /// Distinct from `error` -- cancellation is a normal user action,
    /// not a failure.
    pub fn cancelled(id: String) -> Self {
        Self {
            message_type: "effect_response",
            id,
            status: "cancelled",
            result: None,
            error: None,
        }
    }
}

/// Response to a Query message.
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub id: String,
    pub target: String,
    pub data: Value,
}

impl QueryResponse {
    pub fn new(id: String, target: String, data: Value) -> Self {
        Self {
            message_type: "query_response",
            id,
            target,
            data,
        }
    }
}

/// Response to an Interact message.
#[derive(Debug, Serialize)]
pub struct InteractResponse {
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub id: String,
    pub events: Vec<Value>,
}

impl InteractResponse {
    pub fn new(id: String, events: Vec<Value>) -> Self {
        Self {
            message_type: "interact_response",
            id,
            events,
        }
    }
}

/// Response to a SnapshotCapture message.
///
/// Snapshots capture structural tree data (hash of JSON tree). No pixel data.
/// For pixel data, see the `screenshot_response` message type.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct SnapshotCaptureResponse {
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub id: String,
    pub name: String,
    pub hash: String,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
impl SnapshotCaptureResponse {
    pub fn new(id: String, name: String, hash: String, width: u32, height: u32) -> Self {
        Self {
            message_type: "snapshot_response",
            id,
            name,
            hash,
            width,
            height,
        }
    }
}

/// Empty screenshot response for backends that cannot capture pixels.
///
/// Used by headless mode. The full backend uses a format-aware emit function
/// instead (native binary for msgpack, base64 for JSON).
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ScreenshotResponseEmpty {
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub id: String,
    pub name: String,
    pub hash: String,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
impl ScreenshotResponseEmpty {
    pub fn new(id: String, name: String) -> Self {
        Self {
            message_type: "screenshot_response",
            id,
            name,
            hash: String::new(),
            width: 0,
            height: 0,
        }
    }
}

/// Emit a screenshot response with RGBA pixel data to stdout.
///
/// Uses native msgpack binary (`rmpv::Value::Binary`) for pixel data when the
/// wire codec is MsgPack, and base64-encoded string when JSON. This avoids the
/// ~33% size overhead of base64 on the hot path.
///
/// Shared between test-mode (main.rs) and headless (headless.rs).
#[allow(dead_code)]
pub fn emit_screenshot_response(
    id: &str,
    name: &str,
    hash: &str,
    width: u32,
    height: u32,
    rgba_bytes: &[u8],
) {
    use std::io::Write;

    let codec = crate::codec::Codec::get_global();
    let bytes = match codec {
        crate::codec::Codec::MsgPack => {
            use rmpv::Value as RmpvValue;

            let mut entries = vec![
                (
                    RmpvValue::String("type".into()),
                    RmpvValue::String("screenshot_response".into()),
                ),
                (RmpvValue::String("id".into()), RmpvValue::String(id.into())),
                (
                    RmpvValue::String("name".into()),
                    RmpvValue::String(name.into()),
                ),
                (
                    RmpvValue::String("hash".into()),
                    RmpvValue::String(hash.into()),
                ),
                (
                    RmpvValue::String("width".into()),
                    RmpvValue::Integer((width as i64).into()),
                ),
                (
                    RmpvValue::String("height".into()),
                    RmpvValue::Integer((height as i64).into()),
                ),
            ];
            if !rgba_bytes.is_empty() {
                entries.push((
                    RmpvValue::String("rgba".into()),
                    RmpvValue::Binary(rgba_bytes.to_vec()),
                ));
            }
            let map = RmpvValue::Map(entries);

            let mut payload = Vec::new();
            if let Err(e) = rmpv::encode::write_value(&mut payload, &map) {
                log::error!("msgpack encode screenshot: {e}");
                return;
            }
            let len = match u32::try_from(payload.len()) {
                Ok(n) => n,
                Err(_) => {
                    log::error!(
                        "screenshot payload exceeds u32::MAX ({} bytes)",
                        payload.len()
                    );
                    return;
                }
            };
            let mut bytes = Vec::with_capacity(4 + payload.len());
            bytes.extend_from_slice(&len.to_be_bytes());
            bytes.extend_from_slice(&payload);
            bytes
        }
        crate::codec::Codec::Json => {
            use base64::Engine;

            let mut map = serde_json::json!({
                "type": "screenshot_response",
                "id": id,
                "name": name,
                "hash": hash,
                "width": width,
                "height": height,
            });
            if !rgba_bytes.is_empty() {
                let b64 = base64::engine::general_purpose::STANDARD.encode(rgba_bytes);
                map["rgba"] = serde_json::Value::String(b64);
            }
            match serde_json::to_vec(&map) {
                Ok(mut bytes) => {
                    bytes.push(b'\n');
                    bytes
                }
                Err(e) => {
                    log::error!("json encode screenshot: {e}");
                    return;
                }
            }
        }
    };

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    if let Err(e) = handle.write_all(&bytes) {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            log::error!("stdout broken pipe -- shutting down");
            std::process::exit(0);
        }
        log::error!("stdout write error: {e}");
        return;
    }
    if let Err(e) = handle.flush() {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            log::error!("stdout broken pipe on flush -- shutting down");
            std::process::exit(0);
        }
        log::error!("stdout flush error: {e}");
    }
}

/// Response to a Reset message.
#[derive(Debug, Serialize)]
pub struct ResetResponse {
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub id: String,
    pub status: &'static str,
}

impl ResetResponse {
    pub fn ok(id: String) -> Self {
        Self {
            message_type: "reset_response",
            id,
            status: "ok",
        }
    }
}

// ---------------------------------------------------------------------------
// Binary field deserialization (handles both raw bytes and base64 strings)
// ---------------------------------------------------------------------------

/// Deserializes a binary field that may arrive as:
/// - Raw bytes (msgpack binary type, via rmpv path)
/// - Base64-encoded string (JSON path)
/// - null / absent (returns None)
///
/// When the codec's rmpv-based decode extracts binary fields and injects them
/// as `serde_json::Value::Array` of u8 values, serde picks them up as Vec<u8>.
/// When the field arrives as a base64 string (JSON mode), we decode it here.
fn deserialize_binary_field<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let val: Option<Value> = Option::deserialize(deserializer)?;
    match val {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        // Base64 string (JSON mode)
        Some(Value::String(s)) => {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD
                .decode(&s)
                .map(Some)
                .map_err(|e| D::Error::custom(format!("base64 decode: {e}")))
        }
        // Array of u8 values (injected by rmpv binary extraction)
        Some(Value::Array(arr)) => {
            let bytes: Result<Vec<u8>, _> = arr
                .into_iter()
                .map(|v| {
                    v.as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or_else(|| D::Error::custom("expected u8 in binary array"))
                })
                .collect();
            bytes.map(Some)
        }
        Some(other) => Err(D::Error::custom(format!(
            "expected string, array, or null for binary field, got {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // -----------------------------------------------------------------------
    // IncomingMessage deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn deserialize_snapshot() {
        let json =
            r#"{"type":"snapshot","tree":{"id":"root","type":"column","props":{},"children":[]}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::Snapshot { tree } => {
                assert_eq!(tree.id, "root");
                assert_eq!(tree.type_name, "column");
            }
            _ => panic!("expected Snapshot"),
        }
    }

    #[test]
    fn deserialize_snapshot_nested_tree() {
        let json = r#"{"type":"snapshot","tree":{"id":"root","type":"column","props":{"spacing":10},"children":[{"id":"c1","type":"text","props":{"content":"hello"},"children":[]}]}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::Snapshot { tree } => {
                assert_eq!(tree.children.len(), 1);
                assert_eq!(tree.children[0].id, "c1");
                assert_eq!(tree.children[0].type_name, "text");
                assert_eq!(tree.props["spacing"], 10);
            }
            _ => panic!("expected Snapshot"),
        }
    }

    #[test]
    fn deserialize_patch_replace_node() {
        let json = r#"{"type":"patch","ops":[{"op":"replace_node","path":[0],"node":{"id":"x","type":"text","props":{},"children":[]}}]}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::Patch { ops } => {
                assert_eq!(ops.len(), 1);
                assert_eq!(ops[0].op, "replace_node");
                assert_eq!(ops[0].path, vec![0]);
                assert!(ops[0].rest.get("node").is_some());
            }
            _ => panic!("expected Patch"),
        }
    }

    #[test]
    fn deserialize_patch_multiple_ops() {
        let json = r#"{"type":"patch","ops":[{"op":"update_props","path":[0],"props":{"color":"red"}},{"op":"remove_child","path":[],"index":2}]}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::Patch { ops } => {
                assert_eq!(ops.len(), 2);
                assert_eq!(ops[0].op, "update_props");
                assert_eq!(ops[1].op, "remove_child");
            }
            _ => panic!("expected Patch"),
        }
    }

    #[test]
    fn deserialize_effect_request() {
        let json = r#"{"type":"effect_request","id":"e1","kind":"clipboard_read","payload":{}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::EffectRequest { id, kind, payload } => {
                assert_eq!(id, "e1");
                assert_eq!(kind, "clipboard_read");
                assert!(payload.is_object());
            }
            _ => panic!("expected EffectRequest"),
        }
    }

    #[test]
    fn deserialize_effect_request_with_payload() {
        let json = r#"{"type":"effect_request","id":"e2","kind":"clipboard_write","payload":{"text":"copied"}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::EffectRequest { id, kind, payload } => {
                assert_eq!(id, "e2");
                assert_eq!(kind, "clipboard_write");
                assert_eq!(payload["text"], "copied");
            }
            _ => panic!("expected EffectRequest"),
        }
    }

    #[test]
    fn deserialize_widget_op() {
        let json = r#"{"type":"widget_op","op":"focus","payload":{"target":"input1"}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::WidgetOp { op, payload } => {
                assert_eq!(op, "focus");
                assert_eq!(payload["target"], "input1");
            }
            _ => panic!("expected WidgetOp"),
        }
    }

    #[test]
    fn deserialize_widget_op_no_payload() {
        let json = r#"{"type":"widget_op","op":"blur"}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::WidgetOp { op, payload } => {
                assert_eq!(op, "blur");
                assert!(payload.is_null());
            }
            _ => panic!("expected WidgetOp"),
        }
    }

    #[test]
    fn deserialize_subscription_register() {
        let json = r#"{"type":"subscription_register","kind":"on_key_press","tag":"keys"}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::SubscriptionRegister { kind, tag } => {
                assert_eq!(kind, "on_key_press");
                assert_eq!(tag, "keys");
            }
            _ => panic!("expected SubscriptionRegister"),
        }
    }

    #[test]
    fn deserialize_subscription_unregister() {
        let json = r#"{"type":"subscription_unregister","kind":"on_key_press"}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::SubscriptionUnregister { kind } => {
                assert_eq!(kind, "on_key_press");
            }
            _ => panic!("expected SubscriptionUnregister"),
        }
    }

    #[test]
    fn deserialize_settings() {
        let json = r#"{"type":"settings","settings":{"default_text_size":18}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::Settings { settings } => {
                assert_eq!(settings["default_text_size"], 18);
            }
            _ => panic!("expected Settings"),
        }
    }

    #[test]
    fn deserialize_window_op() {
        let json = r#"{"type":"window_op","op":"resize","window_id":"main","settings":{"width":800,"height":600}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::WindowOp {
                op,
                window_id,
                settings,
            } => {
                assert_eq!(op, "resize");
                assert_eq!(window_id, "main");
                assert_eq!(settings["width"], 800);
                assert_eq!(settings["height"], 600);
            }
            _ => panic!("expected WindowOp"),
        }
    }

    #[test]
    fn deserialize_window_op_no_settings() {
        let json = r#"{"type":"window_op","op":"close","window_id":"popup"}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::WindowOp {
                op,
                window_id,
                settings,
            } => {
                assert_eq!(op, "close");
                assert_eq!(window_id, "popup");
                assert!(settings.is_null());
            }
            _ => panic!("expected WindowOp"),
        }
    }

    #[test]
    fn deserialize_malformed_json_missing_field() {
        let json = r#"{"type":"snapshot"}"#;
        let result = serde_json::from_str::<IncomingMessage>(json);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_unknown_type_tag() {
        let json = r#"{"type":"bogus_message","data":42}"#;
        let result = serde_json::from_str::<IncomingMessage>(json);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_invalid_json_syntax() {
        let json = r#"{"type":"snapshot",,,}"#;
        let result = serde_json::from_str::<IncomingMessage>(json);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // EffectResponse serialization
    // -----------------------------------------------------------------------

    #[test]
    fn effect_response_ok() {
        let resp = EffectResponse::ok("e1".to_string(), json!("clipboard content"));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["type"], "effect_response");
        assert_eq!(json["id"], "e1");
        assert_eq!(json["status"], "ok");
        assert_eq!(json["result"], "clipboard content");
        assert!(json.get("error").is_none());
    }

    #[test]
    fn effect_response_error() {
        let resp = EffectResponse::error("e2".to_string(), "not found".to_string());
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["type"], "effect_response");
        assert_eq!(json["id"], "e2");
        assert_eq!(json["status"], "error");
        assert_eq!(json["error"], "not found");
        assert!(json.get("result").is_none());
    }

    #[test]
    fn effect_response_unsupported() {
        let resp = EffectResponse::unsupported("e3".to_string());
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["status"], "error");
        assert_eq!(json["error"], "unsupported");
    }

    #[test]
    fn effect_response_ok_with_object_result() {
        let resp = EffectResponse::ok("e4".to_string(), json!({"files": ["/a.txt", "/b.txt"]}));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["result"]["files"][0], "/a.txt");
        assert_eq!(json["result"]["files"][1], "/b.txt");
    }

    // -----------------------------------------------------------------------
    // ExtensionCommand deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn extension_command_deserializes() {
        let json = r#"{"type":"extension_command","node_id":"term-1","op":"write","payload":{"data":"hello"}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::ExtensionCommand {
                node_id,
                op,
                payload,
            } => {
                assert_eq!(node_id, "term-1");
                assert_eq!(op, "write");
                assert_eq!(payload["data"], "hello");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn extension_command_batch_deserializes() {
        let json = r#"{"type":"extension_command_batch","commands":[{"node_id":"term-1","op":"write","payload":{"data":"a"}},{"node_id":"log-1","op":"append","payload":{"line":"x"}}]}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::ExtensionCommandBatch { commands } => {
                assert_eq!(commands.len(), 2);
                assert_eq!(commands[0].node_id, "term-1");
                assert_eq!(commands[1].op, "append");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn extension_command_with_default_payload() {
        let json = r#"{"type":"extension_command","node_id":"ext-1","op":"reset"}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::ExtensionCommand { payload, .. } => {
                assert!(payload.is_null());
            }
            _ => panic!("wrong variant"),
        }
    }
}
