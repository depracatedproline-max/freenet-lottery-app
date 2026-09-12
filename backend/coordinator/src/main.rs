// Coordinator node: Task management, aggregation, and lottery
// Coordinates distributed rendering tasks and manages Freenet Scaffold state

use common::{
    TaskId, RenderingTask, Segment, SegmentId, SegmentStatus, WorkerId, WorkerNode,
    SegmentResult, LotteryState, LotteryResult, LotteryEntry, EntryType, ParticipantRecord,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sha2::{Sha256, Digest};

mod task_manager;
mod worker_registry;
mod result_aggregator;
mod lottery_engine;
mod freenet_scaffold;

use task_manager::TaskManager;
use worker_registry::WorkerRegistry;
use result_aggregator::ResultAggregator;
use lottery_engine::LotteryEngine;
use freenet_scaffold::FreenetScaffoldState;

/// Main coordinator node
pub struct CoordinatorNode {
    node_id: String,
    task_manager: Arc<RwLock<TaskManager>>,
    worker_registry: Arc<RwLock<WorkerRegistry>>,
    result_aggregator: Arc<RwLock<ResultAggregator>>,
    lottery_engine: Arc<LotteryEngine>,
    freenet_state: Arc<RwLock<FreenetScaffoldState>>,
    libp2p_handle: Arc<LibP2PCoordinatorHandle>,
}

impl CoordinatorNode {
    /// Create a new coordinator node
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            task_manager: Arc::new(RwLock::new(TaskManager::new())),
            worker_registry: Arc::new(RwLock::new(WorkerRegistry::new())),
            result_aggregator: Arc::new(RwLock::new(ResultAggregator::new())),
            lottery_engine: Arc::new(LotteryEngine::new()),
            freenet_state: Arc::new(RwLock::new(FreenetScaffoldState::new())),
            libp2p_handle: Arc::new(LibP2PCoordinatorHandle::new()),
        }
    }

    /// Start the coordinator node
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("═══════════════════════════════════════════════════════");
        println!("  Freenet Lottery Coordinator Node");
        println!("═══════════════════════════════════════════════════════");
        println!("[Coordinator {}] Starting...", self.node_id);

        // Initialize libp2p network
        self.libp2p_handle.init_network(&self.node_id).await?;
        println!("[Coordinator {}] Connected to libp2p network", self.node_id);

        // Start background tasks
        self.start_worker_health_monitor().await;
        self.start_task_processor().await;
        self.start_result_processor().await;
        self.start_lottery_scheduler().await;
        self.start_freenet_sync().await;

        println!("[Coordinator {}] All systems online", self.node_id);
        Ok(())
    }

    /// Submit a new rendering task
    pub async fn submit_task(&self, task: RenderingTask) -> Result<TaskId, String> {
        println!(
            "[Coordinator {}] Received task: {} (size: {} bytes, {} segments)",
            self.node_id, task.id.0, task.content_size, task.segment_count
        );

        // Store task in manager
        {
            let mut manager = self.task_manager.write().await;
            manager.add_task(task.clone()).await;
        }

        // Decompose into segments
        let segments = self.decompose_task(&task).await?;
        println!(
            "[Coordinator {}] Decomposed task {} into {} segments",
            self.node_id,
            task.id.0,
            segments.len()
        );

        // Store segments
        {
            let mut manager = self.task_manager.write().await;
            for segment in &segments {
                manager.add_segment(segment.clone()).await;
            }
        }

        // Broadcast to workers via libp2p
        self.broadcast_segments_to_workers(&segments).await?;

        // Record in Freenet Scaffold state
        {
            let mut state = self.freenet_state.write().await;
            state.add_task(task.clone());
        }

        Ok(task.id)
    }

    /// Decompose a task into N segments
    async fn decompose_task(&self, task: &RenderingTask) -> Result<Vec<Segment>, String> {
        let mut segments = Vec::new();
        let segment_size = (task.content_size + task.segment_count as u64 - 1) / task.segment_count as u64;

        for i in 0..task.segment_count {
            let start_byte = i as u64 * segment_size;
            let end_byte = std::cmp::min((i + 1) as u64 * segment_size, task.content_size);
            let seg_size = end_byte - start_byte;

            // Hash the segment data range for IPFS reference
            let mut hasher = Sha256::new();
            hasher.update(task.content_hash.as_bytes());
            hasher.update(i.to_le_bytes());
            let segment_hash = format!("{:x}", hasher.finalize());
            let segment_ipfs_hash = format!("Qm{}/{}", task.content_hash, segment_hash);

            let segment = Segment {
                id: SegmentId {
                    task_id: task.id,
                    segment_index: i,
                },
                task_id: task.id,
                segment_index: i,
                data_hash: segment_ipfs_hash,
                data_size: seg_size,
                parameters: task.parameters.clone(),
                assigned_to: None,
                status: SegmentStatus::Pending,
                proof_of_work: None,
            };

            segments.push(segment);
        }

        Ok(segments)
    }

    /// Broadcast segments to workers via libp2p pubsub
    async fn broadcast_segments_to_workers(&self, segments: &[Segment]) -> Result<(), String> {
        println!(
            "[Coordinator {}] Broadcasting {} segments to worker network",
            self.node_id,
            segments.len()
        );

        // Send each segment to the task assignment channel
        for segment in segments {
            self.libp2p_handle
                .broadcast_task_assignment(segment, &self.node_id)
                .await?;
        }

        Ok(())
    }

    /// Process segment result from a worker
    pub async fn process_result(&self, result: SegmentResult) -> Result<(), String> {
        println!(
            "[Coordinator {}] Received result for segment {} from worker {}",
            self.node_id, result.segment_id.segment_index, result.worker_id.0
        );

        // Validate proof of work
        if !self.validate_proof_of_work(&result.proof_of_work) {
            return Err("Invalid proof of work".to_string());
        }

        // Update segment status
        {
            let mut manager = self.task_manager.write().await;
            manager.mark_segment_completed(&result.segment_id, &result.worker_id).await;
        }

        // Update worker reputation
        {
            let mut registry = self.worker_registry.write().await;
            registry.record_successful_completion(&result.worker_id, result.processing_time_ms).await;
        }

        // Check if task is complete
        {
            let manager = self.task_manager.read().await;
            if manager.is_task_complete(result.segment_id.task_id).await {
                drop(manager); // Release lock before calling aggregate

                println!(
                    "[Coordinator {}] Task {} is complete! Aggregating results...",
                    self.node_id, result.segment_id.task_id.0
                );

                self.aggregate_task_results(result.segment_id.task_id).await?;
            }
        }

        Ok(())
    }

    /// Aggregate all segment results into final output
    async fn aggregate_task_results(&self, task_id: TaskId) -> Result<String, String> {
        println!(
            "[Coordinator {}] Aggregating results for task {}",
            self.node_id, task_id.0
        );

        let mut aggregator = self.result_aggregator.write().await;
        let final_hash = aggregator.aggregate(task_id).await?;

        println!(
            "[Coordinator {}] Task {} aggregated! Final IPFS hash: {}",
            self.node_id, task_id.0, final_hash
        );

        // Draw lottery
        self.draw_lottery_for_task(task_id, final_hash).await?;

        Ok(final_hash)
    }

    /// Draw lottery winner for completed task
    async fn draw_lottery_for_task(&self, task_id: TaskId, final_hash: String) -> Result<(), String> {
        println!(
            "[Coordinator {}] Drawing lottery for task {}",
            self.node_id, task_id.0
        );

        // Get all entries for this task
        let entries = {
            let manager = self.task_manager.read().await;
            manager.get_lottery_entries(task_id).await
        };

        if entries.is_empty() {
            return Err("No lottery entries for task".to_string());
        }

        // Draw winner using VRF
        let winner = self.lottery_engine.draw_winner(&entries)?;

        println!(
            "[Coordinator {}] Lottery winner: {} (from {} entries)",
            self.node_id,
            winner.worker_id.0,
            entries.len()
        );

        // Get task for reward pool
        let task = {
            let manager = self.task_manager.read().await;
            manager.get_task(task_id).await.ok_or("Task not found")?
        };

        // Create lottery result
        let result = LotteryResult {
            task_id,
            epoch: 0,
            winner_id: winner.worker_id.clone(),
            winning_entry: winner.clone(),
            total_entries: entries.len() as u64,
            reward_amount: task.reward_pool,
            drawn_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            vrf_proof: "vrf-proof-placeholder".to_string(),
        };

        // Broadcast lottery result via Freenet Scaffold
        {
            let mut state = self.freenet_state.write().await;
            state.add_lottery_result(result.clone());

            // Also update participant records
            state.update_participant_credits(&winner.worker_id, task.reward_pool);
        }

        // Broadcast to all nodes for consensus
        self.libp2p_handle
            .broadcast_lottery_result(&result)
            .await?;

        Ok(())
    }

    /// Validate proof of work
    fn validate_proof_of_work(&self, proof: &common::ProofOfWork) -> bool {
        // Check that hash has required leading zeros
        let leading_zeros = proof.hash.chars().take_while(|c| *c == '0').count();
        leading_zeros >= proof.difficulty as usize
    }

    /// Monitor worker health and availability
    async fn start_worker_health_monitor(&self) {
        let registry = self.worker_registry.clone();
        let libp2p = self.libp2p_handle.clone();
        let node_id = self.node_id.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

                let mut reg = registry.write().await;
                let stale_workers = reg.get_stale_workers(120).await; // 2 minute timeout

                for worker_id in stale_workers {
                    println!("[Coordinator {}] Worker {} is stale, removing", node_id, worker_id.0);
                    reg.remove_worker(&worker_id).await;
                }

                // Print current worker count
                let count = reg.worker_count().await;
                println!("[Coordinator {}] Active workers: {}", node_id, count);
            }
        });
    }

    /// Process incoming segments and assign to workers
    async fn start_task_processor(&self) {
        let task_mgr = self.task_manager.clone();
        let worker_reg = self.worker_registry.clone();
        let node_id = self.node_id.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                let mut mgr = task_mgr.write().await;
                let pending_segments = mgr.get_pending_segments(100).await; // Get up to 100 pending

                let registry = worker_reg.read().await;
                for segment in pending_segments {
                    if let Some(worker) = registry.best_available_worker().await {
                        mgr.assign_segment(&segment.id, &worker.id).await;
                        println!(
                            "[Coordinator {}] Assigned segment {} to worker {}",
                            node_id, segment.segment_index, worker.id.0
                        );
                    }
                }
            }
        });
    }

    /// Process incoming results from workers
    async fn start_result_processor(&self) {
        let node_id = self.node_id.clone();
        // In production, this would subscribe to result channel from libp2p
        // For now, just a placeholder
        println!("[Coordinator {}] Result processor started", node_id);
    }

    /// Schedule periodic lottery draws
    async fn start_lottery_scheduler(&self) {
        let node_id = self.node_id.clone();
        println!("[Coordinator {}] Lottery scheduler started", node_id);
    }

    /// Synchronize state with other coordinators via Freenet Scaffold
    async fn start_freenet_sync(&self) {
        let state = self.freenet_state.clone();
        let libp2p = self.libp2p_handle.clone();
        let node_id = self.node_id.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                // Fetch state from peers
                if let Ok(peer_states) = libp2p.fetch_peer_states().await {
                    let mut local_state = state.write().await;
                    for peer_state in peer_states {
                        local_state.merge(peer_state);
                    }
                    println!("[Coordinator {}] State synchronized with peers", node_id);
                }
            }
        });
    }

    /// Get coordinator status
    pub async fn get_status(&self) -> CoordinatorStatus {
        let task_mgr = self.task_manager.read().await;
        let worker_reg = self.worker_registry.read().await;
        let state = self.freenet_state.read().await;

        CoordinatorStatus {
            node_id: self.node_id.clone(),
            active_tasks: task_mgr.task_count().await,
            pending_segments: task_mgr.pending_segment_count().await,
            completed_segments: task_mgr.completed_segment_count().await,
            active_workers: worker_reg.worker_count().await,
            lottery_results: state.lottery_results_count(),
            state_epoch: state.epoch(),
        }
    }
}

