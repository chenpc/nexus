use libnexus::nexus_service;

pub struct Block;

#[nexus_service]
impl Block {
    /// List all block devices.
    #[command]
    async fn list(&self) -> anyhow::Result<String> {
        Ok("sda, sdb, sdc, nvme0n1".to_string())
    }

    /// Show info for a block device.
    #[command]
    async fn info(&self, device: String) -> anyhow::Result<String> {
        Ok(format!("Block device '{}': size=500G, type=SSD", device))
    }
}
