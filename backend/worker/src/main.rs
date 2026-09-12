// Worker node main entry point
// Connects to Freenet/libp2p network, receives tasks, processes segments

use common::{WorkerId, WorkerNode, ComputeCapacity, Segment, SegmentStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

mod segment_processor;
use segment_processor::{SegmentProcessor, WorkerMetrics};

/// Main worker node
pub struct WorkerNodeServer {
    worker_id: WorkerId,
    processor: SegmentProcessor,
    metrics: Arc<RwLock<WorkerMetrics>>,
    peer_id: String,
    current_tasks: Arc<RwLock<HashMap<u64, Segment>>>,
}

impl WorkerNodeServer {
    /// Create a new worker node
    pub fn new(worker_id: WorkerId, compute_capacity: ComputeCapacity) -> Self {
        let processor = SegmentProcessor::new(worker_id.clone(), compute_capacity);
        let peer_id = format!("worker-{}", worker_id.0);

        Self {
            worker_id,
            processor,
            metrics: Arc::new(RwLock::new(WorkerMetrics::new())),
            peer_id,
            current_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the worker node server
    /// Connects to libp2p network and listens for tasks
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[Worker {}] Starting worker node...", self.worker_id.0);
        println!("[Worker {}] Peer ID: {}", self.worker_id.0, self.peer_id);

        // Initialize libp2p connection
        let libp2p_handle = self.init_libp2p().await?;
        println!("[Worker {}] Connected to libp2p network", self.worker_id.0);

        // Start heartbeat to coordinator
        self.start_heartbeat(libp2p_handle.clone()).await;

        // Start task listener
        self.listen_for_tasks(libp2p_handle).await;

        Ok(())
    }

    /// Initialize libp2p network connection
    async fn init_libp2p(&self) -> Result<LibP2PHandle, Box<dyn std::error::Error>> {
        println!("[Worker {}] Initializing libp2p...", self.worker_id.0);

        // In production, this would:
        // 1. Create a libp2p swarm
        // 2. Configure transport (TCP, QUIC)
        // 3. Set up Identify protocol (peer info)
        // 4. Subscribe to task pubsub channel
        // 5. Set up DHT for peer discovery
        
        let handle = LibP2PHandle {
            peer_id: self.peer_id.clone(),
            pubsub_channel: "freenet-lottery-tasks".to_string(),
            bootstrap_peers: vec![
                "/ip4/127.0.0.1/tcp/30333/p2p/QmBootstrap1".to_string(),
                "/ip4/127.0.0.1/tcp/30334/p2p/QmBootstrap2".to_string(),
            ],
        };

        println!(
            "[Worker {}] libp2p initialized. Subscribing to: {}",
            self.worker_id.0, handle.pubsub_channel
        );

        Ok(handle)
    }

    /// Send periodic heartbeat to coordinator and network
    async fn start_heartbeat(&self, libp2p: LibP2PHandle) {
        let worker_id = self.worker_id.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                let metrics_snapshot = metrics.read().await;
                println!(
                    "[Worker {}] Heartbeat - Processed: {}, Success Rate: {:.2}%, Avg Time: {:.0}ms",
                    worker_id.0,
                    metrics_snapshot.segments_processed,
                    metrics_snapshot.success_rate() * 100.0,
                    metrics_snapshot.avg_processing_time_ms()
                );

                // Broadcast heartbeat via libp2p
                // This updates the worker registry on all nodes
                let heartbeat = WorkerHeartbeat {
                    worker_id: worker_id.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    segments_completed: metrics_snapshot.segments_processed,
                    success_rate: metrics_snapshot.success_rate(),
                    avg_processing_time_ms: metrics_snapshot.avg_processing_time_ms() as u64,
                };

                libp2p.broadcast_heartbeat(&heartbeat).await;
            }
        });
    }

    /// Listen for task assignments and process them
    async fn listen_for_tasks(&self, libp2p: LibP2PHandle) {
        let worker_id = self.worker_id.clone();
        let processor = segment_processor::SegmentProcessor::new(
            worker_id.clone(),
            ComputeCapacity {
                cpu_cores: 8,
                gpu_vram_mb: 2048,
                available_disk_mb: 100_000,
                network_speed_mbps: 100,
            },
        );
        let metrics = self.metrics.clone();
        let current_tasks = self.current_tasks.clone();

        tokio::spawn(async move {
            // Create channel receiver for pubsub messages
            let mut rx = libp2p.subscribe_to_tasks().await;

            while let Some(task_msg) = rx.recv().await {
                println!(
                    "[Worker {}] Received task: {:?}",
                    worker_id.0, task_msg.segment.id
                );

                // Store task
                {
                    let mut tasks = current_tasks.write().await;
                    tasks.insert(task_msg.segment.id.task_id.0, task_msg.segment.clone());
                }

                // Process asynchronously
                let worker_id = worker_id.clone();
                let processor = segment_processor::SegmentProcessor::new(
                    worker_id.clone(),
                    ComputeCapacity {
                        cpu_cores: 8,
                        gpu_vram_mb: 2048,
                        available_disk_mb: 100_000,
                        network_speed_mbps: 100,
                    },
                );
                let metrics = metrics.clone();
                let segment = task_msg.segment.clone();
                let libp2p = libp2p.clone();
                let coordinator_peer = task_msg.coordinator_peer.clone();

                tokio::spawn(async move {
                    match processor.process_segment(&segment).await {
                        Ok(result) => {
                            println!(
                                "[Worker {}] ✓ Completed segment {} in {}ms",
                                worker_id.0, segment.id.segment_index, result.processing_time_ms
                            );

                            // Record success
                            {
                                let mut m = metrics.write().await;
                                m.record_success(
                                    result.processing_time_ms,
                                    result.proof_of_work.difficulty,
                                );
                            }

                            // Send result back to coordinator via direct connection
                            libp2p
                                .send_result_to_coordinator(result, &coordinator_peer)
                                .await;
                        }
                        Err(e) => {
                            println!(
                                "[Worker {}] ✗ Failed to process segment {}: {}",
                                worker_id.0, segment.id.segment_index, e
                            );

                            let mut m = metrics.write().await;
                            m.record_failure();
                        }
                    }
                });
            }
        });
    }

    /// Get current worker status
    pub async fn get_status(&self) -> WorkerStatus {
        let metrics = self.metrics.read().await;
        let tasks = self.current_tasks.read().await;

        WorkerStatus {
            worker_id: self.worker_id.clone(),
            peer_id: self.peer_id.clone(),
            segments_processed: metrics.segments_processed,
            success_rate: metrics.success_rate(),
            avg_processing_time_ms: metrics.avg_processing_time_ms(),
            current_tasks_count: tasks.len() as u32,
            last_heartbeat: metrics.last_heartbeat,
        }
    }
}

