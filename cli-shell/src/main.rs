use libnexus::NexusCli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| libnexus::DEFAULT_ENDPOINT.to_string());

    NexusCli::new(&addr).run().await
}
