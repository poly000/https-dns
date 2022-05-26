use crate::cli::Args;
use crate::local::UdpListener;
use crate::upstream::HttpsClient;
use clap::Parser;
use std::process::exit;
use tracing::error;

mod bootstrap;
mod cache;
mod cli;
mod error;
mod local;
mod upstream;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().compact().init();

    let Args {
        upstream_address,
        local_address,
        local_port,
        upstream_port,
    } = cli::Args::parse();

    let https_client = match HttpsClient::new(upstream_address, upstream_port).await {
        Ok(https_client) => https_client,
        Err(error) => {
            error!("{}", error);
            exit(1);
        }
    };

    let udp_listener = match UdpListener::new(local_address, local_port, https_client).await {
        Ok(udp_listener) => udp_listener,
        Err(error) => {
            error!("{}", error);
            exit(1);
        }
    };
    udp_listener.listen().await;
}
