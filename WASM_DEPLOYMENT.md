# WebAssembly Deployment Guide

## Overview

Freenet Lottery can be deployed in both **native (server-side)** and **browser (WebAssembly)** environments. This guide covers deployment options and technology choices.

## Technology Maturity Status (2024)

| Component | Browser/WASM | Status | Recommendation |
|-----------|--------------|--------|----------------|
| **libp2p** | ✅ Yes (Rust/JS) | **Production-Ready** | Use for P2P networking |
| **IPFS** | ✅ Yes (js-ipfs) | **Production-Ready** | Use for content storage |
| **Freenet** | ❌ No (Native only) | Experimental | Use for server-side state sync |

## Deployment Architecture

### Option 1: Native Server (Recommended for MVP)

```
┌─────────────────────────────────────────────────┐
│          User's Browser                         │
│  ┌─────────────────────────────────────────┐   │
│  │  React Frontend (TypeScript/React)      │   │
│  │  ├─ Submit tasks                        │   │
│  │  ├─ View results                        │   │
│  │  └─ Display lottery outcomes            │   │
│  └─────────────────────────────────────────┘   │
│              ↑                                   │
│              │ HTTP/WebSocket                   │
│              ↓                                   │
├─────────────────────────────────────────────────┤
│         Coordinator Node (Native Rust)          │
│  ├─ Task decomposition                          │
│  ├─ Freenet Scaffold state sync                 │
│  └─ Lottery drawing                             │
├─────────────────────────────────────────────────┤
│              libp2p Network                      │
│  ├─ Task broadcast via pubsub                   │
│  ├─ Worker discovery via DHT                    │
│  └─ Result collection                           │
├──────────────────���──────────────────────────────┤
│           Worker Nodes (Native Rust)            │
│  ├─ Segment processing                          │
│  ├─ Proof-of-work generation                    │
│  └─ Result upload to IPFS                       │
├─────────────────────────────────────────────────┤
│         IPFS Network (js-ipfs or Kubo)          │
│  ├─ Content storage & retrieval                 │
│  ├─ Segment seeding                             │
│  └─ Result distribution                         │
└─────────────────────────────────────────────────┘
```

### Option 2: Hybrid (Browser Workers)

```
┌─────────────────────────────────────────────────┐
│          User's Browser                         │
│  ┌─────────────────────────────────────────┐   │
│  │  React Frontend                         │   │
│  └─────────────────────────────────────────┘   │
│              ↑              ↓                    │
│              │              │                    │
│       ┌──────┴──────────────┴──────┐            │
│       │  Worker (WASM/libp2p)      │            │
│       │  ├─ Segment processing     │            │
│       │  ├─ WebRTC transport       │            │
│       │  └─ IPFS (js-ipfs client)  │            │
│       └──────────────────────────────┘          │
│              ↑              ↑                    │
│              │              │                    │
└──────────────┼──────────────┼──────────────────┘
               │              │
               ↓ WebRTC       ↓ WebSocket
    ┌─────────────────────────────────┐
    │   Coordinator (Native Rust)     │
    │   - libp2p node                 │
    │   - Freenet Scaffold state      │
    │   - Lottery engine              │
    └─────────────────────────────────┘
               ↓
    ┌─────────────────────────────────┐
    │   IPFS Network                  │
    │   - Kubo, Helia, js-ipfs        │
    └─────────────────────────────────┘
```

## Native Deployment (Recommended)

### Coordinator Node (Docker)

```dockerfile
# Dockerfile.coordinator
FROM rust:1.75-slim

WORKDIR /app
COPY . .

RUN cargo build --release -p coordinator

EXPOSE 8080
CMD ["./target/release/coordinator", "--id=coord-1"]
```

### Worker Node (Docker)

```dockerfile
# Dockerfile.worker
FROM rust:1.75-slim

WORKDIR /app
COPY . .

RUN cargo build --release -p worker

CMD ["./target/release/worker", "--id=worker-1"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  coordinator:
    build:
      context: .
      dockerfile: docker/coordinator.Dockerfile
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=debug
    networks:
      - freenet

  worker-1:
    build:
      context: .
      dockerfile: docker/worker.Dockerfile
    environment:
      - RUST_LOG=debug
    networks:
      - freenet
    depends_on:
      - coordinator

  worker-2:
    build:
      context: .
      dockerfile: docker/worker.Dockerfile
    environment:
      - RUST_LOG=debug
    networks:
      - freenet
    depends_on:
      - coordinator

  ipfs:
    image: ipfs/kubo:latest
    ports:
      - "4001:4001"
      - "5001:5001"
      - "8081:8080"
    networks:
      - freenet

networks:
  freenet:
    driver: bridge
```

### Run Locally

```bash
# Start IPFS daemon
ipfs daemon

# Terminal 1: Coordinator
cd backend/coordinator
cargo run --release -- --id=coord-1

# Terminal 2: Worker 1
cd backend/worker
cargo run --release -- --id=worker-1

# Terminal 3: Worker 2
cd backend/worker
cargo run --release -- --id=worker-2

# Terminal 4: Frontend
cd frontend
npm run dev
```

## Browser/WASM Deployment

### Worker in Browser (WASM)

If you want browser-based workers using **libp2p + IPFS in WebAssembly**:

