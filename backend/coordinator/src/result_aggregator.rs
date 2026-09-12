// Result aggregator: combines segment results into final output

use common::TaskId;

/// Aggregates segment results into final rendered output
pub struct ResultAggregator {
    aggregation_cache: std::collections::HashMap<TaskId, Vec<u8>>,
}

impl ResultAggregator {
    pub fn new() -> Self {
        Self {
            aggregation_cache: std::collections::HashMap::new(),
        }
    }

    pub async fn aggregate(&mut self, task_id: TaskId) -> Result<String, String> {
        // In production: fetch all segment results from IPFS
        // Combine them in order
        // Return final IPFS hash

        println!("[ResultAggregator] Aggregating results for task {}", task_id.0);

        // Simulate aggregation
        let final_data = vec![42u8; 10_000_000]; // 10MB mock result
        self.aggregation_cache.insert(task_id, final_data.clone());

        // Generate IPFS hash
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&final_data);
        let hash = format!("Qm{:x}", hasher.finalize());

        Ok(hash)
    }
}
