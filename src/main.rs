use clap::{Parser, Subcommand, ValueEnum};
use llm_json_formatter::{
    generate_schema, Config, FormatMode, JsonIndex, LlmJsonFormatter, SortStrategy,
};
use std::collections::HashSet;
use std::io::{self, Read};

const EXAMPLES: &str = r#"
Examples:
  # Quick format (shortcut, defaults to smart mode)
  jf data.json
  jf *.json

  # Pipe input shortcut
  echo '{"users":[{"id":1,"name":"Alice"}]}' | jf

  # Default format (Auto-detect entities)
  echo '{"users":[{"id":1,"name":"Alice"}]}' | jf format

  # Generate prompt for LLM to identify entities
  jf prompt -i data.json

  # Format with specific entities (force single line)
  jf format -i data.json --entities "users[*],orders[*]"

  # Compact format (minimized)
  jf format -i input.json --mode compact

  # Pretty format (standard indentation)
  jf format -i input.json --mode pretty
"#;

#[derive(Parser)]
#[command(name = "jf")]
#[command(version)]
#[command(about = "LLM-optimized JSON formatter - balance readability and token efficiency")]
#[command(long_about = None)]
#[command(after_help = EXAMPLES)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Format JSON with smart/compact/pretty modes")]
    Format {
        #[arg(
            short,
            long,
            help = "Input JSON file (reads from stdin if not provided)"
        )]
        input: Option<String>,

        #[arg(short, long, help = "Output file (prints to stdout if not provided)")]
        output: Option<String>,

        #[arg(
            short,
            long,
            value_enum,
            default_value = "smart",
            help = "Formatting mode"
        )]
        mode: FormatModeArg,

        #[arg(
            long,
            value_enum,
            default_value = "smart",
            help = "Key sorting strategy"
        )]
        sort: SortArg,

        #[arg(long, default_value = "2", help = "Indentation spaces")]
        indent: usize,

        #[arg(
            long,
            default_value = "80",
            help = "Max line length for inline objects in Smart mode"
        )]
        inline_limit: usize,

        #[arg(
            long,
            default_value = "2048",
            help = "Max line length for array items (entities) in Smart mode"
        )]
        array_item_inline_limit: usize,

        #[arg(
            long,
            default_value = "2000",
            help = "Length threshold for auto-detected entities"
        )]
        entity_threshold: usize,

        #[arg(
            long,
            help = "Comma-separated list of entity paths to force single-line (e.g. 'users[*],items[*]')"
        )]
        entities: Option<String>,
    },

    #[command(about = "Generate LLM prompt to identify entities")]
    Prompt {
        #[arg(
            short,
            long,
            help = "Input JSON file (reads from stdin if not provided)"
        )]
        input: Option<String>,
    },

    #[command(about = "Analyze JSON structure")]
    Analyze {
        #[arg(
            short,
            long,
            help = "Input JSON file (reads from stdin if not provided)"
        )]
        input: Option<String>,
    },

    #[command(about = "Search value by JSON path (e.g., users[0].name)")]
    Search {
        #[arg(
            short,
            long,
            help = "Input JSON file (reads from stdin if not provided)"
        )]
        input: Option<String>,

        #[arg(short, long, help = "JSON path to search")]
        path: String,
    },

    #[command(about = "List all available paths in JSON")]
    Paths {
        #[arg(
            short,
            long,
            help = "Input JSON file (reads from stdin if not provided)"
        )]
        input: Option<String>,
    },

    #[command(about = "Extract compact schema from JSON")]
    Schema {
        #[arg(
            short,
            long,
            help = "Input JSON file (reads from stdin if not provided)"
        )]
        input: Option<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum FormatModeArg {
    #[value(help = "Smart inline/multiline (balance readability and tokens)")]
    Smart,
    #[value(help = "Minimized single line (lowest tokens)")]
    Compact,
    #[value(help = "Standard multiline indentation (best readability)")]
    Pretty,
}

#[derive(Clone, ValueEnum)]
enum SortArg {
    #[value(help = "Sort keys alphabetically")]
    Alphabetic,
    #[value(help = "Sort by importance (id/name first, _internal last)")]
    Smart,
    #[value(help = "Do not sort keys (preserve original order)")]
    None,
}

