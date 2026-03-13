use serde_json::{Map, Value};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapKind {
    Primitive(&'static str),
    ObjectMerged,
}

pub fn type_signature(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", type_signature(&arr[0]))
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let mut sigs: Vec<_> = obj
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, type_signature(v)))
                    .collect();
                sigs.sort();
                format!("{{{}}}", sigs.join(","))
            }
        }
    }
}

pub fn merge_objects<'a>(objects: impl Iterator<Item = &'a Map<String, Value>>) -> Map<String, Value> {
    let mut merged: Map<String, Value> = Map::new();
    for obj in objects {
        for (k, v) in obj {
            merged.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    merged
}

pub fn get_base_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn is_map_like(obj: &Map<String, Value>) -> Option<MapKind> {
    if obj.len() < 2 {
        return None;
    }

    if obj.values().all(|v| matches!(v, Value::Object(_))) {
        return Some(MapKind::ObjectMerged);
    }

    let base_types: HashSet<_> = obj.values().map(get_base_type).collect();
    if base_types.len() == 1 {
        return Some(MapKind::Primitive(base_types.into_iter().next().unwrap()));
    }

    None
}

pub fn key_weight(key: &str) -> i32 {
    let mut weight = 0;

    if matches!(
        key,
        "id" | "name" | "type" | "status" | "title" | "key" | "value"
    ) {
        weight += 100;
    }

    if key.starts_with('_') || key.contains("internal") {
        weight -= 50;
    }

    if key.contains("debug") || key.contains("test") {
        weight -= 30;
    }

    weight
}
