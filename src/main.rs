extern crate failure;
extern crate ipfsapi;
extern crate serde_json;
extern crate tantivy;
extern crate tempdir;

use ipfsapi::IpfsApi;
use failure::Error;
use std::path::Path;
use tempdir::TempDir;
use tantivy::schema::*;
use tantivy::Index;

fn main() {
    if let Ok(dir) = TempDir::new("ipfs-search-index") {
        run_indexer(dir.path()).unwrap();
        dir.close().unwrap();
    }
}

fn run_indexer(index_dir: &Path) -> Result<(), Error> {
    // Build the schema and index
    let mut schema_builder = SchemaBuilder::default();

    schema_builder.add_text_field("hash", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    let schema = schema_builder.build();
    let index = Index::create(index_dir, schema.clone())?;

    let mut index_writer = index.writer(50_000_000)?;

    // Get events from IPFS
    let ipfs_api = IpfsApi::new("127.0.0.1", 5001);
    let filtered_logs = ipfs_api
        .log_tail()?
        .filter(|l| l["event"].as_str() == Some("handleAddProvider"))
        .filter(|x| x["key"].is_string())
        .take(10);

    for line in filtered_logs {
        let hash = line["key"].as_str().unwrap();
        println!("{}", hash);

        // TODO: fetch the hash and parse its content

        let schema_hash = schema.get_field("hash").expect("field name set during dev");
        // let schema_body = schema.get_field("body").expect("field name set during dev");
        let mut doc = Document::default();

        doc.add_text(schema_hash, hash);

        index_writer.add_document(doc);
    }

    index_writer.commit().unwrap();

    Ok(())
}
