use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single node in the UI tree.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeNode {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub props: Value,
    #[serde(default)]
    pub children: Vec<TreeNode>,
}

/// A single patch operation applied incrementally to the retained tree.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PatchOp {
    pub op: String,
    pub path: Vec<usize>,
    #[serde(flatten)]
    pub rest: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // TreeNode deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn tree_node_full() {
        let json = r#"{"id":"root","type":"column","props":{"spacing":10},"children":[{"id":"c1","type":"text","props":{"content":"hi"},"children":[]}]}"#;
        let node: TreeNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.id, "root");
        assert_eq!(node.type_name, "column");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].id, "c1");
        assert_eq!(node.props["spacing"], 10);
    }

    #[test]
    fn tree_node_defaults_props_and_children() {
        let json = r#"{"id":"x","type":"text"}"#;
        let node: TreeNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.id, "x");
        assert_eq!(node.type_name, "text");
        assert!(node.children.is_empty());
    }

    #[test]
    fn tree_node_deeply_nested() {
        let json = r#"{"id":"a","type":"column","children":[{"id":"b","type":"row","children":[{"id":"c","type":"text"}]}]}"#;
        let node: TreeNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.children[0].children[0].id, "c");
    }

    // -----------------------------------------------------------------------
    // PatchOp deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn patch_op_replace_node() {
        let json = r#"{"op":"replace_node","path":[1,2],"node":{"id":"n","type":"text"}}"#;
        let op: PatchOp = serde_json::from_str(json).unwrap();
        assert_eq!(op.op, "replace_node");
        assert_eq!(op.path, vec![1, 2]);
        assert!(op.rest.get("node").is_some());
    }

    #[test]
    fn patch_op_update_props() {
        let json = r#"{"op":"update_props","path":[0],"props":{"color":"red"}}"#;
        let op: PatchOp = serde_json::from_str(json).unwrap();
        assert_eq!(op.op, "update_props");
        assert_eq!(op.rest["props"]["color"], "red");
    }

    #[test]
    fn patch_op_insert_child() {
        let json =
            r#"{"op":"insert_child","path":[],"index":0,"node":{"id":"new","type":"button"}}"#;
        let op: PatchOp = serde_json::from_str(json).unwrap();
        assert_eq!(op.op, "insert_child");
        assert!(op.path.is_empty());
        assert_eq!(op.rest["index"], 0);
    }

    #[test]
    fn patch_op_remove_child() {
        let json = r#"{"op":"remove_child","path":[0],"index":1}"#;
        let op: PatchOp = serde_json::from_str(json).unwrap();
        assert_eq!(op.op, "remove_child");
        assert_eq!(op.rest["index"], 1);
    }
}
