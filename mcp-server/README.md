# LLM JSON Formatter MCP Server

This directory contains a Model Context Protocol (MCP) server that exposes the `jf` (LLM JSON Formatter) functionality to AI assistants.

## Tools Provided

- **`format_json`**: Format JSON strings using Smart, Compact, or Pretty modes.
- **`analyze_json`**: Get metadata about a JSON structure (depth, size, keys, etc.).
- **`search_json`**: Search for specific values using JSON paths (e.g., `users[0].name`).
- **`list_paths`**: List all available paths in a JSON object.

## Prerequisities

1.  Node.js installed.
2.  The `jf` binary built in the parent directory (`cargo build --release`).

## Setup

1.  Install dependencies:
    ```bash
    npm install
    ```

2.  Build the TypeScript server:
    ```bash
    npm run build
    ```

## Configuration

To use this with Claude Desktop or other MCP clients, add the following to your configuration file (e.g., `~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "llm-json-formatter": {
      "command": "node",
      "args": [
        "/ABSOLUTE/PATH/TO/llm_json_formatter/mcp-server/dist/index.js"
      ]
    }
  }
}
```

Replace `/ABSOLUTE/PATH/TO/...` with the actual absolute path to this directory.
