//! Fetches the Node.js release schedule and writes it to a TOML file.

use clap::{arg, command, value_parser};
use nodejs_data::{NodejsRelease, NodejsReleaseMetadata, NodejsReleaseSchedule, Range};
use serde::Deserialize;
use std::path::PathBuf;
use time::{Date, macros::format_description};

fn main() {
    let matches = command!()
        .arg(
            arg!(<schedule_path>)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .get_matches();

    let schedule_path = matches
        .get_one::<PathBuf>("schedule_path")
        .expect("should be a required argument");

    eprintln!("Fetching Node.js release schedule...");
    let releases = list_upstream_releases();

    eprintln!("Writing release schedule...");
    std::fs::write(
        schedule_path,
        NodejsReleaseSchedule { releases }.to_string(),
    )
    .expect("Failed to write release schedule");

    eprintln!("Done.");
}

#[derive(Deserialize, Debug)]
struct UpstreamScheduleEntry {
    start: String,
    end: String,
    lts: Option<String>,
    maintenance: Option<String>,
}

fn parse_date(input: &str) -> Date {
    Date::parse(input, format_description!("[year]-[month]-[day]")).expect("Date should be valid")
}

static RELEASE_SCHEDULE_URL: &str =
    "https://raw.githubusercontent.com/nodejs/Release/main/schedule.json";

fn list_upstream_releases() -> Vec<NodejsRelease> {
    let body: serde_json::Map<String, serde_json::Value> = ureq::get(RELEASE_SCHEDULE_URL)
        .call()
        .expect("Node.js release schedule should be available")
        .body_mut()
        .read_json()
        .expect("Node.js release schedule should be parsable from JSON");

    body.into_iter()
        .map(|(key, value)| {
            let entry: UpstreamScheduleEntry = serde_json::from_value(value)
                .unwrap_or_else(|e| panic!("Invalid schedule entry for {key}: {e}"));
            let range = Range::parse(&key)
                .unwrap_or_else(|_| panic!("Invalid range for schedule key: {key}"));

            NodejsRelease {
                range,
                end_of_life: parse_date(&entry.end),
                metadata: NodejsReleaseMetadata {
                    start: parse_date(&entry.start),
                    lts: entry.lts.as_deref().map(parse_date),
                    maintenance: entry.maintenance.as_deref().map(parse_date),
                },
            }
        })
        .collect()
}
