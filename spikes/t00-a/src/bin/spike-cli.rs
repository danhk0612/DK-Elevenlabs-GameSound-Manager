fn main() {
    let result = std::env::args()
        .nth(1)
        .and_then(|s| serde_json::from_str(&s).ok())
        .map(t00_spike::response)
        .unwrap_or_else(|| serde_json::json!({"ok":false,"error":"JSON command required"}));
    println!("{result}");
    if result["ok"] != true {
        std::process::exit(1);
    }
}
