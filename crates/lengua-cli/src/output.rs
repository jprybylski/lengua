use lengua_core::ShadowWarning;
use owo_colors::OwoColorize;
use serde::Serialize;

pub fn print_json(value: &impl Serialize) {
    anstream::println!(
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
        anstream::eprintln!("{} {message}", "error:".red().bold());
    }
}

/// lengua's terminal color palette (`anstream` auto-respects `NO_COLOR` /
/// `CLICOLOR` / `CLICOLOR_FORCE`, so these are safe to use unconditionally).
/// lenguar's `R/result.R` documents the same mapping so both CLIs read as
/// one brand:
///   green = inserted/added diff lines
///   red   = deleted/removed diff lines, and the "error:" prefix
///   cyan  = tag names
///   bold  = headings/confirmations
///   dim   = de-emphasized metadata (commit hashes)
pub fn inserted(s: &str) -> String {
    s.green().to_string()
}

pub fn deleted(s: &str) -> String {
    s.red().to_string()
}

pub fn tag_name(s: &str) -> String {
    s.cyan().to_string()
}

pub fn heading(s: &str) -> String {
    s.bold().to_string()
}

pub fn dim(s: &str) -> String {
    s.dimmed().to_string()
}

pub fn warning(s: &str) -> String {
    s.yellow().bold().to_string()
}

/// Prints every current name collision across a library's sources, once, to stderr — a
/// shadowed name is never silently resolved. No-op when `warnings` is empty.
pub fn print_shadow_warnings(warnings: &[ShadowWarning]) {
    for w in warnings {
        anstream::eprintln!(
            "{} '{}' is shadowed by '{}' (also defined in '{}')",
            warning("warning:"),
            w.name,
            w.winner,
            w.loser
        );
    }
}