```rust
// backend/worker/Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "Window",
    "Document",
    "HtmlElement",
] }

# libp2p for browser
libp2p = { version = "0.53", features = [
    "webrtc",
    "websocket",
    "mplex",
    "noise",
    "identify",
    "kad",
] }

# For IPFS in browser
helia = "1.0"  # Rust IPFS library (experimental)
```

### Build for WASM

```bash
# Install wasm-pack
cargo install wasm-pack

# Build worker for browser
cd backend/worker
wasm-pack build --target web --release

# This generates JS bindings in frontend/pkg/
```

### React + WASM Worker

```typescript
// frontend/src/components/BrowserWorker.tsx
import { useEffect, useState } from 'react';
import init, { WorkerNodeServer } from '../pkg';

export function BrowserWorker() {
  const [status, setStatus] = useState('initializing');

  useEffect(() => {
    (async () => {
      await init();
      
      // Start worker in browser
      const worker = new WorkerNodeServer('browser-worker-1');
      await worker.start();
      
      setStatus('connected');
    })();
  }, []);

  return <div>Worker Status: {status}</div>;
}
```

### IPFS in Browser (js-ipfs)

```typescript
// frontend/src/services/ipfs.ts
import { create } from 'ipfs-http-client';

const ipfs = create({
  host: 'localhost',
  port: 5001,
  protocol: 'http',
});

export async function uploadToIPFS(data: Uint8Array): Promise<string> {
  const result = await ipfs.add(data);
  return result.path;  // Returns: QmXxxx
}

export async function downloadFromIPFS(hash: string): Promise<Uint8Array> {
  const chunks = [];
  for await (const chunk of ipfs.cat(hash)) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}
```

## Scaling Considerations

### Native Deployment (Recommended for Scale)

✅ **Pros:**
- Full OS-level networking (UDP, TCP, mDNS)
- True Freenet Scaffold integration
- Unlimited compute capacity
- Can run on GPU/TPU
- Better security isolation

❌ **Cons:**
- Requires dedicated servers
- More infrastructure cost

### Browser Deployment (Limited Scope)

✅ **Pros:**
- Zero infrastructure cost
- Auto-scales with users joining
- Natural browser restrictions limit resource abuse

❌ **Cons:**
- Limited by browser APIs (no UDP, limited disk)
- Sandboxed execution
- Can't run 24/7 (user closes browser)
- WebRTC requires STUN/TURN relay in some networks

## Recommended Deployment for Production

```
Phase 1 (MVP): 
  └─ Coordinator (native, 1 instance)
  └─ Workers (native, 5-10 instances)
  └─ IPFS (Kubo, 1-3 instances)
  └─ Frontend (React, static hosting)

Phase 2 (Growth):
  └─ Coordinators (native, 3-5 instances, load-balanced)
  └─ Workers (native, 100-1000 instances)
  └─ IPFS (Kubo cluster, 10+ instances)
  └─ Browser workers (experimental, WASM)

Phase 3 (Scale):
  └─ Multi-region deployment
  └─ Kubernetes orchestration
  └─ Freenet Scaffold cross-region state sync
  └─ 1M+ browser workers optional
```

## Network Topology

### Production Network (No Single Point of Failure)

```
┌─────────────────────────────────────────────────┐
│          Public Internet (libp2p/IPFS)          │
│                                                 │
│  ┌──────────────┐  ┌──────────────┐             │
│  │  Coord 1     │  │  Coord 2     │             │
│  │  (US East)   │──│  (EU West)   │             │
│  └──────────────┘  └──────────────┘             │
│         │                  │                    │
│    ┌────┴──────────────────┴────┐               │
│    │                            │               │
│    ↓ (libp2p pubsub)            ↓               │
│  ┌─────────────────────────────────────────┐   │
│  │  DHT (Distributed Hash Table)           │   │
│  │  Peer Discovery & Routing               │   │
│  └─────────────────────────────────────────┘   │
│    │                            │               │
│    ↓ (Task assignments)         ↓               │
│  ┌──────────────┐  ┌──────────────┐             │
│  │  Workers     │  │  Workers     │             │
│  │  1000+       │  │  1000+       │             │
│  └──────────────┘  └──────────────┘             │
│                                                 │
│  ┌──────────────┐  ┌──────────────┐             │
│  │  IPFS 1      │  │  IPFS 2      │             │
│  │  (Content)   │  │  (Content)   │             │
│  └──────────────┘  └──────────────┘             │
└─────────────────────────────────────────────────┘
```

## Why Not Freenet for Everything?

Freenet is **NOT suitable for browser WASM** because:
- Freenet requires UDP hole-punching (not available in browsers)
- Freenet has complex protocol overhead (designed for high-latency, low-trust networks)
- Freenet is Java-based (no WASM target)
- libp2p + IPFS already solved this problem with WebRTC transport

## Recommendation

**For Freenet Lottery 1.0:**
1. Deploy coordinator and workers as **native Rust applications**
2. Use **libp2p** for P2P networking (proven, WebRTC-capable)
3. Use **IPFS** for content storage (production-ready)
4. Use **Freenet Scaffold concepts** for state sync (implement as custom Rust, not Java Freenet)
5. Frontend stays as **React TypeScript** (no WASM needed for UI)

**For Freenet Lottery 2.0 (if needed):**
- Optionally add browser workers using **libp2p WebRTC + WASM**
- This would enable crowdsourced rendering from web users
- Still use native coordinators for lottery management
