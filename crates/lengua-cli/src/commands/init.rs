use std::path::Path;

use anyhow::Result;
use lengua_core::Store;
use serde::Serialize;

use crate::output::print_json;

#[derive(Serialize)]
struct InitOutput<'a> {
    status: &'static str,
    path: &'a str,
}

pub fn run(store_path: &Path, json: bool) -> Result<()> {
    Store::init(store_path)?;
    let path = store_path.to_string_lossy();
    if json {
        print_json(&InitOutput {
            status: "initialized",
            path: &path,
        });
    } else {
        println!("Initialized empty lengua store at {path}");
    }
    Ok(())
}
