use clap::{arg, command, value_parser};
use keep_a_changelog_file::{ChangeGroup, Changelog};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

fn main() {
    let matches = command!()
        .arg(
            arg!(<changelog_path>)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(arg!(<commit_sha>).required(true))
        .get_matches();

    let changelog_path = matches
        .get_one::<PathBuf>("changelog_path")
        .expect("should be the first required argument");

    let commit_sha = matches
        .get_one::<String>("commit_sha")
        .expect("should be the second required argument");

    eprintln!("Configuration:");
    eprintln!("  Changelog path: {}", changelog_path.display());
    eprintln!("  Commit SHA: {commit_sha}");

    eprintln!("Writing changelog...");
    let changelog_contents = fs::read_to_string(changelog_path)
        .unwrap_or_else(|_| panic!("Failed to read changelog at '{}'", changelog_path.display()));
    let mut changelog = Changelog::from_str(&changelog_contents)
        .unwrap_or_else(|_| panic!("Error parsing changelog at '{}'", changelog_path.display()));

    let change_description =
        format!("Automated sync of `@yarnpkg/plugin-prune-dev-dependencies.js` from {commit_sha}");
    changelog
        .unreleased
        .add(ChangeGroup::Changed, &change_description);

    println!("{change_description}");
    fs::write(changelog_path, changelog.to_string()).unwrap_or_else(|_| {
        panic!(
            "Failed to write to changelog to '{}'",
            changelog_path.display()
        )
    });
}
