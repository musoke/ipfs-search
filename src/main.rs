extern crate failure;
extern crate ipfsapi;
extern crate serde_json;

use ipfsapi::IpfsApi;
use failure::Error;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Error> {
    let api = IpfsApi::new("127.0.0.1", 5001);

    let filtered_logs = api.log_tail()?
        .filter(|l| l["event"].as_str() == Some("handleAddProvider"))
        .filter(|x| x["key"].is_string());

    for line in filtered_logs {
        let hash = line["key"].as_str().unwrap();
        println!("{}", hash);
    }

    Ok(())
}
