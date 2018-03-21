extern crate ipfs_search;
extern crate tempdir;

use tempdir::TempDir;
use ipfs_search::run_indexer;

fn main() {
    if let Ok(dir) = TempDir::new("ipfs-search-index") {
        run_indexer(dir.path()).unwrap();
        dir.close().unwrap();
    }
}
