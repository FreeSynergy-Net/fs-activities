//! `fs-activities`: one binary for the program (ADR 0063). Pattern: Facade over the crates that stay libraries.
//!
//! ```text
//! fs-activities                          make sure the server runs, open the frontend `simple`
//! fs-activities [--frontend ID] [--socket PATH]
//! fs-activities serve [options]          the server (Cap'n Proto, history.capnp)
//! fs-activities --version                the version, without opening anything
//! fs-activities --help                   this help
//! ```

mod starter;

use std::process::ExitCode;

use fs_activities_server::ServeError;

const HELP: &str = "fs-activities [--frontend ID] [--socket PATH]\n\
fs-activities serve [--socket PATH] [--data DIR] [--actor IRI] [--instance DOMAIN] [--facet NAME]\n\
fs-activities --version | --help\n\
Without words: makes sure the server runs and opens the frontend (default: simple).\n\
  serve            run the server (see `fs-activities serve --help`)\n\
  --frontend ID    the frontend to open; if its contract does not fit the server, the standard frontend opens\n\
  --socket PATH    the server's socket\n\
Exit codes: 0 done, 1 failed, 2 wrong words";

fn main() -> ExitCode {
    let raw: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    match raw.first().and_then(|w| w.to_str()) {
        Some("--version" | "-V") if raw.len() == 1 => {
            println!("fs-activities {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") if raw.len() == 1 => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("serve") => serve(&raw[1..]),
        _ => starter::run(&raw),
    }
}

fn serve(words: &[std::ffi::OsString]) -> ExitCode {
    match fs_activities_server::serve(words.iter().cloned()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ServeError::Usage(m)) => {
            eprintln!("fs-activities: {m}");
            ExitCode::from(2)
        }
        Err(ServeError::Failed(m)) => {
            eprintln!("fs-activities: {m}");
            ExitCode::FAILURE
        }
    }
}
