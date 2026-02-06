use libnexus::{nexus_service, NexusServer};

pub struct Volume;

#[nexus_service]
impl Volume {
    /// Create a new volume on the specified disk.
    #[command]
    async fn create(&self, name: String, disk: String) -> anyhow::Result<String> {
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

pub struct Pool;

#[nexus_service]
impl Pool {
    /// Create a new storage pool.
    #[command]
    async fn create(&self, name: String) -> anyhow::Result<String> {
        Ok(format!("Pool '{}' created", name))
    }

    /// Destroy a storage pool.
    #[command]
    async fn destroy(&self, name: String) -> anyhow::Result<String> {
        Ok(format!("Pool '{}' destroyed", name))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    NexusServer::new()
        .register(Volume)
        .register(Pool)
        .serve("[::1]:50051")
        .await
}
