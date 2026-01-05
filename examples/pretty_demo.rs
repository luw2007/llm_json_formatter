use serde_json::json;

fn main() {
    let data = json!({
        "players": [
            {"id": 1, "name": "Alice", "score": 100},
            {"id": 2, "name": "Bob", "score": 85}
        ],
        "game": "chess",
        "round": 3
    });

    println!("=== to_string (紧凑格式) ===");
    let compact = serde_json::to_string(&data).unwrap();
    println!("{}", compact);
    println!("字符数: {}\n", compact.len());
    
    println!("=== to_string_pretty (美化格式) ===");
    let pretty = serde_json::to_string_pretty(&data).unwrap();
    println!("{}", pretty);
    println!("字符数: {}", pretty.len());
}
