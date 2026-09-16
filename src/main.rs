use std::{fs, path::Path};

use anyhow::{Context, anyhow};

fn main() -> anyhow::Result<()> {
    let configs = [
        Path::new("/etc/interception/fluent.d/fluent.conf"),
        Path::new("/etc/interception/fluent.conf"),
    ];

    let config = configs.iter().find(|path| path.is_file());
    let Some(config) = config else {
        return Err(anyhow!("no config found, tried: {:?}", configs));
    };
    let input = fs::read_to_string(config).context("could not read config file")?;
    let config = fluent::config::parse(&input).context("could not parse config file")?;

    fluent::run(&mut std::io::stdin(), &mut std::io::stdout(), &config)?;
    Ok(())
}
