use libcnb::read_toml_file;
use libherokubuildpack::toml_select_value;
use std::path::PathBuf;
use toml::Value;

pub fn is_function<P>(d: P) -> bool
where
    P: Into<PathBuf>,
{
    let dir: PathBuf = d.into();
    println!("dir: {:?}", dir);
    dir.join("function.toml").exists() || {
        read_toml_file(dir.join("project.toml"))
            .ok()
            .and_then(|toml: Value| {
                toml_select_value(vec!["com", "salesforce", "type"], &toml)
                    .and_then(toml::Value::as_str)
                    .map(|value| value == "function")
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::{tempdir, TempDir};

    fn create_dir_with_file(file_name: &str, file_contents: &str) -> TempDir {
        let dir = tempdir().expect("Couldn't create temp dir");
        let path: PathBuf = dir.path().into();
        let file_path = path.join(file_name);
        let mut file = File::create(&file_path).expect("Couldn't create temp project descriptor");
        write!(file, "{}", file_contents).expect("Couldn't write to temp project descriptor");
        println!("wrote file at {:?}", file_path);
        dir
    }

    #[test]
    fn function_toml_is_function() {
        let dir = create_dir_with_file("function.toml", "");
        assert!(is_function(dir.path()))
    }

    #[test]
    fn project_toml_is_function() {
        let dir = create_dir_with_file("project.toml", "[com.salesforce]\n type = \"function\"");
        assert!(is_function(dir.path()))
    }

    #[test]
    fn project_toml_with_diff_type_is_not_function() {
        let dir = create_dir_with_file("project.toml", "[com.salesforce]\n type = \"wubalubdub\"");
        assert!(!is_function(dir.path()))
    }

    #[test]
    fn no_descriptor_is_not_function() {
        let dir = create_dir_with_file("package.json", "");
        assert!(!is_function(dir.path()))
    }
}
