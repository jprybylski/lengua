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
            force,
        } => commands::init(&cli.store, from_dir, from_repo, r#ref, subdir, force, json),
        Command::Add {
            name,
            file,
            title,
            fields,
            message,
        } => commands::add(&cli.store, &name, file, title, &fields, &message, json),
        Command::Get {
            name,
            vars,
            raw,
            rev,
        } => commands::get(&cli.store, &name, &vars, raw, rev, json),
        Command::List => commands::list(&cli.store, json),
        Command::Search { fields } => commands::search(&cli.store, &fields, json),
        Command::Log { name } => commands::log(&cli.store, &name, json),
        Command::Diff { name, from, to } => commands::diff(&cli.store, &name, &from, &to, json),
        Command::Tag { action } => match action {
            TagAction::Add {
                template,
                tag,
                rev,
                force,
            } => commands::tag_add(&cli.store, &template, &tag, rev, force, json),
            TagAction::List { template } => commands::tag_list(&cli.store, &template, json),
            TagAction::Rm { template, tag } => commands::tag_rm(&cli.store, &template, &tag, json),
        },
    };

    if let Err(err) = result {
        output::print_error(json, &format!("{err:#}"));
        std::process::exit(1);
    }
}
