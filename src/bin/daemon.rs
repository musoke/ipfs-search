extern crate env_logger;
extern crate log;

#[macro_use]
extern crate clap;
extern crate ipfs_search;
extern crate tempdir;

use tempdir::TempDir;
use ipfs_search::run_indexer;
use clap::{App, Arg, ArgMatches};
use std::path::Path;
use log::LevelFilter;
use env_logger::Builder;

fn main() {
    let matches = clap_app();

    // Initialize logging
    {
        let mut log_builder = Builder::new();
        let log_module = Some("ipfs_search");
        match matches.occurrences_of("v") {
            0 => log_builder.filter(log_module, LevelFilter::Error),
            1 => log_builder.filter(log_module, LevelFilter::Warn),
            2 => log_builder.filter(log_module, LevelFilter::Info),
            3 => log_builder.filter(log_module, LevelFilter::Debug),
            _ => log_builder.filter(None, LevelFilter::Trace),
        };
        log_builder.init();
    }

    if let Some(dir) = matches.value_of("dir") {
        let path = Path::new(dir);
        run_indexer(&path, None).unwrap();
    } else {
        if let Ok(dir) = TempDir::new("ipfs-search-index") {
            run_indexer(dir.path(), None).unwrap();
            dir.close().unwrap();
        }
    }
}

fn clap_app() -> ArgMatches<'static> {
    let app = App::new(format!("{} {}", crate_name!(), "daemon"))
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("dir")
                .long("dir")
                .short("d")
                .takes_value(true)
                .value_name("DIR")
                .help("Directory containing index"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        );

    app.get_matches()
}
