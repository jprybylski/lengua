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

    /// Path to the template library (a directory containing `.lengua/`).
    #[arg(long, global = true, default_value = ".")]
    pub store: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new template library (a `.lengua/` directory) with its first source,
    /// optionally adopting an existing store instead of starting empty.
    Init {
        /// Adopt an existing store from a local directory instead of
        /// starting empty. Mutually exclusive with `--from-repo`.
        #[arg(long, conflicts_with = "from_repo")]
        from_dir: Option<PathBuf>,

        /// Adopt an existing store by cloning it from a git URL (or
        /// `[host/]owner/repo[/subdir][@ref]` shorthand — host defaults to
        /// github.com, so an explicit host is how GitHub Enterprise is
        /// supported). Mutually exclusive with `--from-dir`.
        #[arg(long, conflicts_with = "from_dir")]
        from_repo: Option<String>,

        /// Branch or tag to check out (not a commit id). Only valid with
        /// `--from-repo`; overrides any `@ref` embedded in its shorthand.
        #[arg(long, requires = "from_repo")]
        r#ref: Option<String>,

        /// Import only this subdirectory of the resolved source as the new
        /// source, rather than the whole thing. Overrides any subdirectory
        /// embedded in `--from-repo`'s shorthand. Note: this imports the
        /// subdirectory's *current* content only — the source's history
        /// and any tags under it are not preserved, and it can't later be
        /// refreshed with `update`.
        #[arg(long)]
        subdir: Option<String>,

        /// Name for this library's first source. Defaults to `local` when
        /// starting empty, or derived from `--from-dir`/`--from-repo` when
        /// adopting one.
        #[arg(long)]
        name: Option<String>,

        /// With `--subdir`, allow re-importing into a store that already
        /// exists, overwriting templates with the same name.
        #[arg(long)]
        force: bool,
    },

    /// Add another source to an already-initialized library. Takes the same adoption flags
    /// as `init`, but requires `.lengua/` to already exist and appends rather than replacing
    /// — the new source becomes the highest-precedence one (see `--source` below).
    Fetch {
        /// Adopt the source from a local directory. Mutually exclusive with `--from-repo`.
        #[arg(long, conflicts_with = "from_repo")]
        from_dir: Option<PathBuf>,

        /// Adopt the source by cloning it from a git URL (or
        /// `[host/]owner/repo[/subdir][@ref]` shorthand). Mutually exclusive with
        /// `--from-dir`.
        #[arg(long, conflicts_with = "from_dir")]
        from_repo: Option<String>,

        /// Branch or tag to check out (not a commit id). Only valid with `--from-repo`.
        #[arg(long, requires = "from_repo")]
        r#ref: Option<String>,

        /// Import only this subdirectory of the resolved source. Note: this source can't
        /// later be refreshed with `update` (history isn't preserved for `--subdir`).
        #[arg(long)]
        subdir: Option<String>,

        /// Name for the new source. Auto-derived from `--from-dir`'s basename or
        /// `--from-repo`'s last path segment if omitted.
        #[arg(long)]
        name: Option<String>,

        /// With `--subdir`, allow re-importing into a source name that already exists.
        #[arg(long)]
        force: bool,
    },

    /// Refresh one source (`--source`) or every source from its origin, fast-forwarding
    /// only — fails loudly per-source if a fast-forward isn't possible rather than
    /// discarding anything. `local` and `--subdir`-imported sources have nothing to
    /// refresh from and are reported as such, not treated as a hard failure.
    Update {
        /// Refresh only this source. Defaults to every source in the library.
        #[arg(long)]
        source: Option<String>,
    },

    /// Add or update a template, committing the change.
    Add {
        /// Relative path/id under the source's `templates/` dir, e.g. `letters/thank-you.md`.
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
        /// Which source to write to. Required if the library has more than one source.
        #[arg(long)]
        source: Option<String>,
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
        /// Read the template as it existed at this revision (a lengua tag
        /// name, or any revspec gix understands, e.g. `HEAD~1`) instead of
        /// the working tree.
        #[arg(long)]
        rev: Option<String>,
        /// Read from this source specifically, bypassing merge precedence — the only way
        /// to reach a copy that's shadowed by another source.
        #[arg(long)]
        source: Option<String>,
    },

    /// List all templates in the library (merged across every source, unless `--source`).
    List {
        /// List only this source's templates.
        #[arg(long)]
        source: Option<String>,
    },

    /// Search templates by frontmatter field (AND of all `--field` filters).
    Search {
        #[arg(long = "field", value_name = "KEY=VALUE", required = true)]
        fields: Vec<String>,
        /// Search only this source's templates.
        #[arg(long)]
        source: Option<String>,
    },

    /// Show the commit history for a template.
    Log {
        name: String,
        /// Which source to read. Required if the library has more than one source.
        #[arg(long)]
        source: Option<String>,
    },

    /// Show a diff of a template's content between two revisions.
    Diff {
        name: String,
        #[arg(default_value = "HEAD~1")]
        from: String,
        #[arg(default_value = "HEAD")]
        to: String,
        /// Which source to read. Required if the library has more than one source.
        #[arg(long)]
        source: Option<String>,
    },

    /// Manage named pointers to a specific revision of a template. These
    /// are lengua's own tags (`refs/lengua/tags/<template>/<tag>`), not git
    /// tags, so the same tag name can exist independently on several
    /// templates.
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    /// Export lengua's bundled coding-agent skill files (`SKILL.md`) to a target directory.
    /// Doesn't touch a library, so `--store` is unused here.
    Skills {
        /// Target directory. Each skill lands under its own
        /// `<directory>/<skill-name>/SKILL.md` subdirectory — point this at
        /// `.claude/skills` for Claude Code to auto-discover them, or
        /// anywhere else for a different tool, or just to inspect the
        /// content.
        #[arg(default_value = ".")]
        directory: PathBuf,
        /// Overwrite an existing `SKILL.md` at the destination.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum TagAction {
    /// Point a tag at a template's current revision (or `--rev`).
    Add {
        template: String,
        tag: String,
        /// Revision to tag instead of the current `HEAD`, e.g. `HEAD~1` —
        /// this is how a *prior* revision gets retroactively tagged.
        #[arg(long)]
        rev: Option<String>,
        /// Overwrite the tag if it already exists.
        #[arg(long)]
        force: bool,
        /// Which source to target. Required if the library has more than one source.
        #[arg(long)]
        source: Option<String>,
    },
    /// List every tag on a template.
    List {
        template: String,
        /// Which source to read. Required if the library has more than one source.
        #[arg(long)]
        source: Option<String>,
    },
    /// Remove a tag from a template.
    Rm {
        template: String,
        tag: String,
        /// Which source to target. Required if the library has more than one source.
        #[arg(long)]
        source: Option<String>,
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
