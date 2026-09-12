# Development Guide

## Prerequisites

- Rust 1.70+
- Node.js 18+
- Cargo
- Docker (optional)

## Project Structure

```
freenet-lottery-app/
├── backend/
│   ├── Cargo.toml                 # Rust workspace config
│   ├── coordinator/               # Coordinator node
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── task_manager.rs
│   │   │   ├── worker_registry.rs
│   │   │   ├── result_aggregator.rs
│   │   │   └── lottery_engine.rs
│   │   └── Cargo.toml
│   ├── worker/                    # Worker node
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── segment_processor.rs
│   │   │   ├── proof_of_work.rs
│   │   │   └── freenet_client.rs
│   │   └── Cargo.toml
│   └── common/                    # Shared types
│       ├── src/
│       │   └── lib.rs
│       └── Cargo.toml
├── frontend/
│   ├── package.json
│   ├── tsconfig.json
│   ├── src/
│   │   ├── pages/
│   │   │   ├── submit_task.tsx
│   │   │   ├── worker_dashboard.tsx
│   │   │   └── lottery_results.tsx
│   │   ├── components/
│   │   │   ├── TaskForm.tsx
│   │   │   ├── WorkerStats.tsx
│   │   │   └── LotteryDrawing.tsx
│   │   └── App.tsx
│   └── vite.config.ts
├── ARCHITECTURE.md
├── DEVELOPMENT.md
└── README.md
```

## Backend Setup

### 1. Initialize Rust Project

```bash
cd backend
cargo init --name coordinator
cargo init --name worker
cargo init --name common
```

### 2. Add Dependencies

**common/Cargo.toml:**
```toml
[package]
name = "common"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
sha2 = "0.10"
uuid = { version = "1.0", features = ["v4"] }
```

**coordinator/Cargo.toml:**
```toml
[package]
name = "coordinator"
version = "0.1.0"
edition = "2021"

[dependencies]
common = { path = "../common" }
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
tower = "0.4"
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
rand = "0.8"
```

**worker/Cargo.toml:**
```toml
[package]
name = "worker"
version = "0.1.0"
edition = "2021"

[dependencies]
common = { path = "../common" }
tokio = { version = "1.0", features = ["full"] }
sha2 = "0.10"
rand = "0.8"
```

### 3. Build Backend

```bash
# Build coordinator
cargo build --release -p coordinator

# Build worker
cargo build --release -p worker
```

## Frontend Setup

### 1. Initialize React Project

```bash
cd frontend
npm create vite@latest . -- --template react-ts
npm install
```

### 2. Install Dependencies

```bash
npm install axios react-router-dom zustand
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

### 3. Development Server

```bash
npm run dev
```

## Running the Application

### Terminal 1: Coordinator Node

```bash
cd backend/coordinator
cargo run --release -- --port 8080
```

### Terminal 2: Worker Node

```bash
cd backend/worker
cargo run --release -- --coordinator-url http://localhost:8080
```

### Terminal 3: Frontend

```bash
cd frontend
npm run dev
```

The app should be available at `http://localhost:5173`

## Testing

### Unit Tests

```bash
cd backend
cargo test --all
```

### Integration Tests

```bash
cargo test --test integration_tests -- --nocapture
```

## Docker Deployment

### Build Docker Images

```bash
# Coordinator
docker build -f docker/coordinator.Dockerfile -t freenet-lottery-coordinator .

# Worker
docker build -f docker/worker.Dockerfile -t freenet-lottery-worker .

# Frontend
docker build -f docker/frontend.Dockerfile -t freenet-lottery-frontend .
```

### Run with Docker Compose

```bash
docker-compose up -d
```

## Debugging

### Enable Logging

```bash
RUST_LOG=debug cargo run
```

### VSCode Debug Configuration

**.vscode/launch.json:**
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Coordinator",
      "cargo": {
        "args": ["build", "-p", "coordinator"],
        "filter": {"name": "coordinator"}
      },
      "args": ["--port", "8080"]
    }
  ]
}
```

## Next Steps

1. Implement `common/src/lib.rs` - Shared data types
2. Implement `coordinator/src/task_manager.rs` - Task distribution
3. Implement `worker/src/segment_processor.rs` - Rendering engine
4. Create frontend pages for task submission
5. Add Freenet Scaffold integration for state sync
6. Implement lottery drawing logic
7. Deploy to Freenet network
