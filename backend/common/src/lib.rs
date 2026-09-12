// Shared types for Freenet Lottery system
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a rendering task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub u64);

/// Unique identifier for a worker node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(pub String);

/// Unique identifier for a rendered segment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SegmentId {
    pub task_id: TaskId,
    pub segment_index: u32,
}

/// A rendering task submitted by a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingTask {
    pub id: TaskId,
    pub content_hash: String,           // IPFS hash of original content
    pub content_size: u64,              // bytes
    pub segment_count: u32,             // how many segments to split into
    pub rendering_type: RenderingType,
    pub parameters: HashMap<String, String>,
    pub created_at: u64,                // unix timestamp
    pub deadline: u64,                  // unix timestamp
    pub reward_pool: u64,               // credits for lottery
    pub priority: u8,                   // 0-255, higher = more priority
}

/// Types of rendering operations supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderingType {
    ImageProcessing,
    VideoFrame,
    ThreeDRender,
    MLInference,
    AudioProcessing,
    DataTransformation,
    Custom(String),
}

/// A single segment of work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: SegmentId,
    pub task_id: TaskId,
    pub segment_index: u32,
    pub data_hash: String,              // IPFS hash of segment data
    pub data_size: u64,
    pub parameters: HashMap<String, String>,
    pub assigned_to: Option<WorkerId>,
    pub status: SegmentStatus,
    pub proof_of_work: Option<ProofOfWork>,
}

/// Status of a segment throughout its lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentStatus {
    Pending,        // Waiting for worker
    Assigned,       // Given to a worker
    Processing,     // Worker is working on it
    Completed,      // Worker returned results
    Verified,       // Proof-of-work validated
    Failed,         // Worker failed or timed out
}

/// Result of segment processing from a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentResult {
    pub segment_id: SegmentId,
    pub worker_id: WorkerId,
    pub result_hash: String,            // IPFS hash of result data
    pub result_size: u64,
    pub processing_time_ms: u64,
    pub proof_of_work: ProofOfWork,
    pub timestamp: u64,
}

/// Proof that work was actually done
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWork {
    pub nonce: u64,                     // Random value used in computation
    pub difficulty: u32,                // How many leading zeros required
    pub hash: String,                   // SHA-256 hash meeting difficulty
    pub computation_hash: String,       // Hash of actual computation result
    pub work_factor: f64,               // Estimated CPU hours consumed
}

/// Worker node registration and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNode {
    pub id: WorkerId,
    pub peer_id: String,                // libp2p peer ID
    pub compute_capacity: ComputeCapacity,
    pub reputation_score: f64,          // 0.0 - 100.0
    pub completion_rate: f64,           // % of assigned tasks completed
    pub avg_processing_speed: f64,      // MB/s
    pub joined_at: u64,
    pub last_heartbeat: u64,
    pub current_segment: Option<SegmentId>,
    pub total_segments_completed: u64,
}

/// Worker's compute resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeCapacity {
    pub cpu_cores: u32,
    pub gpu_vram_mb: u32,
    pub available_disk_mb: u64,
    pub network_speed_mbps: u32,
}

/// Lottery entry for a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryEntry {
    pub id: u64,
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub segment_id: SegmentId,
    pub entry_type: EntryType,
    pub created_at: u64,
}

/// Different types of lottery entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryType {
    SegmentCompletion,   // Completed a rendering segment
    QualityBonus,        // Exceptionally fast completion
    ConsistencyBonus,    // Nth consecutive completed segment
}

/// Lottery drawing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryResult {
    pub task_id: TaskId,
    pub epoch: u64,
    pub winner_id: WorkerId,
    pub winning_entry: LotteryEntry,
    pub total_entries: u64,
    pub reward_amount: u64,
    pub drawn_at: u64,
    pub vrf_proof: String,              // VRF randomness proof
}

/// Participant's account and reputation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantRecord {
    pub worker_id: WorkerId,
    pub credits_balance: u64,
    pub reputation_score: f64,
    pub segments_completed: u64,
    pub lottery_wins: u64,
    pub tasks_submitted: u64,
    pub joined_at: u64,
    pub badges: Vec<Badge>,
}

/// Achievement badges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub name: String,
    pub description: String,
    pub earned_at: u64,
}

/// Freenet Scaffold mergeable state for distributed consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryState {
    pub epoch: u64,
    pub tasks: HashMap<TaskId, RenderingTask>,
    pub segments: HashMap<SegmentId, Segment>,
    pub workers: HashMap<WorkerId, WorkerNode>,
    pub lottery_results: Vec<LotteryResult>,
    pub participant_records: HashMap<WorkerId, ParticipantRecord>,
    pub last_updated: u64,
}

impl LotteryState {
    pub fn new() -> Self {
        Self {
            epoch: 0,
            tasks: HashMap::new(),
            segments: HashMap::new(),
            workers: HashMap::new(),
            lottery_results: Vec::new(),
            participant_records: HashMap::new(),
            last_updated: 0,
        }
    }

    /// Merge two states from different nodes (Byzantine-fault-tolerant)
    pub fn merge(&mut self, other: Self) {
        // Tasks: keep the most recent version
        for (task_id, task) in other.tasks {
            self.tasks
                .entry(task_id)
                .and_modify(|existing| {
                    if task.created_at > existing.created_at {
                        *existing = task.clone();
                    }
                })
                .or_insert(task);
        }

        // Segments: merge completion states
        for (seg_id, segment) in other.segments {
            self.segments
                .entry(seg_id)
                .and_modify(|existing| {
                    if segment.status as u8 > existing.status as u8 {
                        *existing = segment.clone();
                    }
                })
                .or_insert(segment);
        }

        // Workers: update if newer heartbeat
        for (worker_id, worker) in other.workers {
            self.workers
                .entry(worker_id)
                .and_modify(|existing| {
                    if worker.last_heartbeat > existing.last_heartbeat {
                        *existing = worker.clone();
                    }
                })
                .or_insert(worker);
        }

        // Lottery results: append new results (immutable)
        self.lottery_results.extend(other.lottery_results);

        // Participant records: merge accumulated stats
        for (worker_id, record) in other.participant_records {
            self.participant_records
                .entry(worker_id.clone())
                .and_modify(|existing| {
                    existing.credits_balance = existing.credits_balance.max(record.credits_balance);
                    existing.segments_completed += record.segments_completed;
                    existing.lottery_wins += record.lottery_wins;
                    existing.tasks_submitted += record.tasks_submitted;
                })
                .or_insert(record);
        }

        self.epoch = self.epoch.max(other.epoch) + 1;
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lottery_state_merge() {
        let mut state1 = LotteryState::new();
        let mut state2 = LotteryState::new();

        state1.epoch = 1;
        state2.epoch = 2;

        state1.merge(state2);
        assert_eq!(state1.epoch, 3);
    }
}