/// libp2p network handle
#[derive(Clone)]
pub struct LibP2PHandle {
    peer_id: String,
    pubsub_channel: String,
    bootstrap_peers: Vec<String>,
}

impl LibP2PHandle {
    /// Subscribe to task assignment channel
    async fn subscribe_to_tasks(&self) -> tokio::sync::mpsc::Receiver<TaskAssignment> {
        // In production, this creates a real pubsub subscription
        // For now, return a mock channel
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        rx
    }

    /// Broadcast heartbeat to all nodes
    async fn broadcast_heartbeat(&self, heartbeat: &WorkerHeartbeat) {
        println!(
            "[libp2p] Broadcasting heartbeat from {} to all nodes",
            heartbeat.worker_id.0
        );
        // In production: publish to heartbeat pubsub channel
        // All nodes subscribe and update their worker registry
    }

    /// Send processing result back to coordinator
    async fn send_result_to_coordinator(&self, result: common::SegmentResult, coordinator: &str) {
        println!(
            "[libp2p] Sending result to coordinator: {}",
            coordinator
        );
        // In production: direct libp2p connection to coordinator
        // Send result via MPLEX or YAMUX stream
    }
}

/// Task assignment received from coordinator
#[derive(Debug, Clone)]
pub struct TaskAssignment {
    pub segment: Segment,
    pub coordinator_peer: String,
    pub deadline_seconds: u64,
}

/// Worker heartbeat message
#[derive(Debug, Clone)]
pub struct WorkerHeartbeat {
    pub worker_id: WorkerId,
    pub timestamp: u64,
    pub segments_completed: u64,
    pub success_rate: f64,
    pub avg_processing_time_ms: u64,
}

/// Worker status snapshot
#[derive(Debug, Clone)]
pub struct WorkerStatus {
    pub worker_id: WorkerId,
    pub peer_id: String,
    pub segments_processed: u64,
    pub success_rate: f64,
    pub avg_processing_time_ms: f64,
    pub current_tasks_count: u32,
    pub last_heartbeat: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let worker_id = std::env::args()
        .find(|arg| arg.starts_with("--id="))
        .and_then(|arg| arg.strip_prefix("--id=").map(String::from))
        .unwrap_or_else(|| format!("worker-{}", uuid::Uuid::new_v4()));

    let capacity = ComputeCapacity {
        cpu_cores: 8,
        gpu_vram_mb: 2048,
        available_disk_mb: 100_000,
        network_speed_mbps: 100,
    };

    // Create and start worker node
    let worker = WorkerNodeServer::new(WorkerId(worker_id), capacity);

    println!("═══════════════════════════════════════════════════════");
    println!("  Freenet Lottery Worker Node");
    println!("═══════════════════════════════════════════════════════");

    worker.start().await?;

    // Keep server running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        let status = worker.get_status().await;
        println!("[Worker Status] {:?}", status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_creation() {
        let capacity = ComputeCapacity {
            cpu_cores: 8,
            gpu_vram_mb: 2048,
            available_disk_mb: 100_000,
            network_speed_mbps: 100,
        };
        let worker = WorkerNodeServer::new(WorkerId("test-worker".into()), capacity);
        assert_eq!(worker.worker_id.0, "test-worker");
        assert!(worker.peer_id.contains("test-worker"));
    }

    #[tokio::test]
    async fn test_worker_status() {
        let capacity = ComputeCapacity {
            cpu_cores: 8,
            gpu_vram_mb: 2048,
            available_disk_mb: 100_000,
            network_speed_mbps: 100,
        };
        let worker = WorkerNodeServer::new(WorkerId("test".into()), capacity);
        let status = worker.get_status().await;
        assert_eq!(status.segments_processed, 0);
        assert_eq!(status.success_rate, 0.0);
    }
}
