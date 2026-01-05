# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-05

### Added

- Initial release
- **Smart Format Mode**: Intelligent formatting with entity detection
  - Auto-detect "entity objects" in arrays based on P90 length analysis
  - Keep entities on single lines while expanding overall structure
  - Configurable `entity-threshold` for auto-detection sensitivity
  - Manual entity specification via `--entities` flag
- **Compact Format Mode**: Minimized single-line output for maximum token efficiency
- **Pretty Format Mode**: Standard indented output for maximum readability
- **Key Sorting Strategies**:
  - `alphabetic`: Sort keys alphabetically (default)
  - `smart`: Sort by importance (id/name/type first, _internal last)
- **Schema Extraction**: Generate compact type schemas from JSON data
  - Automatic map detection for homogeneous object values
  - Merged schema for objects with different field sets
- **LLM Prompt Generation**: Generate prompts for LLM to identify business entities
- **JSON Analysis**: Analyze JSON structure (depth, object count, array stats)
- **Path Operations**:
  - `search`: Query values by JSON path
  - `paths`: List all available paths in JSON
- **CLI Tool** (`jf`):
  - `format`: Format JSON with configurable options
  - `prompt`: Generate LLM prompt for entity identification
  - `schema`: Extract compact schema
  - `analyze`: Analyze JSON structure
  - `search`: Search by path
  - `paths`: List all paths
- **Library API**:
  - `LlmJsonFormatter`: Main formatter with configurable options
  - `Config`: Configuration struct for all formatting options
  - `JsonIndex`: Path-based JSON indexing and search
  - `SchemaStats`: Schema statistics for entity detection
  - `generate_schema()`: Schema extraction function
