use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct TreeDef {
    pub root: NodeDef,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct NodeDef {
    pub node_type: String,
    #[serde(default)]
    pub children: Vec<NodeDef>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub ports: std::collections::HashMap<String, String>,
}

pub fn parse_tree(json: &str) -> Result<TreeDef, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_tree() {
        let json = r#"{
            "root": {
                "node_type": "Sequence",
                "children": [
                    { "node_type": "Condition", "ports": { "in": "blackboard_key" }, "config": { "delay": 10 } }
                ]
            }
        }"#;
        let tree = parse_tree(json).unwrap();
        assert_eq!(tree.root.node_type, "Sequence");
        assert_eq!(tree.root.children.len(), 1);
        assert_eq!(tree.root.children[0].node_type, "Condition");
        assert_eq!(tree.root.children[0].ports.get("in").unwrap(), "blackboard_key");
        assert!(tree.root.children[0].config.is_some());
    }
}
