use crate::proto::nexus_service_client::NexusServiceClient;
use crate::proto::{CommandRequest, ListServicesRequest, ServiceInfo};
use rustyline::DefaultEditor;

/// Interactive CLI shell that connects to a Nexus gRPC server.
pub struct NexusCli {
    addr: String,
}

impl NexusCli {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let mut client = NexusServiceClient::connect(self.addr).await?;

        // Fetch available services on startup.
        let services = client
            .list_services(ListServicesRequest {})
            .await?
            .into_inner()
            .services;

        println!("Connected. Type 'help' for available commands, 'quit' to exit.");

        let mut rl = DefaultEditor::new()?;

        loop {
            let line = match rl.readline("cli> ") {
                Ok(line) => line,
                Err(rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof) => break,
                Err(e) => return Err(e.into()),
            };

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let _ = rl.add_history_entry(line);

            if line == "quit" || line == "exit" {
                break;
            }

            if line == "help" {
                print_help(&services);
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                println!("Usage: <service> <command> [args...]");
                continue;
            }

            let service = parts[0].to_string();
            let action = parts[1].to_string();
            let args: Vec<String> = parts[2..].iter().map(|s| s.to_string()).collect();

            let response = client
                .execute(CommandRequest {
                    service,
                    action,
                    args,
                })
                .await?
                .into_inner();

            if response.success {
                println!("{}", response.message);
            } else {
                println!("Error: {}", response.message);
            }
        }

        Ok(())
    }
}

fn print_help(services: &[ServiceInfo]) {
    println!("Available commands:");
    for svc in services {
        println!("  {}:", svc.name);
        for cmd in &svc.commands {
            let args_str = cmd
                .args
                .iter()
                .map(|a| format!("<{}>", a))
                .collect::<Vec<_>>()
                .join(" ");
            let desc = if cmd.description.is_empty() {
                String::new()
            } else {
                format!(" - {}", cmd.description)
            };
            println!("    {} {}{}", cmd.name, args_str, desc);
        }
    }
}
