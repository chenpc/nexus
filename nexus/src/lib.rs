pub mod registry;
pub mod server;
pub mod cli;

pub mod proto {
    tonic::include_proto!("nexus");
}

pub use registry::{CommandInfo, Service};
pub use server::NexusServer;
pub use cli::NexusCli;
pub use nexus_derive::nexus_service;
