import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  Tool,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Path to the compiled rust binary
// Assumes running from dist/index.js, so binary is at ../../../target/release/jf
const JF_BINARY_PATH = path.resolve(__dirname, "../../../target/release/jf");

// Define tools
const TOOLS: Tool[] = [
  {
    name: "format_json",
    description: "Format JSON using smart, compact, or pretty modes. 'smart' mode balances readability and token usage.",
    inputSchema: zodToJsonSchema(
      z.object({
        json: z.string().describe("The JSON string to format"),
        mode: z.enum(["smart", "compact", "pretty"]).optional().default("smart").describe("Formatting mode"),
        indent: z.number().optional().default(2).describe("Indentation spaces (default: 2)"),
        sort: z.enum(["alphabetic", "smart"]).optional().default("alphabetic").describe("Key sorting strategy"),
      })
    ),
  },
  {
    name: "analyze_json",
    description: "Analyze JSON structure to get metadata like depth, object count, etc.",
    inputSchema: zodToJsonSchema(
      z.object({
        json: z.string().describe("The JSON string to analyze"),
      })
    ),
  },
  {
    name: "search_json",
    description: "Search for a value in JSON using a path (e.g. 'users[0].name').",
    inputSchema: zodToJsonSchema(
      z.object({
        json: z.string().describe("The JSON string to search in"),
        path: z.string().describe("The JSON path to search for"),
      })
    ),
  },
  {
    name: "list_paths",
    description: "List all available paths in the JSON object.",
    inputSchema: zodToJsonSchema(
      z.object({
        json: z.string().describe("The JSON string to list paths from"),
      })
    ),
  },
];

// Helper to convert Zod schema to JSON Schema
function zodToJsonSchema(schema: z.ZodType<any>): any {
  // Simple conversion for basic types used here
  // For production, might want to use zod-to-json-schema package, 
  // but for this simple case, we can construct it or use a simplified approach.
  // Actually, MCP SDK expects standard JSON Schema. 
  // Let's implement a minimal converter or just define schemas directly if Zod is too complex to map manually without a lib.
  // Using zod-to-json-schema is better, but I didn't install it.
  // I'll manually define the JSON schemas to avoid extra dependencies for now, 
  // or I can quickly install zod-to-json-schema. 
  // Let's stick to manual definition for simplicity in this generated code to minimize steps,
  // matching the Zod definitions above.
  
  if (schema instanceof z.ZodObject) {
      const shape = schema.shape;
      const properties: Record<string, any> = {};
      const required: string[] = [];
      
      for (const [key, value] of Object.entries(shape)) {
          properties[key] = zodToJsonSchema(value as z.ZodType<any>);
          if (!(value as any).isOptional()) {
              required.push(key);
          }
      }
      return { type: "object", properties, required };
  }
  if (schema instanceof z.ZodString) return { type: "string" };
  if (schema instanceof z.ZodNumber) return { type: "number" };
  if (schema instanceof z.ZodEnum) return { type: "string", enum: (schema as any)._def.values };
  if (schema instanceof z.ZodOptional) return zodToJsonSchema((schema as any)._def.innerType);
  if (schema instanceof z.ZodDefault) return zodToJsonSchema((schema as any)._def.innerType); // Defaults handled in logic
  
  return { type: "string" }; // Fallback
}

// Server implementation
const server = new Server(
  {
    name: "llm-json-formatter-mcp",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// Helper to run jf command
function runJf(args: string[], inputJson: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const process = spawn(JF_BINARY_PATH, args, {
        stdio: ['pipe', 'pipe', 'pipe']
    });

    let stdout = "";
    let stderr = "";

    process.stdout.on("data", (data) => {
      stdout += data.toString();
    });

    process.stderr.on("data", (data) => {
      stderr += data.toString();
    });

    process.on("close", (code) => {
      if (code === 0) {
        resolve(stdout.trim());
      } else {
        reject(new Error(`jf failed with code ${code}: ${stderr}`));
      }
    });

    process.on("error", (err) => {
        reject(err);
    });

    // Write input JSON to stdin
    process.stdin.write(inputJson);
    process.stdin.end();
  });
}

server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: TOOLS,
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "format_json": {
        const { json, mode, indent, sort } = args as any;
        const cmdArgs = ["format", "--mode", mode, "--indent", String(indent), "--sort", sort];
        const result = await runJf(cmdArgs, json);
        return {
          content: [{ type: "text", text: result }],
        };
      }

      case "analyze_json": {
        const { json } = args as any;
        const result = await runJf(["analyze"], json);
        return {
          content: [{ type: "text", text: result }],
        };
      }

      case "search_json": {
        const { json, path } = args as any;
        const result = await runJf(["search", "--path", path], json);
        return {
          content: [{ type: "text", text: result }],
        };
      }

      case "list_paths": {
        const { json } = args as any;
        const result = await runJf(["paths"], json);
        return {
          content: [{ type: "text", text: result }],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error: any) {
    return {
      content: [{ type: "text", text: `Error: ${error.message}` }],
      isError: true,
    };
  }
});

async function run() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("LLM JSON Formatter MCP Server running on stdio");
}

run().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
