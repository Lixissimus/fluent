use std::fs::{self, File};

use fluent::config::Config;

#[test]
fn validate_examples() {
    let paths = fs::read_dir("examples")
        .unwrap()
        .flatten()
        .map(|f| f.path())
        .filter(|p| p.extension().unwrap_or_default() == "json");
    for path in paths {
        let reader = File::open(path.clone()).unwrap();
        let _: Config = serde_json::from_reader(reader).expect(&format!(
            "example config invalid: {}",
            path.to_str().unwrap()
        ));
    }
}
