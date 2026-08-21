mod cli;
mod commands;
mod output;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    let result = match cli.command {
        Command::Init => commands::init(&cli.store, json),
        Command::Add {
            name,
            file,
            title,
            fields,
            message,
        } => commands::add(&cli.store, &name, file, title, &fields, &message, json),
        Command::Get { name, vars, raw } => commands::get(&cli.store, &name, &vars, raw, json),
        Command::List => commands::list(&cli.store, json),
        Command::Search { fields } => commands::search(&cli.store, &fields, json),
        Command::Log { name } => commands::log(&cli.store, &name, json),
        Command::Diff { name, from, to } => commands::diff(&cli.store, &name, &from, &to, json),
    };

    if let Err(err) = result {
        output::print_error(json, &format!("{err:#}"));
        std::process::exit(1);
    }
}
