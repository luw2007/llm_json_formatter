# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-03-07

### Fixed

- **Homebrew Distribution**: Prepare a patch release for the refreshed GitHub Release artifacts
  - Bump the crate and release metadata to `0.1.3`
  - Align the Homebrew formula template version with the next release
  - Avoid `brew install jf` checksum failures caused by stale `0.1.2` release packaging

## [0.1.2] - 2026-03-04

### Changed

- **Release Infrastructure**: Add GitHub Releases packaging and Homebrew binary distribution
  - Add `scripts/package_release.sh` for automated release packaging
  - Add Homebrew formula template for binary installation
  - Add comprehensive release documentation in `docs/release.md`
  - Update README with Homebrew and binary installation instructions

### Fixed

- Fix git commit history to use correct author email (luw2007@gmail.com)

## [0.1.1] - 2025-01-06

### Added

- **Shortcut Command**: Support direct file path as argument for quick formatting
  - Usage: `jf data.json` automatically executes `jf format -i data.json`
  - Support multiple files: `jf file1.json file2.json file3.json`
  - Defaults to smart mode with auto entity detection
- **Pipe Input Shortcut**: Support pipe input without explicit format command
  - Usage: `echo '{}' | jf` automatically executes `jf format`
  - Simplifies common pipe input operations
- **Comprehensive Test Suite**: Added 17+ integration tests
  - Tests cover all shortcut commands (file, pipe input)
  - Tests cover all CLI subcommands and modes
  - Tests include error handling scenarios

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
  - `smart`: Sort by importance (id/name/type first, \_internal last)
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
