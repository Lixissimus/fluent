use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::{Context, bail};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "/etc/interception/fluent.json")]
    config: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !args.config.is_file() {
        bail!("file not found: {:?}", args.config);
    }
    let file = File::open(args.config).context("could not open config file")?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader).context("could not parse config file")?;

    fluent::run(&mut std::io::stdin(), &mut std::io::stdout(), &config)?;
    Ok(())
}
