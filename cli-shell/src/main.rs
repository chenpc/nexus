use libnexus::NexusCli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    NexusCli::new("http://[::1]:50051").run().await
}
