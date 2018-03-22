extern crate clap;
extern crate ipfs_search;
extern crate tempdir;

use tempdir::TempDir;
use ipfs_search::run_indexer;
use clap::{App, Arg, ArgMatches};
use std::path::Path;

fn main() {
    let matches = clap_app();

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
    let app = App::new("ipfs-search").arg(
        Arg::with_name("dir")
            .long("dir")
            .short("d")
            .takes_value(true)
            .value_name("DIR")
            .help("directory containing index"),
    );

    app.get_matches()
}
