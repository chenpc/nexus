# Nexus

A Rust framework for building gRPC-based microservices with macro-driven command dispatch.

## Prerequisites

### Ubuntu/Debian

```bash
sudo apt-get update && sudo apt-get install -y build-essential protobuf-compiler
```

### Rust

Requires Rust 1.85+ (for edition 2024 support). Install via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build

```bash
cargo build
```

## Example

```bash
# Start the example server
cargo run -p volume-example --bin server

# In another terminal, start the CLI client
cargo run -p volume-example --bin cli
```
