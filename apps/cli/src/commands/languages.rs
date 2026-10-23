use dubbit_core::languages::LANGUAGES;

pub fn run(json: bool) {
    if json {
        let rows = LANGUAGES
            .iter()
            .map(|(code, name)| serde_json::json!({ "code": code, "name": name }))
            .collect::<Vec<serde_json::Value>>();
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_default()
        );
        return;
    }
    for (code, name) in LANGUAGES {
        println!("{code:<6} {name}");
    }
}
