# LLM JSON Formatter Technical Specification

## 1. Requirements Analysis

### Core Objective
Optimize JSON display for LLM consumption, finding the optimal balance between **readability** and **token efficiency**.

### Key Trade-off
```
Readability ←→ Token Cost
     ↓              ↓
LLM Understanding   Context Limit
```

**Solution:** Entity-aware smart formatting - detect "business entities" (like user records, order items) and keep them on single lines while expanding the overall structure.

---

## 2. System Architecture

### Overall Flow

```
Input JSON
    ↓
[1] Parse & Sort Keys
    ↓
[2] Schema Statistics (Smart mode)
    ↓
[3] Entity Detection
    ├── Auto: P90 length analysis
    └── Manual: User-specified paths
    ↓
[4] Smart Formatting
    ├── Entities → Single line (up to array_item_inline_limit)
    └── Structure → Expanded with indentation
    ↓
[5] Formatted Output
```

### Core Modules

```rust
┌─────────────────────────────────────┐
│         LlmJsonFormatter            │
├─────────────────────────────────────┤
│ + format(json) → String             │ ← Main formatting
│ + generate_prompt(json) → String    │ ← LLM entity labeling
│ + get_metadata(json) → Metadata     │ ← Structure analysis
└─────────────────────────────────────┘
           ↓
    ┌──────┴──────┐
    ↓             ↓
FormatMode     SortStrategy
- Smart        - Alphabetic
- Compact      - Smart
- Pretty
```

---

## 3. Data Structures

### Config

```rust
pub struct Config {
    pub mode: FormatMode,              // Smart | Compact | Pretty
    pub sort_strategy: SortStrategy,   // Alphabetic | Smart
    pub indent: usize,                 // Default: 2
    pub inline_limit: usize,           // Default: 80 (for normal objects)
    pub array_item_inline_limit: usize, // Default: 2048 (for entities)
    pub entity_threshold: usize,       // Default: 2000 (P90 length threshold)
    pub entities: HashSet<String>,     // User-specified entity paths
}
```

### Metadata

```rust
pub struct Metadata {
    pub byte_size: usize,        // Serialized byte count
    pub depth: usize,            // Maximum nesting depth
    pub array_count: usize,      // Total array count
    pub max_array_len: usize,    // Longest array length
    pub object_count: usize,     // Total object count
    pub total_keys: usize,       // Total key count
}
```

### FormatMode

```rust
pub enum FormatMode {
    Smart,    // Entity-aware formatting (default)
    Compact,  // Minimized single-line output
    Pretty,   // Standard indented output
}
```

### SortStrategy

```rust
pub enum SortStrategy {
    Alphabetic,  // Sort keys alphabetically (default)
    Smart,       // Sort by importance (id/name first, _internal last)
}
```

---

## 4. Format Mode Details

### Mode 1: Compact

**Goal:** Minimum tokens, no whitespace

```rust
fn format_compact(value: &Value) -> String {
    serde_json::to_string(&sorted_value)
}
```

**Example:**
```json
{"age":30,"city":"NYC","name":"Alice"}
```

---

### Mode 2: Pretty

**Goal:** Maximum readability, standard indentation

```rust
fn format_pretty(value: &Value) -> String {
    serde_json::to_string_pretty(&sorted_value)
}
```

**Example:**
```json
{
  "age": 30,
  "city": "NYC",
  "name": "Alice"
}
```

---

### Mode 3: Smart (Default)

**Goal:** Balance readability and token efficiency via entity detection

#### Entity Detection Algorithm

```rust
fn format_smart(&self, value: &Value, depth: usize, path: String) -> String {
    // 1. Check if path is a forced entity (user-specified)
    let is_forced_entity = self.config.entities.contains(&path);

    // 2. Check if path is an auto-detected entity (P90 analysis)
    let is_auto_entity = if let Some(stats) = &self.schema_stats {
        let p90 = stats.get_p90_length(&path);
        path.ends_with("[*]") && p90 > 0 && p90 <= self.config.entity_threshold
    } else {
        false
    };

    // 3. Determine inline limit
    let limit = if is_forced_entity || is_auto_entity {
        self.config.array_item_inline_limit  // 2048
    } else {
        self.config.inline_limit  // 80
    };

    // 4. If compact form fits within limit, use it
    let compact = serde_json::to_string(value);
    if compact.len() <= limit {
        return compact;
    }

    // 5. Otherwise, expand with indentation
    // ... recursive formatting
}
```

#### P90 Length Analysis

For each array path pattern (e.g., `users[*]`), collect all item lengths and calculate P90:

```rust
fn get_p90_length(&self, path: &str) -> usize {
    let mut sorted = node.lengths.clone();
    sorted.sort_unstable();
    let idx = (sorted.len() as f64 * 0.9) as usize;
    sorted[idx.min(sorted.len() - 1)]
}
```

If P90 ≤ `entity_threshold` (default 2000), the path is marked as an entity.

**Example:**
```json
// Input
{
  "users": [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob", "email": "bob@example.com"}
  ]
}

// Output (Smart mode)
{
  "users": [
    {"email":"alice@example.com","id":1,"name":"Alice"},
    {"email":"bob@example.com","id":2,"name":"Bob"}
  ]
}
```

**Token Savings:** ~40% compared to pretty format

---

## 5. Key Sorting Strategy

### Alphabetic (Default)

Sort keys in alphabetical order:

```json
{"age":30,"city":"NYC","name":"Alice"}
```

### Smart

Sort by importance weight:

