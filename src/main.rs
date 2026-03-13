use clap::{Parser, Subcommand, ValueEnum};
use llm_json_formatter::{
    generate_schema, parse_json_or_json5, Config, FormatMode, JsonIndex, LlmJsonFormatter,
    OutputSyntax, SortStrategy,
};
use std::collections::HashSet;
use std::io;
use std::path::Path;

const EXAMPLES: &str = r#"
Examples:
  # Quick format (shortcut, defaults to smart mode)
  jf data.json
  jf *.json

  # Default format (Auto-detect entities)
  jf format data.json

  # Generate prompt for LLM to identify entities
  jf prompt data.json

  # Fuzzy search paths by key
  jf search data.json --key name

  # Format with specific entities (force single line)
  jf format data.json --entities "users[*],orders[*]"

  # Compact format (minimized)
  jf format input.json --mode compact

  # Pretty format (standard indentation)
  jf format input.json --mode pretty

  # Keep input syntax automatically (JSON in -> JSON out, JSON5 in -> JSON5 out)
  jf format input.json5 --output-syntax auto

  # Force JSON5 output
  jf format input.json --output-syntax json5
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
        #[arg(value_name = "INPUT", help = "Input JSON file path")]
        input: String,

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

        #[arg(
            long,
            value_enum,
            default_value = "auto",
            help = "Output syntax: auto/json/json5"
        )]
        output_syntax: OutputSyntaxArg,

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
        #[arg(value_name = "INPUT", help = "Input JSON file path")]
        input: String,
    },

    #[command(about = "Analyze JSON structure")]
    Analyze {
        #[arg(value_name = "INPUT", help = "Input JSON file path")]
        input: String,
    },

    #[command(about = "Search by JSON path or fuzzy key")]
    Search {
        #[arg(value_name = "INPUT", help = "Input JSON file path")]
        input: String,

        #[arg(
            short,
            long,
            help = "JSON path to search",
            required_unless_present = "key",
            conflicts_with = "key"
        )]
        path: Option<String>,

        #[arg(
            long,
            help = "Fuzzy key query to search matched paths (case-insensitive)",
            required_unless_present = "path",
            conflicts_with = "path"
        )]
        key: Option<String>,

        #[arg(short, long, help = "Output full JSON value instead of preview", conflicts_with = "key")]
        full: bool,
    },

    #[command(about = "List all available paths in JSON")]
    Paths {
        #[arg(value_name = "INPUT", help = "Input JSON file path")]
        input: String,
    },

    #[command(about = "Extract compact schema from JSON")]
    Schema {
        #[arg(value_name = "INPUT", help = "Input JSON file path")]
        input: String,
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

#[derive(Clone, ValueEnum)]
enum OutputSyntaxArg {
    #[value(help = "Auto-detect from input syntax")]
    Auto,
    #[value(help = "Force JSON output")]
    Json,
    #[value(help = "Force JSON5 output")]
    Json5,
}

fn read_input(path: &str) -> io::Result<String> {
    std::fs::read_to_string(path)
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

    if args.len() >= 2
        && !args[1].starts_with('-')
        && !matches!(
            args[1].as_str(),
            "format" | "prompt" | "analyze" | "search" | "paths" | "schema" | "help"
        )
    {
        let all_are_existing_files = args.iter().skip(1).all(|arg| Path::new(arg).is_file());
        let all_are_json_or_json5_names = args
            .iter()
            .skip(1)
            .all(|arg| arg.ends_with(".json") || arg.ends_with(".json5"));

        if all_are_existing_files || all_are_json_or_json5_names {
            for arg in args.iter().skip(1) {
                if !Path::new(arg).is_file() && !arg.ends_with(".json") && !arg.ends_with(".json5")
                {
                    eprintln!(
                        "Error: All arguments must be existing files or JSON/JSON5 file names when using shortcut mode"
                    );
                    std::process::exit(1);
                }
            }

            for file_arg in args.iter().skip(1) {
                let format_args = vec![
                    args[0].clone(),
                    "format".to_string(),
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
            output_syntax,
            indent,
            inline_limit,
            array_item_inline_limit,
            entity_threshold,
            entities,
        } => {
            let json = match read_input(&input) {
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
                output_syntax: match output_syntax {
                    OutputSyntaxArg::Auto => OutputSyntax::Auto,
                    OutputSyntaxArg::Json => OutputSyntax::Json,
                    OutputSyntaxArg::Json5 => OutputSyntax::Json5,
                },
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
            let json = match read_input(&input) {
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
            let json = match read_input(&input) {
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

        Commands::Search {
            input,
            path,
            key,
            full,
        } => {
            let json = match read_input(&input) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    std::process::exit(1);
                }
            };

            match JsonIndex::build(&json) {
                Ok(index) => {
                    if let Some(path) = path {
                        if let Some(info) = index.search(&path) {
                            if full {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&info.full_value).unwrap_or_default()
                                );
                            } else {
                                println!("Path: {}", path);
                                println!("Type: {:?}", info.value_type);
                                println!("Preview: {}", info.preview);
                            }
                            Ok(())
                        } else {
                            eprintln!("Path not found: {}", path);
                            std::process::exit(1);
                        }
                    } else if let Some(key) = key {
                        let paths = index.search_paths_by_key(&key);
                        if paths.is_empty() {
                            eprintln!("No matched paths for key query: {}", key);
                            std::process::exit(1);
                        } else {
                            for path in paths {
                                println!("{}", path);
                            }
                            Ok(())
                        }
                    } else {
                        eprintln!("Either --path or --key must be provided");
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
            let json = match read_input(&input) {
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
            let json = match read_input(&input) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    std::process::exit(1);
                }
            };

            match parse_json_or_json5(&json).map(|(value, _)| value) {
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
