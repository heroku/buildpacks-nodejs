use libnodejs::resolve_version::Software;
use node_semver::{Range, SemverError};

const SUCCESS_EXIT_CODE: i32 = 0;
const ARGS_EXIT_CODE: i32 = 1;
const VERSION_REQS_EXIT_CODE: i32 = 2;
const IO_EXIT_CODE: i32 = 3;
const TOML_EXIT_CODE: i32 = 4;
const ARCH: &str = "x64";
const OS: &str = "linux";

/// Translates the semver requirements from `package.json` into `semver-node::Range`. It handles
/// these cases:
///
/// * "latest" as "*"
/// * "~=" as "="
/// * any other semver compatible requirements will get parsed
///
/// # Failures
/// Invalid semver requirement wil return an error
fn translate_version_requirements(requirement: &str) -> Result<Range, SemverError> {
    let trimmed = requirement.trim();

    if requirement == "latest" {
        Ok(Range::any())
    } else if let Ok(range) = Range::parse(&trimmed) {
        Ok(range)
    } else if trimmed.starts_with("~=") {
        let version = trimmed.replacen("=", "", 1);
        Range::parse(version)
    } else {
        Range::parse(&trimmed)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if &args[1] == "-v" || &args[1] == "--version" {
        const VERSION: &'static str = env!("CARGO_PKG_VERSION");
        println!("v{}", VERSION);
        std::process::exit(SUCCESS_EXIT_CODE);
    }

    if args.len() < 3 {
        eprintln!("$ resolve <toml file> <version requirements>");
        std::process::exit(ARGS_EXIT_CODE);
    }
    let filename = &args[1];
    let version_requirements = translate_version_requirements(&args[2]).unwrap_or_else(|e| {
        eprintln!("Could not parse Version Requirements '{}': {}", &args[2], e);
        std::process::exit(VERSION_REQS_EXIT_CODE);
    });

    let contents = std::fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Could not read file '{}': {}", filename, e);
        std::process::exit(IO_EXIT_CODE);
    });
    let software: Software = toml::from_str(&contents).unwrap_or_else(|e| {
        eprintln!("Could not parse toml of '{}': {}", filename, e);
        std::process::exit(TOML_EXIT_CODE);
    });

    let current_arch = format!("{}-{}", OS, ARCH);
    let version = software.resolve(version_requirements, &current_arch, "release");
    if let Some(version) = version {
        println!("{} {}", version.version, version.url);
    } else {
        println!("No result");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_latest() {
        let result = translate_version_requirements("latest");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("*", format!("{}", reqs));
        }
    }

    #[test]
    fn it_handles_exact_versions() {
        let result = translate_version_requirements("14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn it_handles_starts_with_v() {
        let result = translate_version_requirements("v14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn it_handles_semver_semantics() {
        let result = translate_version_requirements(">= 12.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=12.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn it_handles_pipe_statements() {
        let result = translate_version_requirements("^12 || ^13 || ^14");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(
                ">=12.0.0 <13.0.0-0||>=13.0.0 <14.0.0-0||>=14.0.0 <15.0.0-0",
                format!("{}", reqs)
            );
        }
    }

    #[test]
    fn it_handles_tilde_with_equals() {
        let result = translate_version_requirements("~=14.4");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.0 <14.5.0-0", format!("{}", reqs));
        }
    }

    #[test]
    fn it_handles_tilde_with_equals_and_patch() {
        let result = translate_version_requirements("~=14.4.3");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.3 <14.5.0-0", format!("{}", reqs));
        }
    }

    #[test]
    fn it_handles_v_within_string() {
        let result = translate_version_requirements(">v15.5.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">15.5.0", format!("{}", reqs));
        }
    }

    #[test]
    fn it_handles_v_with_space() {
        let result = translate_version_requirements(">= v10.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=10.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn it_handles_equal_with_v() {
        let result = translate_version_requirements("=v10.22.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("10.22.0", format!("{}", reqs));
        }
    }

    #[test]
    fn it_returns_error_for_invalid_reqs() {
        let result = translate_version_requirements("12.%");
        println!("{:?}", result);

        assert!(result.is_err());
    }
}
