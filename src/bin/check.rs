extern crate failure;
extern crate ipfs_search;
extern crate ipfsapi;
extern crate mime;
extern crate serde_json;
extern crate tantivy;
extern crate tempdir;
extern crate tree_magic;

use failure::Error;
use std::path::Path;
use tempdir::TempDir;
use tantivy::Index;
use tantivy::collector::TopCollector;
use tantivy::query::QueryParser;
use ipfs_search::run_indexer;

fn main() {
    if let Ok(dir) = TempDir::new("ipfs-search-index") {
        run_indexer(dir.path(), Some(10)).unwrap();
        check_index(dir.path()).unwrap();
        dir.close().unwrap();
    }
}

fn check_index(index_dir: &Path) -> Result<(), Error> {
    println!("Checking index");

    let index = Index::open(index_dir)?;
    index.load_searchers()?;
    let schema = index.schema();

    let schema_hash = schema.get_field("hash").expect("field name set during dev");
    let schema_mime = schema.get_field("mime").expect("field name set during dev");
    let schema_body = schema.get_field("body").expect("field name set during dev");

    let searcher = index.searcher();
    let query_parser = QueryParser::for_index(&index, vec![schema_hash, schema_mime, schema_body]);
    let mut top_collector = TopCollector::with_limit(10);

    println!("Search for \"text\":");
    let query = query_parser.parse_query("text").unwrap();
    searcher.search(&*query, &mut top_collector)?;
    let doc_addresses = top_collector.docs();
    for doc_address in doc_addresses {
        let retrieved_doc = searcher.doc(&doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    println!("Search for \"IPFS\":");
    let query = query_parser.parse_query("IPFS").unwrap();
    searcher.search(&*query, &mut top_collector)?;
    let doc_addresses = top_collector.docs();
    for doc_address in doc_addresses {
        let retrieved_doc = searcher.doc(&doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
