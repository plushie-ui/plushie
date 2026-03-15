// test_mode.rs - Helpers for --test mode
//
// When running with --test, the regular iced::daemon runs normally but the App
// also handles Query/Interact/SnapshotCapture/Reset messages from stdin
// (instead of passing them to Core::apply where they'd hit the catch-all).
//
// The actual protocol logic lives in test_protocol.rs (shared with headless).
// This module provides the is_test_message() gate and re-exports the handlers.

pub mod test_helpers {
    use julep_core::protocol::IncomingMessage;

    /// Check if a message is a test-mode message (Query, Interact, etc.)
    pub fn is_test_message(msg: &IncomingMessage) -> bool {
        matches!(
            msg,
            IncomingMessage::Query { .. }
                | IncomingMessage::Interact { .. }
                | IncomingMessage::SnapshotCapture { .. }
                | IncomingMessage::ScreenshotCapture { .. }
                | IncomingMessage::Reset { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::test_helpers;
    use julep_core::protocol::{IncomingMessage, TreeNode};

    fn make_tree_node(id: &str, type_name: &str) -> TreeNode {
        TreeNode {
            id: id.to_string(),
            type_name: type_name.to_string(),
            props: Value::Object(Default::default()),
            children: vec![],
        }
    }

    // -- is_test_message --

    #[test]
    fn is_test_message_returns_true_for_query() {
        let msg = IncomingMessage::Query {
            id: "q1".to_string(),
            target: "tree".to_string(),
            selector: Value::Null,
        };
        assert!(test_helpers::is_test_message(&msg));
    }

    #[test]
    fn is_test_message_returns_true_for_interact() {
        let msg = IncomingMessage::Interact {
            id: "i1".to_string(),
            action: "click".to_string(),
            selector: Value::Null,
            payload: Value::Null,
        };
        assert!(test_helpers::is_test_message(&msg));
    }

    #[test]
    fn is_test_message_returns_true_for_reset() {
        let msg = IncomingMessage::Reset {
            id: "r1".to_string(),
        };
        assert!(test_helpers::is_test_message(&msg));
    }

    #[test]
    fn is_test_message_returns_true_for_snapshot_capture() {
        let msg = IncomingMessage::SnapshotCapture {
            id: "sc1".to_string(),
            name: "my_snap".to_string(),
            theme: Value::Null,
            viewport: Value::Null,
        };
        assert!(test_helpers::is_test_message(&msg));
    }

    #[test]
    fn is_test_message_returns_false_for_snapshot() {
        let msg = IncomingMessage::Snapshot {
            tree: make_tree_node("root", "column"),
        };
        assert!(!test_helpers::is_test_message(&msg));
    }

    #[test]
    fn is_test_message_returns_false_for_patch() {
        let msg = IncomingMessage::Patch { ops: vec![] };
        assert!(!test_helpers::is_test_message(&msg));
    }

    #[test]
    fn is_test_message_returns_false_for_settings() {
        let msg = IncomingMessage::Settings {
            settings: serde_json::json!({}),
        };
        assert!(!test_helpers::is_test_message(&msg));
    }

    // -- handle_query --

    #[test]
    fn query_response_has_correct_structure() {
        use julep_core::protocol::QueryResponse;

        let resp = QueryResponse::new(
            "q42".to_string(),
            "tree".to_string(),
            serde_json::json!({"id": "root"}),
        );
        assert_eq!(resp.id, "q42");
        assert_eq!(resp.target, "tree");
        assert_eq!(resp.message_type, "query_response");
        assert_eq!(resp.data, serde_json::json!({"id": "root"}));
    }

    #[test]
    fn query_response_null_data_when_tree_empty() {
        use julep_core::protocol::QueryResponse;

        let resp = QueryResponse::new("q1".to_string(), "tree".to_string(), Value::Null);
        assert_eq!(resp.data, Value::Null);
    }

    // -- screenshot protocol --

    #[test]
    fn is_test_message_returns_true_for_screenshot_capture() {
        let msg = IncomingMessage::ScreenshotCapture {
            id: "sc1".to_string(),
            name: "test_shot".to_string(),
            width: None,
            height: None,
        };
        assert!(test_helpers::is_test_message(&msg));
    }

    #[test]
    fn snapshot_capture_response_has_no_rgba_field() {
        use julep_core::protocol::SnapshotCaptureResponse;

        let resp = SnapshotCaptureResponse::new(
            "s1".to_string(),
            "snap".to_string(),
            "abc123".to_string(),
            100,
            200,
        );
        let json = serde_json::to_value(&resp).unwrap();
        assert!(
            json.get("rgba_base64").is_none(),
            "SnapshotCaptureResponse should not have an rgba_base64 field"
        );
    }

    #[test]
    fn screenshot_response_empty_has_correct_structure() {
        use julep_core::protocol::ScreenshotResponseEmpty;

        let resp = ScreenshotResponseEmpty::new("sc1".to_string(), "test_shot".to_string());
        assert_eq!(resp.message_type, "screenshot_response");
        assert_eq!(resp.hash, "");
        assert_eq!(resp.width, 0);
        assert_eq!(resp.height, 0);

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json.get("type").unwrap(), "screenshot_response");
    }
}
