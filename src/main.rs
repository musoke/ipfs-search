extern crate failure;
extern crate ipfsapi;
extern crate mime;
extern crate serde_json;
extern crate tantivy;
extern crate tempdir;
extern crate tree_magic;

use ipfsapi::IpfsApi;
use failure::Error;
use std::path::Path;
use tempdir::TempDir;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};
use tantivy::collector::TopCollector;
use tantivy::query::QueryParser;

fn main() {
    if let Ok(dir) = TempDir::new("ipfs-search-index") {
        run_indexer(dir.path()).unwrap();
        check_index(dir.path()).unwrap();
        dir.close().unwrap();
    }
}

fn run_indexer(index_dir: &Path) -> Result<(), Error> {
    // Build the schema and index
    let mut schema_builder = SchemaBuilder::default();

    schema_builder.add_text_field("hash", TEXT | STORED);
    schema_builder.add_text_field("mime", TEXT | STORED);
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

    // Readme directory
    add_hash_to_index(
        "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
        &schema,
        &mut index_writer,
    )?;
    // IPFS About page
    add_hash_to_index(
        "QmZTR5bcpQD7cFgTorqxZDYaew1Wqgfbd2ud9QqGPAkK2V",
        &schema,
        &mut index_writer,
    )?;
    // Hello World
    add_hash_to_index(
        "QmfM2r8seH2GiRaC4esTjeraXEachRt8ZsSeGaWTPLyMoG",
        &schema,
        &mut index_writer,
    )?;

    for line in filtered_logs {
        let hash = line["key"].as_str().unwrap();
        println!("{}", hash);
        add_hash_to_index(hash, &schema, &mut index_writer)?;
    }

    index_writer.commit().unwrap();

    Ok(())
}

fn add_hash_to_index(
    hash: &str,
    schema: &Schema,
    index_writer: &mut IndexWriter,
) -> Result<(), Error> {
    use mime::*;

    // TODO: Check if hash already indexed
    // TODO: Check and limit size. `IpfsApi::object_stats` should be able to do this.

    // Fetch the hash and check mime-type
    let ipfs_api = IpfsApi::new("127.0.0.1", 5001);
    let file: Vec<u8> = ipfs_api.cat(hash)?.collect();
    let mime: mime::Mime = tree_magic::from_u8(&file).parse().expect("valid mime");

    let schema_hash = schema.get_field("hash").expect("field name set during dev");
    let schema_mime = schema.get_field("mime").expect("field name set during dev");
    let schema_body = schema.get_field("body").expect("field name set during dev");
    let mut doc = Document::default();

    doc.add_text(schema_hash, hash);
    doc.add_text(schema_mime, mime.as_ref());

    // Index actual content in some cases
    match (mime.type_(), mime.subtype()) {
        (TEXT, PLAIN) => {
            let body = String::from_utf8(file)?;
            doc.add_text(schema_body, &body);
        }
        (TEXT, _) => {
            // TODO: Preprocess somehow - don't want HTML tags, etc indexed
            let body = String::from_utf8(file)?;
            doc.add_text(schema_body, &body);
        }
        _ => {}
    }

    index_writer.add_document(doc);
    Ok(())
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
