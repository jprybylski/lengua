mod cli;
mod commands;
mod from_repo;
mod output;

use clap::Parser;
use cli::{Cli, Command, TagAction};

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    let result = match cli.command {
        Command::Init {
            from_dir,
            from_repo,
            r#ref,
            subdir,
            name,
            force,
        } => commands::init(
            &cli.store, from_dir, from_repo, r#ref, subdir, name, force, json,
        ),
        Command::Fetch {
            from_dir,
            from_repo,
            r#ref,
            subdir,
            name,
            force,
        } => commands::fetch(
            &cli.store, from_dir, from_repo, r#ref, subdir, name, force, json,
        ),
        Command::Update { source } => commands::update(&cli.store, source, json),
        Command::Add {
            name,
            file,
            title,
            fields,
            message,
            source,
        } => commands::add(
            &cli.store, &name, file, title, &fields, &message, source, json,
        ),
        Command::Get {
            name,
            vars,
            raw,
            rev,
            source,
        } => commands::get(&cli.store, &name, &vars, raw, rev, source, json),
        Command::List { source } => commands::list(&cli.store, source, json),
        Command::Search { fields, source } => commands::search(&cli.store, &fields, source, json),
        Command::Log { name, source } => commands::log(&cli.store, &name, source, json),
        Command::Diff {
            name,
            from,
            to,
            source,
        } => commands::diff(&cli.store, &name, &from, &to, source, json),
        Command::Tag { action } => match action {
            TagAction::Add {
                template,
                tag,
                rev,
                force,
                source,
            } => commands::tag_add(&cli.store, &template, &tag, rev, force, source, json),
            TagAction::List { template, source } => {
                commands::tag_list(&cli.store, &template, source, json)
            }
            TagAction::Rm {
                template,
                tag,
                source,
            } => commands::tag_rm(&cli.store, &template, &tag, source, json),
        },
    };

    if let Err(err) = result {
        output::print_error(json, &format!("{err:#}"));
        std::process::exit(1);
    }
}
