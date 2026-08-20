# Helium - P2P Compute Marketplace

A decentralized peer-to-peer marketplace for sharing compute resources (CPU, RAM, GPU) among trusted community members.

## Architecture

Helium is organized as a Rust workspace with multiple crates:

```
helium/
├── crates/
│   ├── helium-core/     # P2P discovery and resource matching
│   ├── helium-tunnel/     # WireGuard P2P tunnels
│   ├── helium-vm/         # Firecracker microVM orchestration
│   └── helium-cli/        # Command-line interface
├── docs/                  # Documentation
└── scripts/               # Installation and setup scripts
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Linux kernel 5.10+ (for Firecracker)
- WireGuard tools

### Installation

```bash
# Clone repository
git clone https://github.com/helium-project/helium
cd helium

# Build release binary
cargo build --release

# Install binary
cargo install --path crates/helium-cli

# Initialize node
helium init --node-type provider  # or "borrower"

# Start node
helium start
```

## Usage

### Provider (Share Resources)

```bash
# Advertise 32GB RAM at $0.50/hour
helium advertise --resource-type ram --capacity 32 --price 0.50

# List active connections
helium peers
```

### Borrower (Use Resources)

```bash
# Request 16GB RAM for 2 hours, max $1/hour
helium request --resource-type ram --min-capacity 16 --max-price 1.0 --duration 2

# Connect to a provider
helium connect --peer-id <PROVIDER_ID>

# Create VM on provider
helium vm create --vcpus 4 --memory 8192
```

### VM Management

```bash
# List VMs
helium vm list

# Start/Stop VMs
helium vm start <VM_ID>
helium vm stop <VM_ID>
```

### Tunnel Management

```bash
# List active P2P tunnels
helium tunnel list

# Create tunnel to peer
helium tunnel create --peer-id <PEER> --endpoint <IP:PORT>
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test --workspace

# Run with logging
RUST_LOG=debug cargo run --bin helium -- start
```

### Project Structure

- **helium-core**: libp2p-based peer discovery, DHT, resource matching engine
- **helium-tunnel**: WireGuard interface management, NAT traversal, hole punching
- **helium-vm**: Firecracker API client, cgroups resource management
- **helium-cli**: User interface, command parsing, status display

### Key Technologies

- **P2P Networking**: libp2p (mDNS, Kademlia DHT, Noise encryption)
- **Tunnels**: WireGuard (kernel module + userspace)
- **Isolation**: AWS Firecracker microVMs
- **Database**: SQLite (local credits, peer cache)
- **Async Runtime**: Tokio

## Security

- All P2P connections use WireGuard with automatically rotated keys
- VM isolation via Firecracker (KVM-based microVMs)
- Resource limits enforced via cgroups v2
- No blockchain - local SQLite credits only

## Roadmap

See [ROADMAP.md](./ROADMAP.md) for development timeline.

## License

MIT License - See LICENSE file

## Contributing

Contributions welcome! Please read CONTRIBUTING.md first.
