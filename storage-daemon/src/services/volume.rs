use libnexus::nexus_service;

pub struct Volume;

#[nexus_service]
impl Volume {
    /// Create a new volume on the specified disk.
    #[command]
    async fn create(
        &self,
        #[arg(hint = "volume name")] name: String,
        #[arg(hint = "device", complete = "block.list")] disk: String,
    ) -> anyhow::Result<String> {
        Ok(format!("Volume '{}' created on disk '{}'", name, disk))
    }

    /// Delete an existing volume.
    #[command]
    async fn delete(&self, name: String) -> anyhow::Result<String> {
        Ok(format!("Volume '{}' deleted", name))
    }

    /// List all volumes.
    #[command]
    async fn list(&self) -> anyhow::Result<String> {
        Ok("vol0, vol1, vol2".to_string())
    }
}
