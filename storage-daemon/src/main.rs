mod services;

use libnexus::NexusServer;
use services::{Block, Network, Pool, Volume};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| libnexus::DEFAULT_ENDPOINT.to_string());

    NexusServer::new()
        .register(Volume)
        .register(Block)
        .register(Network)
        .register(Pool)
        .serve(&addr)
        .await
}
