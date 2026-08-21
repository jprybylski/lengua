use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lengua",
    version,
    about = "A git-backed library of templated text and snippets"
)]
pub struct Cli {
    /// Emit structured JSON instead of human-readable output.
    #[arg(long, global = true)]
    pub json: bool,

    /// Path to the template store (a git repo with a `templates/` dir).
    #[arg(long, global = true, default_value = ".")]
    pub store: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new template-library git repo.
    Init,

    /// Add or update a template, committing the change.
    Add {
        /// Relative path/id under the store's `templates/` dir, e.g. `letters/thank-you.md`.
        name: String,
        /// Read the template body from this file instead of stdin.
        #[arg(long)]
        file: Option<PathBuf>,
        /// Frontmatter title.
        #[arg(long)]
        title: Option<String>,
        /// Frontmatter field as `key=value`; repeatable. Values are stored as strings.
        #[arg(long = "field", value_name = "KEY=VALUE")]
        fields: Vec<String>,
        /// Commit message.
        #[arg(long, default_value = "add/update template")]
        message: String,
    },

    /// Render a template with variables substituted.
    #[command(alias = "render")]
    Get {
        name: String,
        /// Template variable as `key=value`; repeatable.
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,
        /// Print the raw (unrendered) body instead of rendering it.
        #[arg(long)]
        raw: bool,
    },

    /// List all templates in the store.
    List,

    /// Search templates by frontmatter field (AND of all `--field` filters).
    Search {
        #[arg(long = "field", value_name = "KEY=VALUE", required = true)]
        fields: Vec<String>,
    },

    /// Show the commit history for a template.
    Log { name: String },

    /// Show a diff of a template's content between two revisions.
    Diff {
        name: String,
        #[arg(default_value = "HEAD~1")]
        from: String,
        #[arg(default_value = "HEAD")]
        to: String,
    },
}

/// Parses a repeated `key=value` CLI argument.
pub fn parse_kv_pairs(pairs: &[String]) -> Result<Vec<(String, String)>, String> {
    pairs
        .iter()
        .map(|pair| {
            pair.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .ok_or_else(|| format!("expected `key=value`, got `{pair}`"))
        })
        .collect()
}
