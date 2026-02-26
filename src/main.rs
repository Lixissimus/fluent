use std::{fs::File, io::BufReader};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let file = File::open("/etc/fluent/config.json").context("could not open config file")?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader).context("could not parse config file")?;

    fluent::run(&mut std::io::stdin(), &mut std::io::stdout(), &config)?;
    Ok(())
}
