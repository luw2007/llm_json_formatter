
use llm_json_formatter::{Config, LlmJsonFormatter, SortStrategy};

fn main() {
    let json = r#"{"z": 1, "a": 2, "c": 3}"#;
    
    // Default (Alphabetic)
    let mut formatter = LlmJsonFormatter::new(Config::default());
    let formatted_default = formatter.format(json).unwrap();
    println!("Original: {}", json);
    println!("Formatted (Default/Alphabetic): {}", formatted_default);

    // Smart
    let mut config_smart = Config::default();
    config_smart.sort_strategy = SortStrategy::Smart;
    let mut formatter_smart = LlmJsonFormatter::new(config_smart);
    let formatted_smart = formatter_smart.format(json).unwrap();
    println!("Formatted (Smart): {}", formatted_smart);

    // None
    let mut config_none = Config::default();
    config_none.sort_strategy = SortStrategy::None;
    let mut formatter_none = LlmJsonFormatter::new(config_none);
    let formatted_none = formatter_none.format(json).unwrap();
    println!("Formatted (None): {}", formatted_none);
}
