// Worker registry: tracks available workers and their health

use common::{WorkerId, WorkerNode, ComputeCapacity};
use std::collections::HashMap;

/// Registry of active worker nodes
pub struct WorkerRegistry {
    workers: HashMap<WorkerId, WorkerNode>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    pub async fn register_worker(&mut self, worker: WorkerNode) {
        self.workers.insert(worker.id.clone(), worker);
    }

    pub async fn remove_worker(&mut self, worker_id: &WorkerId) {
        self.workers.remove(worker_id);
    }

    pub async fn get_stale_workers(&self, timeout_secs: u64) -> Vec<WorkerId> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.workers
            .values()
            .filter(|w| (now - w.last_heartbeat) > timeout_secs)
            .map(|w| w.id.clone())
            .collect()
    }

    pub async fn best_available_worker(&self) -> Option<WorkerNode> {
        // Select worker with highest reputation and lowest current load
        self.workers
            .values()
            .filter(|w| w.current_segment.is_none())
            .max_by(|a, b| a.reputation_score.partial_cmp(&b.reputation_score).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
    }

    pub async fn record_successful_completion(&mut self, worker_id: &WorkerId, processing_time_ms: u64) {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            worker.reputation_score = (worker.reputation_score * 0.95) + 2.5;
            worker.completion_rate = (worker.completion_rate * 0.9) + 10.0;
            worker.total_segments_completed += 1;
        }
    }

    pub async fn worker_count(&self) -> u64 {
        self.workers.len() as u64
    }
}
