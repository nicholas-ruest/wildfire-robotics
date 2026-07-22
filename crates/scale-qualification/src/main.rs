#![allow(missing_docs)]

use scale_qualification::{Campaign, Objectives, Workload};

fn main() {
    let result = match Campaign::new(Workload::production()).run() {
        Ok(result) => result,
        Err(error) => {
            eprintln!("qualification failed: {error}");
            std::process::exit(2);
        }
    };
    if !result.objectives_pass(&Objectives::approved()) {
        eprintln!("approved scale objectives failed");
        std::process::exit(1);
    }
    match serde_json::to_string_pretty(&result) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("could not serialize evidence: {error}");
            std::process::exit(2);
        }
    }
}
