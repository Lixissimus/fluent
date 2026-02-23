use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, anyhow};

fn main() -> anyhow::Result<()> {
    let configs = [
        Path::new("/etc/interception/fluent.d/fluent.json"),
        Path::new("/etc/interception/fluent.json"),
    ];

    let config = configs.iter().find(|path| path.is_file());
    let Some(config) = config else {
        return Err(anyhow!("no config found, tried: {:?}", configs));
    };
    let file = File::open(config).context("could not open config file")?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader).context("could not parse config file")?;

    fluent::run(&mut std::io::stdin(), &mut std::io::stdout(), &config)?;
    Ok(())
}
