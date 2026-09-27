use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Socket for communication with fluentctl
    ///
    /// The default value is fine as long as you don't also change the socket when running fluentctl
    #[arg(long, default_value = "/tmp/fluent-if.sock")]
    pub if_socket: PathBuf,

    /// Socket for communication with the fluent instances
    ///
    /// The default value is fine as long as you don't also change the socket when running the instances
    #[arg(long, default_value = "/tmp/fluent.sock")]
    pub inst_socket: PathBuf,
}
