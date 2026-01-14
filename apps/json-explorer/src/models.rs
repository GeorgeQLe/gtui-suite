use serde_json::Value;

#[derive(Debug, Clone)]
pub struct JsonNode {
    pub key: Option<String>,
    pub value: Value,
    pub path: String,
    pub depth: usize,
    pub expanded: bool,
    pub index: Option<usize>,
}

impl JsonNode {
    pub fn from_value(value: Value) -> Self {
        Self {
            key: None,
            value,
            path: "$".to_string(),
            depth: 0,
            expanded: true,
            index: None,
        }
    }

    pub fn value_type(&self) -> ValueType {
        match &self.value {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Boolean,
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    ValueType::Integer
                } else {
                    ValueType::Float
                }
            }
            Value::String(_) => ValueType::String,
            Value::Array(_) => ValueType::Array,
            Value::Object(_) => ValueType::Object,
        }
    }

    pub fn child_count(&self) -> usize {
        match &self.value {
            Value::Array(arr) => arr.len(),
            Value::Object(obj) => obj.len(),
            _ => 0,
        }
    }

    pub fn is_expandable(&self) -> bool {
        matches!(&self.value, Value::Array(_) | Value::Object(_))
    }

    pub fn display_key(&self) -> String {
        if let Some(ref key) = self.key {
            key.clone()
        } else if let Some(idx) = self.index {
            format!("[{}]", idx)
        } else {
            "root".to_string()
        }
    }

    pub fn display_value(&self) -> String {
        match &self.value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => {
                if s.len() > 50 {
                    format!("\"{}...\"", &s[..47])
                } else {
                    format!("\"{}\"", s)
                }
            }
            Value::Array(arr) => format!("[{} items]", arr.len()),
            Value::Object(obj) => format!("{{{}  keys}}", obj.len()),
        }
    }

    pub fn children(&self) -> Vec<JsonNode> {
        match &self.value {
            Value::Array(arr) => arr
                .iter()
                .enumerate()
                .map(|(i, v)| JsonNode {
                    key: None,
                    value: v.clone(),
                    path: format!("{}[{}]", self.path, i),
                    depth: self.depth + 1,
                    expanded: false,
                    index: Some(i),
                })
                .collect(),
            Value::Object(obj) => obj
                .iter()
                .map(|(k, v)| JsonNode {
                    key: Some(k.clone()),
                    value: v.clone(),
                    path: format!("{}.{}", self.path, k),
                    depth: self.depth + 1,
                    expanded: false,
                    index: None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    Array,
    Object,
}

impl ValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValueType::Null => "null",
            ValueType::Boolean => "boolean",
            ValueType::Integer => "integer",
            ValueType::Float => "float",
            ValueType::String => "string",
            ValueType::Array => "array",
            ValueType::Object => "object",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ValueType::Null => "∅",
            ValueType::Boolean => "◉",
            ValueType::Integer => "#",
            ValueType::Float => "~",
            ValueType::String => "\"",
            ValueType::Array => "[]",
            ValueType::Object => "{}",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub node: JsonNode,
    pub visible: bool,
}

pub fn flatten_tree(root: &JsonNode, nodes: &mut Vec<FlatNode>) {
    nodes.push(FlatNode {
        node: root.clone(),
        visible: true,
    });

    if root.expanded {
        for child in root.children() {
            flatten_tree(&child, nodes);
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsonStats {
    pub total_keys: usize,
    pub max_depth: usize,
    pub string_count: usize,
    pub number_count: usize,
    pub boolean_count: usize,
    pub null_count: usize,
    pub array_count: usize,
    pub object_count: usize,
}

impl JsonStats {
    pub fn compute(value: &Value) -> Self {
        let mut stats = Self {
            total_keys: 0,
            max_depth: 0,
            string_count: 0,
            number_count: 0,
            boolean_count: 0,
            null_count: 0,
            array_count: 0,
            object_count: 0,
        };
        Self::count_recursive(value, 0, &mut stats);
        stats
    }

    fn count_recursive(value: &Value, depth: usize, stats: &mut JsonStats) {
        stats.max_depth = stats.max_depth.max(depth);

        match value {
            Value::Null => stats.null_count += 1,
            Value::Bool(_) => stats.boolean_count += 1,
            Value::Number(_) => stats.number_count += 1,
            Value::String(_) => stats.string_count += 1,
            Value::Array(arr) => {
                stats.array_count += 1;
                for item in arr {
                    Self::count_recursive(item, depth + 1, stats);
                }
            }
            Value::Object(obj) => {
                stats.object_count += 1;
                stats.total_keys += obj.len();
                for (_, v) in obj {
                    Self::count_recursive(v, depth + 1, stats);
                }
            }
        }
    }
}
