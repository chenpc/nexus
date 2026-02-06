use crate::proto::nexus_service_server::{NexusService, NexusServiceServer};
use crate::proto::{
    CommandDef, CommandRequest, CommandResponse, ListServicesRequest, ListServicesResponse,
    ServiceInfo,
};
use crate::registry::{Registry, Service};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// gRPC server wrapping a service registry.
pub struct NexusServer {
    registry: Arc<Registry>,
}

impl NexusServer {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Registry::new()),
        }
    }

    /// Register a service with the server. Must be called before `serve`.
    pub fn register<S: Service>(mut self, service: S) -> Self {
        Arc::get_mut(&mut self.registry)
            .expect("register must be called before serve")
            .register(service);
        self
    }

    /// Start the gRPC server on the given address.
    pub async fn serve(self, addr: &str) -> anyhow::Result<()> {
        let addr = addr.parse()?;
        let grpc_service = NexusGrpcService {
            registry: self.registry,
        };
        println!("Nexus server listening on {}", addr);
        tonic::transport::Server::builder()
            .add_service(NexusServiceServer::new(grpc_service))
            .serve(addr)
            .await?;
        Ok(())
    }
}

struct NexusGrpcService {
    registry: Arc<Registry>,
}

#[tonic::async_trait]
impl NexusService for NexusGrpcService {
    async fn execute(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let req = request.into_inner();
        match self.registry.execute(&req.service, &req.action, req.args).await {
            Ok(message) => Ok(Response::new(CommandResponse {
                success: true,
                message,
            })),
            Err(e) => Ok(Response::new(CommandResponse {
                success: false,
                message: e.to_string(),
            })),
        }
    }

    async fn list_services(
        &self,
        _request: Request<ListServicesRequest>,
    ) -> Result<Response<ListServicesResponse>, Status> {
        let services = self
            .registry
            .list_services()
            .into_iter()
            .map(|(name, commands)| ServiceInfo {
                name: name.to_string(),
                commands: commands
                    .into_iter()
                    .map(|c| CommandDef {
                        name: c.name,
                        args: c.args,
                        description: c.description,
                    })
                    .collect(),
            })
            .collect();

        Ok(Response::new(ListServicesResponse { services }))
    }
}
