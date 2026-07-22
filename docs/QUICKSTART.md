# bkg-peer Quickstart Guide

Get up and running with bkg-peer in minutes.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourorg/bkg-peer.git
cd bkg-peer

# Build release binary
cargo build --release

# Binary is at ./target/release/bkg-peer
```

### Verify Installation

```bash
./target/release/bkg-peer version
# bkg-peer 0.2.0
```

## Quick Start

### 1. Download a Model

```bash
mkdir -p ~/.bkg-peer/models

# Llama 3.2 1B (~770MB) - fast, good for testing
curl -L -o ~/.bkg-peer/models/llama-3.2-1b-instruct-q4_k_m.gguf \
  "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf"

# Llama 3.2 3B (~2GB) - better quality
curl -L -o ~/.bkg-peer/models/llama-3.2-3b-instruct-q4_k_m.gguf \
  "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf"
```

### 2. Run Interactive Chat

```bash
# Ollama-style quick chat
bkg-peer run llama-3.2-1b

# Full-featured chat with slash commands
bkg-peer chat
```

### 3. Create a Wallet

Every peer needs an identity (Ed25519 keypair):

```bash
bkg-peer wallet create
```

Output:
```
Wallet created successfully!
  Address: 12D3KooWQL62BcJz9zqRNRnDkKfYiHSdSUG5n7LZ4xRZBPPDT9at
  Keyfile: ~/.bkg-peer/identity.key
  Balance: 0.000000 PCLAW
```

### 4. Start a Node

Start your peer node to join the network:

```bash
# Basic node
bkg-peer serve

# With web dashboard
bkg-peer serve --web 127.0.0.1:8080

# As a resource provider (accept jobs)
bkg-peer serve --provider
```

Output:
```
INFO  Starting bkg-peer node...
INFO  Peer ID: 12D3KooWQL62BcJz9zqRNRnDkKfYiHSdSUG5n7LZ4xRZBPPDT9at
INFO  Listening on /ip4/0.0.0.0/tcp/0
INFO  Web dashboard at http://127.0.0.1:8080
INFO  Node running. Press Ctrl+C to stop.
```

## Chat Mode

The full-featured chat mode supports slash commands:

```bash
bkg-peer chat
```

### Slash Commands

| Command | Description |
|---------|-------------|
| `/help` | Show all commands |
| `/model <name>` | Switch model |
| `/temperature <n>` | Set temperature (0.0-2.0) |
| `/max_tokens <n>` | Set max output tokens |
| `/settings` | Open settings menu |
| `/status` | Show runtime status |
| `/peers` | List connected peers |
| `/balance` | Show token balance |
| `/tools` | List available tools |
| `/tool_exec <name> <args>` | Execute a tool |
| `/distributed <on\|off>` | Toggle network inference |
| `/stream <on\|off>` | Toggle streaming output |
| `/clear` | Clear conversation history |
| `/history` | Show conversation history |
| `/export <file>` | Export conversation |

## Vector Memory

Store and search information semantically:

```bash
# Create a collection
bkg-peer vector create notes

# Insert data
bkg-peer vector insert notes "bkg-peer uses libp2p for networking"
bkg-peer vector insert notes "WASM sandbox provides tool isolation"
bkg-peer vector insert notes "Vector search uses HNSW indexing"

# Semantic search
bkg-peer vector search notes "how does networking work" -k 3
```

## Skills

Install and manage prompt extensions:

```bash
# List installed skills
bkg-peer skill list

# Install a skill
bkg-peer skill install ./skills/code-review.md

