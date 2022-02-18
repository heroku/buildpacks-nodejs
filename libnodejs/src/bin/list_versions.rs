use libnodejs::version::{Software, BUCKET, REGION};
use libnodejs::nodebin_s3;
use std::convert::TryFrom;

const FAILED_EXIT_CODE: i32 = 1;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("$ list <node|yarn>");
        std::process::exit(FAILED_EXIT_CODE);
    }

    let software_name = &args[1];
    let result = nodebin_s3::list_objects(BUCKET, REGION, software_name)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to fetch from S3: {}", e);
            std::process::exit(FAILED_EXIT_CODE);
        });
    let software = Software::try_from(result).unwrap_or_else(|e| {
        eprintln!("Failed to parse AWS S3 XML: {}", e);
        std::process::exit(FAILED_EXIT_CODE);
    });

    let toml_string = toml::to_string(&software).unwrap_or_else(|e| {
        eprintln!("Failed to convert to toml: {}", e);
        std::process::exit(FAILED_EXIT_CODE);
    });
    println!("{}", toml_string);
}