/// libp2p coordinator handle
#[derive(Clone)]
pub struct LibP2PCoordinatorHandle;

impl LibP2PCoordinatorHandle {
    pub fn new() -> Self {
        Self
    }

    async fn init_network(&self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("[libp2p] Initializing coordinator network interface");
        // In production: set up libp2p swarm for coordinator role
        Ok(())
    }

    async fn broadcast_task_assignment(
        &self,
        segment: &Segment,
        coordinator_id: &str,
    ) -> Result<(), String> {
        println!(
            "[libp2p] Broadcasting segment {} from coordinator {}",
            segment.segment_index, coordinator_id
        );
        // In production: publish to "freenet-lottery-tasks" pubsub channel
        Ok(())
    }

    async fn broadcast_lottery_result(&self, result: &LotteryResult) -> Result<(), String> {
        println!(
            "[libp2p] Broadcasting lottery result - Winner: {}",
            result.winner_id.0
        );
        // In production: publish to "freenet-lottery-results" pubsub channel
        Ok(())
    }

    async fn fetch_peer_states(&self) -> Result<Vec<FreenetScaffoldState>, String> {
        // In production: query peers for their state snapshots
        Ok(vec![])
    }
}

/// Coordinator status snapshot
#[derive(Debug, Clone)]
pub struct CoordinatorStatus {
    pub node_id: String,
    pub active_tasks: u64,
    pub pending_segments: u64,
    pub completed_segments: u64,
    pub active_workers: u64,
    pub lottery_results: u64,
    pub state_epoch: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node_id = std::env::args()
        .find(|arg| arg.starts_with("--id="))
        .and_then(|arg| arg.strip_prefix("--id=").map(String::from))
        .unwrap_or_else(|| format!("coordinator-{}", uuid::Uuid::new_v4()));

    let coordinator = CoordinatorNode::new(node_id);
    coordinator.start().await?;

    // Keep server running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        let status = coordinator.get_status().await;
        println!("[Coordinator Status] {:?}", status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coord = CoordinatorNode::new("test-coord".to_string());
        assert_eq!(coord.node_id, "test-coord");
    }

    #[tokio::test]
    async fn test_decompose_task() {
        let coord = CoordinatorNode::new("test".to_string());
        let task = RenderingTask {
            id: TaskId(1),
            content_hash: "QmTest".to_string(),
            content_size: 1000,
            segment_count: 10,
            rendering_type: common::RenderingType::ImageProcessing,
            parameters: HashMap::new(),
            created_at: 0,
            deadline: 0,
            reward_pool: 100,
            priority: 5,
        };

        let segments = coord.decompose_task(&task).await.unwrap();
        assert_eq!(segments.len(), 10);
        assert_eq!(segments[0].data_size, 100);
    }
}