fn read_input(input: Option<String>) -> io::Result<String> {
    match input {
        Some(path) => std::fs::read_to_string(path),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn write_output(output: Option<String>, content: &str) -> io::Result<()> {
    match output {
        Some(path) => std::fs::write(path, content),
        None => {
            println!("{}", content);
            Ok(())
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        use std::io::IsTerminal;
        if !io::stdin().is_terminal() {
            let format_args = vec![args[0].clone(), "format".to_string()];

            let cli = match Cli::try_parse_from(format_args) {
                Ok(cli) => cli,
                Err(e) => {
                    eprintln!("Error parsing arguments: {}", e);
                    std::process::exit(1);
                }
            };

            let result = execute_command(cli.command);
            if let Err(e) = result {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            return;
        }
    }

    if args.len() >= 2
        && !args[1].starts_with('-')
        && !matches!(
            args[1].as_str(),
            "format" | "prompt" | "analyze" | "search" | "paths" | "schema" | "help"
        )
    {
        let file_path = &args[1];
        if file_path.ends_with(".json") {
            for arg in args.iter().skip(1) {
                if !arg.ends_with(".json") {
                    eprintln!("Error: All arguments must be JSON files when using shortcut mode");
                    std::process::exit(1);
                }
            }

            for file_arg in args.iter().skip(1) {
                let format_args = vec![
                    args[0].clone(),
                    "format".to_string(),
                    "-i".to_string(),
                    file_arg.clone(),
                ];

                let cli = match Cli::try_parse_from(format_args) {
                    Ok(cli) => cli,
                    Err(e) => {
                        eprintln!("Error parsing arguments: {}", e);
                        std::process::exit(1);
                    }
                };

                let result = execute_command(cli.command);
                if let Err(e) = result {
                    eprintln!("Error processing {}: {}", file_arg, e);
                    std::process::exit(1);
                }
            }
            return;
        }
    }

    let cli = Cli::parse();
    let result = execute_command(cli.command);
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn execute_command(command: Commands) -> io::Result<()> {
    let result = match command {
        Commands::Format {
            input,
            output,
            mode,
            sort,
            indent,
            inline_limit,
            array_item_inline_limit,
            entity_threshold,
            entities,
        } => {
            let json = match read_input(input) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    std::process::exit(1);
                }
            };

            let sort_strategy = match sort {
                SortArg::Alphabetic => SortStrategy::Alphabetic,
                SortArg::Smart => SortStrategy::Smart,
                SortArg::None => SortStrategy::None,
            };

            let format_mode = match mode {
                FormatModeArg::Smart => FormatMode::Smart,
                FormatModeArg::Compact => FormatMode::Compact,
                FormatModeArg::Pretty => FormatMode::Pretty,
            };

            let mut entity_set = HashSet::new();
            if let Some(e_str) = entities {
                let trimmed = e_str.trim();
                // Support both JSON array format and comma-separated format
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    // JSON array format: ["a", "b", "c"]
                    if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
                        for item in arr {
                            entity_set.insert(item);
                        }
                    } else {
                        eprintln!("Warning: Failed to parse entities as JSON array, trying comma-separated");
                        for part in e_str.split(',') {
                            entity_set.insert(part.trim().to_string());
                        }
                    }
                } else {
                    // Comma-separated format: a,b,c
                    for part in e_str.split(',') {
                        entity_set.insert(part.trim().to_string());
                    }
                }
            }

            let config = Config {
                mode: format_mode,
                sort_strategy,
                indent,
                inline_limit,
                array_item_inline_limit,
                entity_threshold,
                entities: entity_set,
            };

            let mut formatter = LlmJsonFormatter::new(config);
            match formatter.format(&json) {
                Ok(result) => write_output(output, &result),
                Err(e) => {
                    eprintln!("Error formatting JSON: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Prompt { input } => {
            let json = match read_input(input) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    std::process::exit(1);
                }
            };

            let mut formatter = LlmJsonFormatter::new(Config::default());
            match formatter.generate_prompt(&json) {
                Ok(prompt) => {
                    println!("{}", prompt);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error generating prompt: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Analyze { input } => {
            let json = match read_input(input) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    std::process::exit(1);
                }
            };

            let formatter = LlmJsonFormatter::new(Config::default());
            match formatter.get_metadata(&json) {
                Ok(meta) => {
                    println!("JSON Analysis:");
                    println!("  Byte Size: {} bytes", meta.byte_size);
                    println!("  Max Depth: {}", meta.depth);
                    println!("  Object Count: {}", meta.object_count);
                    println!("  Total Keys: {}", meta.total_keys);
                    println!("  Array Count: {}", meta.array_count);
                    println!("  Max Array Length: {}", meta.max_array_len);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error analyzing JSON: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Search { input, path } => {
            let json = match read_input(input) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    std::process::exit(1);
                }
            };

            match JsonIndex::build(&json) {
                Ok(index) => {
                    if let Some(info) = index.search(&path) {
                        println!("Path: {}", path);
                        println!("Type: {:?}", info.value_type);
                        println!("Preview: {}", info.preview);
                        Ok(())
                    } else {
                        eprintln!("Path not found: {}", path);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error building index: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Paths { input } => {
            let json = match read_input(input) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    std::process::exit(1);
                }
            };

            match JsonIndex::build(&json) {
                Ok(index) => {
                    for path in index.list_paths() {
                        println!("{}", path);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error building index: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Schema { input } => {
            let json = match read_input(input) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    std::process::exit(1);
                }
            };

            match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(value) => {
                    let schema = generate_schema(&value, 0);
                    println!("{}", schema);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error parsing JSON: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    result
}
