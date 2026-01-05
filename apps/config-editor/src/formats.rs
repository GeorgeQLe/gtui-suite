//! Configuration file format handling.

use serde_json::Value as JsonValue;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Json,
    Unknown,
}

impl ConfigFormat {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => ConfigFormat::Toml,
            Some("json") => ConfigFormat::Json,
            _ => ConfigFormat::Unknown,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ConfigFormat::Toml => "TOML",
            ConfigFormat::Json => "JSON",
            ConfigFormat::Unknown => "Text",
        }
    }
}

/// A config value node for tree display.
#[derive(Debug, Clone)]
pub struct ConfigNode {
    pub key: String,
    pub value: ConfigValue,
    pub path: Vec<String>,
    pub depth: usize,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigNode>),
    Table(Vec<ConfigNode>),
    Null,
}

impl ConfigValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            ConfigValue::String(_) => "string",
            ConfigValue::Integer(_) => "integer",
            ConfigValue::Float(_) => "float",
            ConfigValue::Boolean(_) => "boolean",
            ConfigValue::Array(_) => "array",
            ConfigValue::Table(_) => "table",
            ConfigValue::Null => "null",
        }
    }

    pub fn display_value(&self) -> String {
        match self {
            ConfigValue::String(s) => format!("\"{}\"", s),
            ConfigValue::Integer(i) => i.to_string(),
            ConfigValue::Float(f) => f.to_string(),
            ConfigValue::Boolean(b) => b.to_string(),
            ConfigValue::Array(arr) => format!("[{} items]", arr.len()),
            ConfigValue::Table(tbl) => format!("{{{} keys}}", tbl.len()),
            ConfigValue::Null => "null".to_string(),
        }
    }

    pub fn is_container(&self) -> bool {
        matches!(self, ConfigValue::Array(_) | ConfigValue::Table(_))
    }
}

pub fn parse_toml(content: &str) -> Result<ConfigNode, String> {
    let value: toml::Value = toml::from_str(content).map_err(|e| e.to_string())?;
    Ok(toml_to_node("root", &value, vec![], 0))
}

fn toml_to_node(key: &str, value: &toml::Value, path: Vec<String>, depth: usize) -> ConfigNode {
    let mut current_path = path.clone();
    if !key.is_empty() && key != "root" {
        current_path.push(key.to_string());
    }

    let config_value = match value {
        toml::Value::String(s) => ConfigValue::String(s.clone()),
        toml::Value::Integer(i) => ConfigValue::Integer(*i),
        toml::Value::Float(f) => ConfigValue::Float(*f),
        toml::Value::Boolean(b) => ConfigValue::Boolean(*b),
        toml::Value::Datetime(d) => ConfigValue::String(d.to_string()),
        toml::Value::Array(arr) => {
            let children: Vec<ConfigNode> = arr
                .iter()
                .enumerate()
                .map(|(i, v)| toml_to_node(&i.to_string(), v, current_path.clone(), depth + 1))
                .collect();
            ConfigValue::Array(children)
        }
        toml::Value::Table(tbl) => {
            let children: Vec<ConfigNode> = tbl
                .iter()
                .map(|(k, v)| toml_to_node(k, v, current_path.clone(), depth + 1))
                .collect();
            ConfigValue::Table(children)
        }
    };

    ConfigNode {
        key: key.to_string(),
        value: config_value,
        path: current_path,
        depth,
        expanded: depth < 2,
    }
}

pub fn parse_json(content: &str) -> Result<ConfigNode, String> {
    let value: JsonValue = serde_json::from_str(content).map_err(|e| e.to_string())?;
    Ok(json_to_node("root", &value, vec![], 0))
}

fn json_to_node(key: &str, value: &JsonValue, path: Vec<String>, depth: usize) -> ConfigNode {
    let mut current_path = path.clone();
    if !key.is_empty() && key != "root" {
        current_path.push(key.to_string());
    }

    let config_value = match value {
        JsonValue::Null => ConfigValue::Null,
        JsonValue::Bool(b) => ConfigValue::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                ConfigValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                ConfigValue::Float(f)
            } else {
                ConfigValue::String(n.to_string())
            }
        }
        JsonValue::String(s) => ConfigValue::String(s.clone()),
        JsonValue::Array(arr) => {
            let children: Vec<ConfigNode> = arr
                .iter()
                .enumerate()
                .map(|(i, v)| json_to_node(&i.to_string(), v, current_path.clone(), depth + 1))
                .collect();
            ConfigValue::Array(children)
        }
        JsonValue::Object(obj) => {
            let children: Vec<ConfigNode> = obj
                .iter()
                .map(|(k, v)| json_to_node(k, v, current_path.clone(), depth + 1))
                .collect();
            ConfigValue::Table(children)
        }
    };

    ConfigNode {
        key: key.to_string(),
        value: config_value,
        path: current_path,
        depth,
        expanded: depth < 2,
    }
}

pub fn validate_format(content: &str, format: ConfigFormat) -> Result<(), String> {
    match format {
        ConfigFormat::Toml => {
            toml::from_str::<toml::Value>(content).map_err(|e| e.to_string())?;
            Ok(())
        }
        ConfigFormat::Json => {
            serde_json::from_str::<JsonValue>(content).map_err(|e| e.to_string())?;
            Ok(())
        }
        ConfigFormat::Unknown => Ok(()),
    }
}
