use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatterError {
    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, FormatterError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub byte_size: usize,
    pub depth: usize,
    pub array_count: usize,
    pub max_array_len: usize,
    pub object_count: usize,
    pub total_keys: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortStrategy {
    #[default]
    Alphabetic,
    Smart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatMode {
    #[default]
    Smart,
    Compact,
    Pretty,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: FormatMode,
    pub sort_strategy: SortStrategy,
    pub indent: usize,
    pub inline_limit: usize,
    pub array_item_inline_limit: usize,
    pub entity_threshold: usize, // Length threshold for auto-detected entities
    pub entities: HashSet<String>, // User-specified entity paths
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: FormatMode::default(),
            sort_strategy: SortStrategy::default(),
            indent: 2,
            inline_limit: 80,
            array_item_inline_limit: 2048,
            entity_threshold: 2000,
            entities: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathInfo {
    pub value_type: ValueType,
    pub preview: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

impl From<&Value> for ValueType {
    fn from(v: &Value) -> Self {
        match v {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Boolean,
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::Array(_) => ValueType::Array,
            Value::Object(_) => ValueType::Object,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct JsonIndex {
    path_map: HashMap<String, PathInfo>,
}

impl JsonIndex {
    pub fn build(json: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(json)?;
        let mut path_map = HashMap::new();
        Self::traverse(&value, String::new(), &mut path_map);
        Ok(Self { path_map })
    }

    fn traverse(val: &Value, path: String, map: &mut HashMap<String, PathInfo>) {
        let info = PathInfo {
            value_type: ValueType::from(val),
            preview: Self::preview(val),
        };
        if !path.is_empty() {
            map.insert(path.clone(), info);
        }

        match val {
            Value::Object(obj) => {
                for (k, v) in obj {
                    let new_path = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", path, k)
                    };
                    Self::traverse(v, new_path, map);
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, i);
                    Self::traverse(v, new_path, map);
                }
            }
            _ => {}
        }
    }

    fn preview(val: &Value) -> String {
        let s = serde_json::to_string(val).unwrap_or_default();
        if s.len() > 100 {
            format!("{}...", &s[..100])
        } else {
            s
        }
    }

    pub fn search(&self, path: &str) -> Option<&PathInfo> {
        self.path_map.get(path)
    }

    pub fn list_paths(&self) -> Vec<&String> {
        let mut paths: Vec<_> = self.path_map.keys().collect();
        paths.sort();
        paths
    }
}

#[derive(Debug, Clone)]
struct SchemaNode {
    lengths: Vec<usize>,
    samples: Vec<String>,
    is_object: bool,
}

#[derive(Debug, Clone, Default)]
struct SchemaStats {
    stats: HashMap<String, SchemaNode>,
}

impl SchemaStats {
    fn analyze(value: &Value) -> Self {
        let mut stats = HashMap::new();
        Self::traverse(value, String::new(), &mut stats);
        Self { stats }
    }

    fn traverse(val: &Value, path: String, stats: &mut HashMap<String, SchemaNode>) {
        let len = serde_json::to_string(val).map(|s| s.len()).unwrap_or(0);
        let is_object = matches!(val, Value::Object(_));
        let entry = stats.entry(path.clone()).or_insert_with(|| SchemaNode {
            lengths: Vec::new(),
            samples: Vec::new(),
            is_object,
        });
        entry.lengths.push(len);
        if entry.samples.len() < 3 {
            entry.samples.push(Self::preview(val));
        }

        match val {
            Value::Object(obj) => {
                // Check if this object is a "map" (all values have same base type)
                let is_map = if obj.len() >= 2 {
                    let base_types: std::collections::HashSet<_> = obj.values()
                        .map(|v| match v {
                            Value::Null => "null",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Array(_) => "array",
                            Value::Object(_) => "object",
                        })
                        .collect();
                    base_types.len() == 1
                } else {
                    false
                };
                
                for (k, v) in obj {
                    let new_path = if is_map {
                        format!("{}[*]", path)
                    } else if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", path, k)
                    };
                    Self::traverse(v, new_path, stats);
                }
            }
            Value::Array(arr) => {
                let new_path = format!("{}[*]", path);
                for v in arr {
                    Self::traverse(v, new_path.clone(), stats);
                }
            }
            _ => {}
        }
    }

    fn preview(val: &Value) -> String {
        let s = serde_json::to_string(val).unwrap_or_default();
        if s.len() > 50 {
            format!("{}...", &s[..50])
        } else {
            s
        }
    }

    fn get_p90_length(&self, path: &str) -> usize {
        if let Some(node) = self.stats.get(path) {
            if node.lengths.is_empty() {
                return 0;
            }
            let mut sorted = node.lengths.clone();
            sorted.sort_unstable();
            let idx = (sorted.len() as f64 * 0.9) as usize;
            // Clamp index to valid range
            let idx = idx.min(sorted.len() - 1);
            sorted[idx]
        } else {
            0
        }
    }

    fn get_samples(&self) -> Vec<(String, Vec<String>)> {
        let mut result = Vec::new();
        for (path, node) in &self.stats {
            if path.contains("[*]") && node.is_object {
                result.push((path.clone(), node.samples.clone()));
            }
        }
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

fn get_type_signature(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", get_type_signature(&arr[0]))
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let mut sigs: Vec<_> = obj.iter()
                    .map(|(k, v)| format!("{}:{}", k, get_type_signature(v)))
                    .collect();
                sigs.sort();
                format!("{{{}}}", sigs.join(","))
            }
        }
    }
}

fn merge_objects<'a>(objects: impl Iterator<Item = &'a Map<String, Value>>) -> Map<String, Value> {
    let mut merged: Map<String, Value> = Map::new();
    for obj in objects {
        for (k, v) in obj {
            merged.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    merged
}

fn get_base_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn generate_schema(value: &Value, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let item_schema = generate_schema(&arr[0], indent + 1);
                if item_schema.contains('\n') {
                    format!("[\n{}{}\n{}]", "  ".repeat(indent + 1), item_schema, prefix)
                } else {
                    format!("[{}]", item_schema)
                }
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let mut type_groups: HashMap<String, Vec<(&String, &Value)>> = HashMap::new();
                for (k, v) in obj {
                    let sig = get_type_signature(v);
                    type_groups.entry(sig).or_default().push((k, v));
                }
                
                // Case 1: All values have exactly same type signature
                if type_groups.len() == 1 && obj.len() >= 2 {
                    let (_, items) = type_groups.iter().next().unwrap();
                    let (_, sample_val) = items[0];
                    let val_schema = generate_schema(sample_val, indent + 1);
                    return format!("map[string]{}", val_schema);
                }
                
                // Case 2: All values are objects (but with different fields) - merge them
                let all_objects = obj.values().all(|v| matches!(v, Value::Object(_)));
                if all_objects && obj.len() >= 2 {
                    let objects_iter = obj.values().filter_map(|v| {
                        if let Value::Object(o) = v { Some(o) } else { None }
                    });
                    let merged = merge_objects(objects_iter);
                    let merged_value = Value::Object(merged);
                    let val_schema = generate_schema(&merged_value, indent + 1);
                    return format!("map[string]{}", val_schema);
                }
                
                // Case 3: All values are same primitive type
                let base_types: HashSet<_> = obj.values().map(get_base_type).collect();
                if base_types.len() == 1 && obj.len() >= 2 {
                    let base_type = base_types.into_iter().next().unwrap();
                    return format!("map[string]{}", base_type);
                }
                
                // Default: enumerate all keys
                let mut lines = vec!["{".to_string()];
                let inner_prefix = "  ".repeat(indent + 1);
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                for key in keys {
                    let val = &obj[key];
                    let val_schema = generate_schema(val, indent + 1);
                    lines.push(format!("{}\"{}\": {}", inner_prefix, key, val_schema));
                }
                lines.push(format!("{}}}", prefix));
                lines.join("\n")
            }
        }
    }
}

pub struct LlmJsonFormatter {
    config: Config,
    schema_stats: Option<SchemaStats>,
}

impl LlmJsonFormatter {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            schema_stats: None,
        }
    }

    pub fn analyze(&self, value: &Value) -> Metadata {
        let byte_size = serde_json::to_string(value).map(|s| s.len()).unwrap_or(0);
        let mut depth = 0;
        let mut array_count = 0;
        let mut max_array_len = 0;
        let mut object_count = 0;
        let mut total_keys = 0;

        Self::collect_stats(
            value,
            0,
            &mut depth,
            &mut array_count,
            &mut max_array_len,
            &mut object_count,
            &mut total_keys,
        );

        Metadata {
            byte_size,
            depth,
            array_count,
            max_array_len,
            object_count,
            total_keys,
        }
    }

    fn collect_stats(
        value: &Value,
        current_depth: usize,
        max_depth: &mut usize,
        array_count: &mut usize,
        max_array_len: &mut usize,
        object_count: &mut usize,
        total_keys: &mut usize,
    ) {
        *max_depth = (*max_depth).max(current_depth);

        match value {
            Value::Object(obj) => {
                *object_count += 1;
                *total_keys += obj.len();
                for v in obj.values() {
                    Self::collect_stats(
                        v,
                        current_depth + 1,
                        max_depth,
                        array_count,
                        max_array_len,
                        object_count,
                        total_keys,
                    );
                }
            }
            Value::Array(arr) => {
                *array_count += 1;
                *max_array_len = (*max_array_len).max(arr.len());
                for v in arr {
                    Self::collect_stats(
                        v,
                        current_depth + 1,
                        max_depth,
                        array_count,
                        max_array_len,
                        object_count,
                        total_keys,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn format(&mut self, json: &str) -> Result<String> {
        let value: Value = serde_json::from_str(json)?;
        let sorted_value = self.deep_sort_keys(&value);

        // Analyze schema stats for smart formatting
        if self.config.mode == FormatMode::Smart {
            self.schema_stats = Some(SchemaStats::analyze(&sorted_value));
        }

        match self.config.mode {
            FormatMode::Compact => Ok(serde_json::to_string(&sorted_value)?),
            FormatMode::Pretty => Ok(serde_json::to_string_pretty(&sorted_value)?),
            FormatMode::Smart => Ok(self.format_smart(&sorted_value, 0, String::new())),
        }
    }

    pub fn generate_prompt(&mut self, json: &str) -> Result<String> {
        let value: Value = serde_json::from_str(json)?;
        let stats = SchemaStats::analyze(&value);
        let samples = stats.get_samples();

        let mut prompt = String::new();
        prompt.push_str("Analyze the JSON schema below and identify 'Business Entities' - array items that represent meaningful data records (e.g., user, order, product) suitable for single-line display.\n\n");
        
        prompt.push_str("Schema:\n");
        prompt.push_str(&generate_schema(&value, 0));
        prompt.push_str("\n\n");

        prompt.push_str("Array paths and samples:\n\n");

        for (path, sample_list) in samples {
            prompt.push_str(&format!("Path: {}\n", path));
            prompt.push_str("Samples:\n");
            for s in sample_list {
                prompt.push_str(&format!("  - {}\n", s));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("Output ONLY a JSON array of entity paths. No explanation, no markdown, no code blocks.\n");
        prompt.push_str("Example output: [\"users[*]\",\"orders[*]\"]\n");
        prompt.push_str("If no entities found, output: []\n");

        Ok(prompt)
    }

    fn format_smart(&self, value: &Value, depth: usize, path: String) -> String {
        let is_forced_entity = self.config.entities.contains(&path);

        let is_auto_entity = if let Some(stats) = &self.schema_stats {
            let p90 = stats.get_p90_length(&path);
            path.ends_with("[*]") && p90 > 0 && p90 <= self.config.entity_threshold
        } else {
            false
        };

        let is_simple_value = matches!(value, Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_));

        let is_simple_map = match value {
            Value::Object(obj) if obj.len() >= 2 => {
                obj.values().all(|v| matches!(v, Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)))
            }
            _ => false,
        };

        let limit = if is_forced_entity || is_auto_entity || is_simple_value || is_simple_map {
            self.config.array_item_inline_limit
        } else {
            self.config.inline_limit
        };

        let compact = serde_json::to_string(value).unwrap_or_default();
        if compact.len() <= limit {
            return compact;
        }

        // 6. If too long, expand
        match value {
            Value::Array(arr) => {
                if arr.is_empty() {
                    return "[]".to_string();
                }
                let mut s = String::from("[\n");
                let new_path_pattern = format!("{}[*]", path);
                
                for (i, item) in arr.iter().enumerate() {
                    s.push_str(&self.indent(depth + 1));
                    s.push_str(&self.format_smart(item, depth + 1, new_path_pattern.clone()));
                    if i < arr.len() - 1 {
                        s.push_str(",");
                    }
                    s.push_str("\n");
                }
                s.push_str(&self.indent(depth));
                s.push_str("]");
                s
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    return "{}".to_string();
                }
                
                let is_map = if obj.len() >= 2 {
                    let base_types: HashSet<_> = obj.values().map(get_base_type).collect();
                    base_types.len() == 1
                } else {
                    false
                };
                
                let mut s = String::from("{\n");
                for (i, (k, v)) in obj.iter().enumerate() {
                    let new_path = if is_map {
                        format!("{}[*]", path)
                    } else if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", path, k)
                    };
                    
                    s.push_str(&self.indent(depth + 1));
                    s.push_str(&format!("\"{}\": ", k));
                    s.push_str(&self.format_smart(v, depth + 1, new_path));
                    if i < obj.len() - 1 {
                        s.push_str(",");
                    }
                    s.push_str("\n");
                }
                s.push_str(&self.indent(depth));
                s.push_str("}");
                s
            }
            _ => compact,
        }
    }

    fn indent(&self, depth: usize) -> String {
        " ".repeat(depth * self.config.indent)
    }

    fn deep_sort_keys(&self, value: &Value) -> Value {
        match value {
            Value::Object(obj) => {
                let sorted = self.sort_keys(obj);
                Value::Object(
                    sorted
                        .into_iter()
                        .map(|(k, v)| (k, self.deep_sort_keys(&v)))
                        .collect(),
                )
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.deep_sort_keys(v)).collect()),
            _ => value.clone(),
        }
    }

    fn sort_keys(&self, obj: &Map<String, Value>) -> Map<String, Value> {
        let mut items: Vec<_> = obj.iter().collect();

        match self.config.sort_strategy {
            SortStrategy::Alphabetic => {
                items.sort_by(|a, b| a.0.cmp(b.0));
            }
            SortStrategy::Smart => {
                items.sort_by(|a, b| {
                    let weight_a = Self::calculate_weight(a.0);
                    let weight_b = Self::calculate_weight(b.0);
                    match weight_b.cmp(&weight_a) {
                        std::cmp::Ordering::Equal => a.0.cmp(b.0),
                        other => other,
                    }
                });
            }
        }

        items.into_iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    fn calculate_weight(key: &str) -> i32 {
        let mut weight = 0;

        if matches!(key, "id" | "name" | "type" | "status" | "title" | "key" | "value") {
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

    pub fn get_metadata(&self, json: &str) -> Result<Metadata> {
        let value: Value = serde_json::from_str(json)?;
        Ok(self.analyze(&value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_format_small_object() {
        let mut formatter = LlmJsonFormatter::new(Config::default());
        let input = r#"{"name":"Alice","age":30}"#;
        let result = formatter.format(input).unwrap();
        assert_eq!(result, r#"{"age":30,"name":"Alice"}"#);
    }

    #[test]
    fn test_smart_format_nested_array() {
        let mut formatter = LlmJsonFormatter::new(Config {
            inline_limit: 30, // Low limit to force structure expansion
            entity_threshold: 2000, // High enough to cover inner objects
            ..Default::default()
        });
        // Short enough to be inline
        let input = r#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}"#;
        let result = formatter.format(input).unwrap();
        
        // "users[*]" P90 length is ~23.
        // 23 <= 2000, so it's an entity.
        // Entity limit is 2048.
        // So inner objects should be compact.
        // Outer structure expands because "users" array itself > 30?
        // Wait, "users" path is "users". "users[*]" is the item.
        // The array itself is formatted with path "users".
        // Its length is > 30 (input is ~56).
        // So it expands.
        
        let expected = r#"{
  "users": [
    {"id":1,"name":"Alice"},
    {"id":2,"name":"Bob"}
  ]
}"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_forced_entities() {
        let mut entities = HashSet::new();
        entities.insert("users[*]".to_string());
        
        let mut formatter = LlmJsonFormatter::new(Config {
            inline_limit: 10, // Very low
            entities,
            ..Default::default()
        });
        
        let input = r#"{"users":[{"id":1,"name":"LongNameAlice"},{"id":2,"name":"LongNameBob"}]}"#;
        let result = formatter.format(input).unwrap();
        
        // Even though inline_limit is 10, entities should be compact (using default array_item_inline_limit 2048)
        let expected = r#"{
  "users": [
    {"id":1,"name":"LongNameAlice"},
    {"id":2,"name":"LongNameBob"}
  ]
}"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_smart_sort() {
        let mut formatter = LlmJsonFormatter::new(Config {
            sort_strategy: SortStrategy::Smart,
            ..Default::default()
        });
        let input = r#"{"zzz":1,"name":"Alice","_internal":2,"id":100}"#;
        let result = formatter.format(input).unwrap();

        // Should be compact because it's short
        assert_eq!(result, r#"{"id":100,"name":"Alice","zzz":1,"_internal":2}"#);
    }
}
