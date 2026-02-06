use libnexus::nexus_service;

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
