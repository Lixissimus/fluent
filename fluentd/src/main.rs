use clap::Parser;
use fluentd::args::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    fluentd::run(args).await
}
