# jf - LLM-Optimized JSON Formatter

[![Crates.io](https://img.shields.io/crates/v/llm_json_formatter.svg)](https://crates.io/crates/llm_json_formatter)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A JSON formatting tool optimized for LLM consumption, balancing readability and token efficiency.

## Why jf?

When feeding JSON data to LLMs, you face a trade-off:

- **Pretty format**: High readability but wastes tokens on whitespace
- **Compact format**: Token efficient but hard to read

**jf** solves this by intelligently detecting "entity objects" (like user records, order items) and keeping them on single lines while expanding the overall structure. This achieves **~40% token reduction** compared to pretty format while maintaining excellent readability.

## Features

- **Entity-Aware Formatting**: Auto-detect "entity objects" in arrays and keep them on single lines
- **Schema Statistics**: Infer entities based on P90 length analysis
- **LLM-Assisted Labeling**: Generate prompts for LLM to identify entities, with manual override support
- **Smart Key Sorting**: Alphabetic or weighted sorting (id/name first)
- **Multiple Format Modes**: Smart / Compact / Pretty
- **Schema Extraction**: Generate compact type schemas from JSON data

## Installation

### From Source

```bash
git clone https://github.com/luw2007/llm_json_formatter.git
cd llm_json_formatter
cargo build --release
# Binary: target/release/jf
```

### From Crates.io

```bash
cargo install llm_json_formatter
```

## Quick Start

```bash
# Quick format - shortcut (automatically uses format command)
jf data.json

# Format multiple files
jf file1.json file2.json file3.json

# Pipe input - shortcut (automatically uses format command)
echo '{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}' | jf

# Default formatting (auto-detect entities) - explicit command
echo '{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}' | jf format

# Output:
# {
#   "users": [
#     {"id":1,"name":"Alice"},
#     {"id":2,"name":"Bob"}
#   ]
# }
```

## Commands

### format - Format JSON

```bash
jf format [OPTIONS] [-i <INPUT>] [-o <OUTPUT>]
```

**Options**:

| Option                      | Default    | Description                                      |
| --------------------------- | ---------- | ------------------------------------------------ |
| `-m, --mode`                | smart      | Format mode: `smart` / `compact` / `pretty`      |
| `--sort`                    | alphabetic | Key sorting: `alphabetic` / `smart`              |
| `--indent`                  | 2          | Indentation spaces                               |
| `--inline-limit`            | 80         | Max line length for inline objects in Smart mode |
| `--array-item-inline-limit` | 2048       | Max line length for array items (entities)       |
| `--entity-threshold`        | 2000       | Length threshold for auto-detected entities      |
| `--entities`                | -          | Comma-separated or JSON array of entity paths    |

**Examples**:

```bash
# Quick format (shortcut, defaults to smart mode)
jf data.json

# Compact mode (minimum tokens)
jf format -i data.json --mode compact

# Pretty mode (maximum readability)
jf format -i data.json --mode pretty

# Manually specify entities
jf format -i data.json --entities "users[*],orders[*]"

# JSON array format for entities
jf format -i data.json --entities '["users[*]","orders[*]"]'

# Disable auto entity detection
jf format -i data.json --entity-threshold 0

# Smart key sorting (id/name first)
jf format -i data.json --sort smart
```

### prompt - Generate LLM Prompt

Generate a prompt containing schema structure and samples for LLM to identify "business entities".

```bash
jf prompt [-i <INPUT>]
```

**Example**:

```bash
jf prompt -i data.json
```

**Output**:

```
Analyze the JSON schema below and identify 'Business Entities'...

Schema:
{
  "users": [{...}]
}

Array paths and samples:

Path: users[*]
Samples:
  - {"id":1,"name":"Alice"...
  - {"id":2,"name":"Bob"...

Output ONLY a JSON array of entity paths...
```

### schema - Extract Schema

Extract a compact type schema from JSON data.

```bash
jf schema [-i <INPUT>]
```

**Example**:

```bash
echo '{"users":[{"id":1,"name":"Alice"}],"config":{"debug":true}}' | jf schema
```

**Output**:

```
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

### analyze - Analyze JSON Structure

```bash
jf analyze [-i <INPUT>]
```

**Output**:

```
JSON Analysis:
  Byte Size: 1234 bytes
  Max Depth: 4
  Object Count: 15
  Total Keys: 42
  Array Count: 3
  Max Array Length: 100
```

### search - Path Query

```bash
jf search [-i <INPUT>] -p <PATH>
```

**Example**:

```bash
echo '{"users":[{"id":1,"name":"Alice"}]}' | jf search -p "users[0].name"
```

### paths - List All Paths

```bash
jf paths [-i <INPUT>]
```

## Entity Detection

### Option 1: Automatic Statistical Inference

Based on schema analysis, calculate P90 length for each array path. If P90 ≤ `entity-threshold`, mark as entity.

```bash
# Default threshold 2000
jf format -i data.json

# Stricter (only short objects count as entities)
jf format -i data.json --entity-threshold 100
```

### Option 2: LLM-Assisted Labeling

```bash
# 1. Generate prompt
jf prompt -i data.json > prompt.txt

# 2. Send to LLM, get entity list
# LLM returns: ["users[*]", "orders[*]"]

# 3. Format with labeled entities
jf format -i data.json --entities '["users[*]","orders[*]"]'
```

## Format Mode Comparison

| Mode        | Description                         | Token Efficiency | Readability |
| ----------- | ----------------------------------- | ---------------- | ----------- |
| **Smart**   | Entities inline, structure expanded | ⭐⭐⭐⭐         | ⭐⭐⭐⭐    |
| **Compact** | Fully compressed                    | ⭐⭐⭐⭐⭐       | ⭐          |
| **Pretty**  | Fully expanded                      | ⭐⭐             | ⭐⭐⭐⭐⭐  |

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
llm_json_formatter = "0.1"
```

### Basic Usage

```rust
use llm_json_formatter::{LlmJsonFormatter, Config, FormatMode};

fn main() {
    let config = Config {
        mode: FormatMode::Smart,
        ..Default::default()
    };

    let mut formatter = LlmJsonFormatter::new(config);
    let json = r#"{"users":[{"id":1,"name":"Alice"}]}"#;

    let result = formatter.format(json).unwrap();
    println!("{}", result);
}
```

### Custom Entity Detection

```rust
use llm_json_formatter::{LlmJsonFormatter, Config, FormatMode};
use std::collections::HashSet;

fn main() {
    let mut entities = HashSet::new();
    entities.insert("users[*]".to_string());
    entities.insert("orders[*]".to_string());

    let config = Config {
        mode: FormatMode::Smart,
        entities,
        entity_threshold: 0, // Disable auto detection
        ..Default::default()
    };

    let mut formatter = LlmJsonFormatter::new(config);
    let result = formatter.format(json).unwrap();
}
```

### Generate Schema

```rust
use llm_json_formatter::generate_schema;
use serde_json::Value;

fn main() {
    let json: Value = serde_json::from_str(r#"{"id":1,"name":"Alice"}"#).unwrap();
    let schema = generate_schema(&json, 0);
    println!("{}", schema);
    // Output:
    // {
    //   "id": number
    //   "name": string
    // }
}
```

### Analyze JSON Metadata

```rust
use llm_json_formatter::{LlmJsonFormatter, Config};

fn main() {
    let formatter = LlmJsonFormatter::new(Config::default());
    let metadata = formatter.get_metadata(json).unwrap();

    println!("Size: {} bytes", metadata.byte_size);
    println!("Depth: {}", metadata.depth);
    println!("Objects: {}", metadata.object_count);
}
```

## API Reference

### Config

```rust
pub struct Config {
    pub mode: FormatMode,           // Smart | Compact | Pretty
    pub sort_strategy: SortStrategy, // Alphabetic | Smart
    pub indent: usize,              // Default: 2
    pub inline_limit: usize,        // Default: 80
    pub array_item_inline_limit: usize, // Default: 2048
    pub entity_threshold: usize,    // Default: 2000
    pub entities: HashSet<String>,  // User-specified entity paths
}
```

### FormatMode

- `Smart`: Intelligent formatting with entity detection
- `Compact`: Minimized single-line output
- `Pretty`: Standard indented output

### SortStrategy

- `Alphabetic`: Sort keys alphabetically
- `Smart`: Sort by importance (id/name/type first, \_internal last)

## Testing

Run all tests:

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test cli_tests

# Run with output
cargo test -- --nocapture
```

The test suite includes 17+ integration tests covering:

- ✅ Shortcut commands (single file, multiple files, pipe input)
- ✅ Explicit format commands with various modes
- ✅ All CLI subcommands (analyze, schema, paths, search)
- ✅ Error handling (invalid JSON, nonexistent files)
- ✅ Help and version flags

## Benchmarks

Run benchmarks:

```bash
cargo bench
```

## License

MIT
