use crate::local::LocalUdpSocket;
use crate::upstream::UpstreamHttpsClient;
use clap::Parser;

mod bootstrap;
mod cache;
mod cli;
mod error;
mod local;
mod upstream;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    let upstream_address = args.upstream_address;
    let upstream_port = args.upstream_port;
    let upstream_https_client = UpstreamHttpsClient::new(upstream_address, upstream_port).await;

    let local_address = args.local_address;
    let local_port = args.local_port;
    let local_udp_socket =
        LocalUdpSocket::new(local_address, local_port, upstream_https_client).await;

    local_udp_socket.listen().await;
}
