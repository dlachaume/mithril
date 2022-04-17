#![doc = include_str!("../README.md")]

mod apispec;
mod entities;
mod fake_data;
mod http_server;
mod snapshotter;

use clap::Parser;
use std::thread;

use crate::http_server::Server;
use crate::snapshotter::Snapshotter;

/// Node args
#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Server listening IP
    #[clap(long, default_value = "0.0.0.0")]
    server_ip: String,

    /// Server listening port
    #[clap(long, default_value_t = 8080)]
    server_port: u16,

    /// Verbosity level
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Snapshot interval, in seconds
    #[clap(long, default_value_t = 10000)]
    snapshot_interval: u32,
}

#[tokio::main]
async fn main() {
    // Load args
    let args = Args::parse();

    // Init logger
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .filter_level(args.verbose.log_level_filter())
        .init();

    println!("Starting server...");
    println!("Press Ctrl+C to stop...");
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    };

    // Start snapshot uploader
    let snapshotter = Snapshotter::new(args.snapshot_interval);
    let stopper = snapshotter.stopper();

    thread::spawn(move || snapshotter.run());

    // Start REST server
    let http_server = Server::new(args.server_ip, args.server_port);
    http_server.start(shutdown_signal).await;

    stopper.stop();

    println!("Exiting...");
}
