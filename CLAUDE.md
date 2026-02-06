# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nexus is a Rust framework for building gRPC-based microservices with a macro-driven command dispatch system. Services register commands via a `#[nexus_service]` proc macro, which are then accessible through both a gRPC server and an interactive CLI client.

## Build & Development Commands

```bash
cargo build                                    # Build entire workspace
cargo build --release                          # Optimized build
cargo test                                     # Run all tests
cargo test -p libnexus                         # Run tests for core library crate
cargo test -p nexus-derive                     # Run tests for derive macro crate
cargo run -p storage-daemon                    # Run example gRPC server
cargo run -p cli-shell                         # Run example CLI client
```

Proto files are compiled automatically via `libnexus/build.rs` using `tonic-build` during `cargo build`.

## Workspace Structure

Four crates in a Cargo workspace:

- **`libnexus/`** — Core framework library: gRPC server (`server.rs`), service registry (`registry.rs`), CLI client (`cli.rs`), and protobuf definitions (`proto/nexus.proto`)
- **`libnexus/nexus-derive/`** — Proc macro crate providing `#[nexus_service]` and `#[command]` attribute macros
- **`storage-daemon/`** — Example gRPC server with Volume and Pool services
- **`cli-shell/`** — Example CLI client connecting to the server

## Architecture

### Command Dispatch Flow

1. User defines a struct and annotates its `impl` block with `#[nexus_service]`
2. Methods marked `#[command]` are extracted by the proc macro, which generates:
   - `CommandInfo` metadata (name, args, description from doc comments)
   - A `Service` trait implementation with dispatch match arms
   - Position-based argument extraction from `Vec<String>`
3. Services are registered into a `Registry` (a `HashMap<String, Box<dyn Service>>`)
4. `NexusServer` wraps the registry, translating gRPC `Execute`/`ListServices` RPCs into registry calls
5. `NexusCli` connects to the server and provides a REPL with command `<service> <command> [args...]`

### Key Traits and Types

- **`Service` trait** (`libnexus/src/registry.rs`): `name()`, `commands()`, `execute(action, args)` — all async
- **`Registry`** (`libnexus/src/registry.rs`): holds registered services, dispatches by service name
- **`NexusServer`** (`libnexus/src/server.rs`): builder pattern for registering services and starting the gRPC server
- **`NexusCli`** (`libnexus/src/cli.rs`): connects to server, fetches service list, runs interactive REPL

### Key Conventions

- All service commands are async and return `anyhow::Result<String>`
- Arguments are string-based (`Vec<String>`) for uniform gRPC/CLI transport
- gRPC protocol defined in `libnexus/proto/nexus.proto` with `NexusService` having `Execute` and `ListServices` RPCs
- Uses tonic 0.12 / prost 0.13 for gRPC, tokio for async runtime