# Search network for skills
bkg-peer skill search "data analysis"
```

## CLI Commands Reference

### Model Commands

```bash
bkg-peer models list              # List downloaded models
bkg-peer models download <model>  # Download from HuggingFace
bkg-peer pull <model>             # Alias for download
```

### Wallet Commands

```bash
bkg-peer wallet create            # Create new wallet
bkg-peer wallet info              # Show wallet info
bkg-peer wallet balance           # Check balance
bkg-peer wallet send <addr> <amt> # Send tokens
bkg-peer wallet history           # Transaction history
bkg-peer wallet escrows           # Show active escrows
```

### Network Commands

```bash
bkg-peer network status           # Show network status
bkg-peer peers list               # List connected peers
bkg-peer network discover         # Force peer discovery
```

### Job Commands

```bash
bkg-peer job submit <spec>        # Submit job to network
bkg-peer job status <id>          # Check job status
bkg-peer job list                 # List active jobs
```

### Agent Commands

```bash
bkg-peer agent run agent.toml     # Run an agent
bkg-peer agent list               # List running agents
bkg-peer agent logs <id>          # View agent logs
bkg-peer agent stop <id>          # Stop an agent
bkg-peer agent attach <id>        # Attach to agent REPL
```

### Tool Commands

```bash
bkg-peer tool list                # List available tools
bkg-peer tool info <name>         # Show tool details
bkg-peer tool build <path>        # Build WASM tool
bkg-peer tool install <path>      # Install WASM tool
```

## Running Multiple Nodes (P2P Testing)

To test P2P features locally, run multiple nodes with separate data directories:

### Terminal 1 - Node A

```bash
mkdir -p /tmp/bkg-peer-node-a
BKG_PEER_HOME=/tmp/bkg-peer-node-a bkg-peer wallet create
BKG_PEER_HOME=/tmp/bkg-peer-node-a bkg-peer serve --web 127.0.0.1:8080
```

### Terminal 2 - Node B

```bash
mkdir -p /tmp/bkg-peer-node-b
BKG_PEER_HOME=/tmp/bkg-peer-node-b bkg-peer wallet create
BKG_PEER_HOME=/tmp/bkg-peer-node-b bkg-peer serve --web 127.0.0.1:8081
```

Nodes automatically discover each other via mDNS on the local network.

### Test Cluster (Automated)

```bash
bkg-peer test cluster --nodes 3
```

This spawns 3 nodes with separate data directories for testing.

## OpenAI-Compatible API

Start a node with the web server enabled:

```bash
bkg-peer serve --web 127.0.0.1:8080
```

Use any OpenAI SDK:

```python
from openai import OpenAI

client = OpenAI(base_url="http://localhost:8080/v1", api_key="unused")
response = client.chat.completions.create(
    model="llama-3.2-3b",
    messages=[{"role": "user", "content": "Hello!"}],
    stream=True
)
for chunk in response:
    print(chunk.choices[0].delta.content, end="")
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BKG_PEER_HOME` | Base directory for data | `~/.bkg-peer` |
| `BKG_PEER_LOG` | Log level | `info` |

### Config File

Create `~/.bkg-peer/config.toml`:

```toml
[p2p]
listen_addresses = ["/ip4/0.0.0.0/tcp/0"]
bootstrap_peers = []
mdns_enabled = true

[web]
enabled = false
listen_addr = "127.0.0.1:8080"

[inference]
models_path = "~/.bkg-peer/models"
default_model = "llama-3.2-3b"

[vector]
embedding_dim = 384
persistence_path = "~/.bkg-peer/vector"

[safety]
leak_detection = true
injection_defense = true
```

## Example: Agent Configuration

Create an agent specification in `agent.toml`:

```toml
[agent]
name = "my-assistant"
version = "0.1.0"
description = "A helpful AI assistant"

[model]
provider = "local"
model = "llama-3.2-3b"
max_tokens_per_request = 2048

[budget]
max_spend_per_hour = 100
max_spend_total = 1000

[capabilities]
web_access = true
vector_memory = true

[web_access]
allowed_hosts = ["*.wikipedia.org", "arxiv.org"]
max_requests_per_minute = 10

[tools]
builtin = ["web_fetch", "memory_search", "memory_write"]

[channels]
repl = true

[memory]
collection = "assistant-memory"
auto_persist = true
```

Run the agent:

```bash
bkg-peer agent run agent.toml
```

## Troubleshooting

### Node won't start

Check if another instance is running:
```bash
ps aux | grep bkg-peer
```

### Peers not discovering each other

1. Ensure both nodes are on the same network
2. Check firewall settings - mDNS uses UDP port 5353
3. Verify mDNS is enabled in config

### Model loading fails

1. Check model file exists in `~/.bkg-peer/models/`
2. Ensure enough RAM for model size
3. Check file isn't corrupted (re-download)

### Database errors

Reset the database:
```bash
rm -rf ~/.bkg-peer/data/bkg-peer.redb
```

### Out of memory

Reduce model size or use quantized versions:
- Q4_K_M: Good balance of quality and memory
- Q4_0: Smaller, slightly lower quality
- Q8_0: Higher quality, more memory

## Next Steps

- Read the [Architecture](ARCHITECTURE.md) for system design
- Explore [Token Economy](TOKENS.md) to understand PCLAW tokens
- Check [Security](SECURITY.md) for safety features
- Learn about [Agents](AGENTS.md) for autonomous AI deployment

---

*bkg-peer v0.2 — Quickstart Guide*
