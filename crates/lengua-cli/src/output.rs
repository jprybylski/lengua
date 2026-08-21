use serde::Serialize;

pub fn print_json(value: &impl Serialize) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("value is always serializable")
    );
}

pub fn print_error(json: bool, message: &str) {
    if json {
        eprintln!(
            r#"{{"error": {}}}"#,
            serde_json::to_string(message).unwrap()
        );
    } else {
        eprintln!("error: {message}");
    }
}