```rust
fn calculate_weight(key: &str) -> i32 {
    let mut weight = 0;

    // High priority keys
    if matches!(key, "id" | "name" | "type" | "status" | "title" | "key" | "value") {
        weight += 100;
    }

    // Low priority: internal fields
    if key.starts_with('_') || key.contains("internal") {
        weight -= 50;
    }

    // Low priority: debug fields
    if key.contains("debug") || key.contains("test") {
        weight -= 30;
    }

    weight
}
```

**Example:**
```json
// Before (alphabetic)
{"_internal":2,"id":100,"name":"Alice","zzz":1}

// After (smart)
{"id":100,"name":"Alice","zzz":1,"_internal":2}
```

---

## 6. Path Query (JsonIndex)

### Building Index

```rust
pub struct JsonIndex {
    path_map: HashMap<String, PathInfo>,
}

pub struct PathInfo {
    pub value_type: ValueType,
    pub preview: String,  // First 100 chars
}

impl JsonIndex {
    pub fn build(json: &str) -> Result<Self>;
    pub fn search(&self, path: &str) -> Option<&PathInfo>;
    pub fn list_paths(&self) -> Vec<&String>;
}
```

### Path Format

- Object keys: `users`, `config.debug`
- Array indices: `users[0]`, `users[0].name`

**Example:**
```rust
let index = JsonIndex::build(json)?;

// O(1) lookup
if let Some(info) = index.search("users[0].email") {
    println!("Type: {:?}", info.value_type);
    println!("Preview: {}", info.preview);
}
```

---

## 7. Schema Extraction

Generate compact type schema from JSON data:

```rust
pub fn generate_schema(value: &Value, indent: usize) -> String;
```

### Type Mapping

| JSON Type | Schema Output |
|-----------|---------------|
| null | `null` |
| boolean | `boolean` |
| number | `number` |
| string | `string` |
| array | `[item_type]` |
| object | `{ "key": type, ... }` |
| map-like object | `map[string]value_type` |

### Map Detection

Objects with ≥2 keys where all values have the same type are treated as maps:

```json
// Input
{"en": "Hello", "zh": "你好", "ja": "こんにちは"}

// Schema
map[string]string
```

**Example:**
```json
// Input
{
  "users": [{"id": 1, "name": "Alice"}],
  "config": {"debug": true}
}

// Schema Output
{
  "config": {
    "debug": boolean
  }
  "users": [
    {
      "id": number
      "name": string
    }
  ]
}
```

---

## 8. LLM Prompt Generation

Generate prompts for LLM to identify business entities:

```rust
pub fn generate_prompt(&mut self, json: &str) -> Result<String>;
```

**Output Format:**
```
Analyze the JSON schema below and identify 'Business Entities' - array items 
that represent meaningful data records (e.g., user, order, product) suitable 
for single-line display.

Schema:
{
  "users": [{
    "id": number
    "name": string
  }]
}

Array paths and samples:

Path: users[*]
Samples:
  - {"id":1,"name":"Alice"...
  - {"id":2,"name":"Bob"...

Output ONLY a JSON array of entity paths. No explanation, no markdown.
Example output: ["users[*]","orders[*]"]
If no entities found, output: []
```

---

## 9. CLI Commands

| Command | Description |
|---------|-------------|
| `jf format` | Format JSON with smart/compact/pretty modes |
| `jf prompt` | Generate LLM prompt for entity identification |
| `jf schema` | Extract compact type schema |
| `jf analyze` | Show JSON structure metadata |
| `jf search` | Query value by JSON path |
| `jf paths` | List all available paths |

### Format Options

| Option | Default | Description |
|--------|---------|-------------|
| `--mode` | smart | Format mode: smart/compact/pretty |
| `--sort` | alphabetic | Key sorting: alphabetic/smart |
| `--indent` | 2 | Indentation spaces |
| `--inline-limit` | 80 | Max line length for inline objects |
| `--array-item-inline-limit` | 2048 | Max line length for entities |
| `--entity-threshold` | 2000 | P90 length threshold for auto-detection |
| `--entities` | - | Comma-separated or JSON array of entity paths |

---

## 10. Library API

```rust
use llm_json_formatter::{LlmJsonFormatter, Config, FormatMode};

// Basic usage
let mut formatter = LlmJsonFormatter::new(Config::default());
let result = formatter.format(json)?;

// Custom entity detection
let config = Config {
    mode: FormatMode::Smart,
    entities: HashSet::from(["users[*]".to_string()]),
    entity_threshold: 0,  // Disable auto-detection
    ..Default::default()
};
let mut formatter = LlmJsonFormatter::new(config);
let result = formatter.format(json)?;

// Generate schema
use llm_json_formatter::generate_schema;
let schema = generate_schema(&value, 0);

// Analyze metadata
let metadata = formatter.get_metadata(json)?;
println!("Depth: {}, Objects: {}", metadata.depth, metadata.object_count);

// Path query
use llm_json_formatter::JsonIndex;
let index = JsonIndex::build(json)?;
let paths = index.list_paths();
if let Some(info) = index.search("users[0].name") {
    println!("{:?}", info);
}
```

---

## 11. Format Mode Comparison

| Mode | Description | Token Efficiency | Readability |
|------|-------------|------------------|-------------|
| **Smart** | Entities inline, structure expanded | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Compact** | Fully compressed | ⭐⭐⭐⭐⭐ | ⭐ |
| **Pretty** | Fully expanded | ⭐⭐ | ⭐⭐⭐⭐⭐ |

### Token Savings Example

For a JSON with 100 user records:

| Mode | Approximate Tokens | Savings vs Pretty |
|------|-------------------|-------------------|
| Pretty | 1000 | - |
| Smart | 600 | 40% |
| Compact | 500 | 50% |
