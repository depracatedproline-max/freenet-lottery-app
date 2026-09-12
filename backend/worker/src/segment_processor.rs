// Worker node: Processes rendering segments
// Handles proof-of-work generation, segment processing, and communication

use common::{
    Segment, SegmentId, SegmentResult, ProofOfWork, WorkerId, WorkerNode, 
    ComputeCapacity, SegmentStatus,
};
use sha2::{Sha256, Digest};
use std::time::Instant;

/// Main segment processor
pub struct SegmentProcessor {
    worker_id: WorkerId,
    compute_capacity: ComputeCapacity,
    pow_difficulty: u32,
}

impl SegmentProcessor {
    pub fn new(worker_id: WorkerId, compute_capacity: ComputeCapacity) -> Self {
        Self {
            worker_id,
            compute_capacity,
            pow_difficulty: 20, // Difficulty: 20 leading zeros in SHA256
        }
    }

    /// Main entry point: process a segment
    pub async fn process_segment(&self, segment: &Segment) -> Result<SegmentResult, ProcessingError> {
        let start_time = Instant::now();

        // 1. Validate segment
        self.validate_segment(segment)?;

        // 2. Fetch segment data from IPFS
        let segment_data = self.fetch_from_ipfs(&segment.data_hash).await?;

        // 3. Process the segment based on rendering type
        let processed_data = self.render_segment(&segment_data, segment).await?;

        // 4. Generate proof of work
        let proof = self.generate_proof_of_work(&processed_data).await?;

        // 5. Upload result to IPFS
        let result_hash = self.upload_to_ipfs(&processed_data).await?;

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        // 6. Create result record
        Ok(SegmentResult {
            segment_id: segment.id,
            worker_id: self.worker_id.clone(),
            result_hash,
            result_size: processed_data.len() as u64,
            processing_time_ms,
            proof_of_work: proof,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Validate segment integrity
    fn validate_segment(&self, segment: &Segment) -> Result<(), ProcessingError> {
        // Check that segment is properly formed
        if segment.segment_index == u32::MAX {
            return Err(ProcessingError::InvalidSegment("Invalid segment index".into()));
        }

        // Check data size is reasonable
        if segment.data_size == 0 {
            return Err(ProcessingError::InvalidSegment("Empty segment data".into()));
        }

        if segment.data_size > 10_000_000_000 {
            // 10GB max
            return Err(ProcessingError::InvalidSegment("Segment too large".into()));
        }

        Ok(())
    }

    /// Fetch segment data from IPFS network
    async fn fetch_from_ipfs(&self, ipfs_hash: &str) -> Result<Vec<u8>, ProcessingError> {
        // In production, this uses actual IPFS client
        // For now, mock implementation
        
        println!("[Worker {}] Fetching from IPFS: {}", self.worker_id.0, ipfs_hash);

        // Simulate network latency
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // In real implementation:
        // let client = IpfsHttpClient::default();
        // let data = client.cat(ipfs_hash).collect::<Vec<u8>>().await?;

        // Mock: return dummy data based on hash
        Ok(vec![42u8; 1024 * 1024]) // 1MB dummy data
    }

    /// Render/process the segment based on rendering type
    async fn render_segment(
        &self,
        segment_data: &[u8],
        segment: &Segment,
    ) -> Result<Vec<u8>, ProcessingError> {
        // Dispatch to appropriate renderer based on task parameters
        let rendering_type = segment
            .parameters
            .get("rendering_type")
            .map(|s| s.as_str())
            .unwrap_or("default");

        match rendering_type {
            "image_processing" => self.process_image(segment_data).await,
            "video_frame" => self.process_video_frame(segment_data).await,
            "ml_inference" => self.process_ml_inference(segment_data).await,
            "audio_processing" => self.process_audio(segment_data).await,
            "3d_render" => self.process_3d(segment_data).await,
            _ => self.process_default(segment_data).await,
        }
    }

    /// Image processing: filters, color correction, scaling
    async fn process_image(&self, data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
        println!("[Worker {}] Processing image segment...", self.worker_id.0);

        // Simulate image processing work
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // In production, use actual image library (image, ndarray, etc.)
        // For now, simulate by modifying data
        let mut processed = data.to_vec();

        // Simulate some computation (e.g., blur filter)
        for i in 0..processed.len() {
            processed[i] = processed[i]
                .wrapping_add(42)
                .wrapping_mul(13)
                .wrapping_add(1);
        }

        Ok(processed)
    }

    /// Video frame rendering
    async fn process_video_frame(&self, data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
        println!("[Worker {}] Processing video frame...", self.worker_id.0);

        // Simulate frame processing (might use ffmpeg, opencv, etc.)
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let mut processed = data.to_vec();
        for i in 0..processed.len() {
            processed[i] = (processed[i] as f64 * 0.95) as u8;
        }

        Ok(processed)
    }

    /// ML inference on data chunk
    async fn process_ml_inference(&self, data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
        println!("[Worker {}] Running ML inference...", self.worker_id.0);

        // Simulate ML computation
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // In production: load model, run inference
        // For now, generate inference output (e.g., class labels)
        let mut output = Vec::new();
        for chunk in data.chunks(32) {
            let prediction = chunk[0] % 10; // Simulate class 0-9
            output.push(prediction);
        }

        Ok(output)
    }

    /// Audio processing
    async fn process_audio(&self, data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
        println!("[Worker {}] Processing audio...", self.worker_id.0);

        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        // Simulate audio filtering (EQ, compression, normalization)
        let mut processed = data.to_vec();
        for i in 0..processed.len() {
            processed[i] = processed[i].saturating_mul(1); // Normalize
        }

        Ok(processed)
    }

    /// 3D rendering
    async fn process_3d(&self, data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
        println!("[Worker {}] Rendering 3D scene...", self.worker_id.0);

        // 3D rendering is expensive
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // In production: use blender, three.js, rusttype for GPU rendering
        let mut rendered = vec![0u8; data.len() * 2]; // Output larger (includes depth, normals)
        for i in 0..data.len() {
            rendered[i * 2] = data[i];
            rendered[i * 2 + 1] = (data[i] as u16 >> 8) as u8;
        }

        Ok(rendered)
    }

    /// Default processing (passthrough with minimal work)
    async fn process_default(&self, data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
        println!("[Worker {}] Processing with default handler...", self.worker_id.0);

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(data.to_vec())
    }

    /// Generate cryptographic proof of work
    /// This proves the worker actually did the computation
    async fn generate_proof_of_work(&self, data: &[u8]) -> Result<ProofOfWork, ProcessingError> {
        println!("[Worker {}] Generating proof of work...", self.worker_id.0);

        let start = Instant::now();

        // 1. Hash the actual computation result
        let mut hasher = Sha256::new();
        hasher.update(data);
        let computation_hash = format!("{:x}", hasher.finalize());

        // 2. Find nonce that produces hash with N leading zeros (difficulty)
        let mut nonce = 0u64;
        let target_zeros = self.pow_difficulty;
        let mut hash = String::new();

        loop {
            // Hash: SHA256(computation_hash + nonce)
            let mut hasher = Sha256::new();
            hasher.update(&computation_hash);
            hasher.update(nonce.to_le_bytes());
            hash = format!("{:x}", hasher.finalize());

            // Check for leading zeros
            let leading_zeros = hash.chars().take_while(|c| *c == '0').count();
            if leading_zeros >= target_zeros as usize {
                break;
            }

            nonce += 1;

            // Prevent infinite loops (timeout after 1 minute)
            if start.elapsed().as_secs() > 60 {
                return Err(ProcessingError::ProofOfWorkTimeout);
            }
        }

        let work_factor = start.elapsed().as_secs_f64() / 60.0; // Estimated CPU-minutes

        Ok(ProofOfWork {
            nonce,
            difficulty: target_zeros,
            hash: hash.clone(),
            computation_hash,
            work_factor,
        })
    }

    /// Upload result to IPFS
    async fn upload_to_ipfs(&self, data: &[u8]) -> Result<String, ProcessingError> {
        println!(
            "[Worker {}] Uploading {} bytes to IPFS...",
            self.worker_id.0,
            data.len()
        );

        // Simulate IPFS upload
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Generate IPFS hash (SHA256 of data)
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = format!("Qm{:x}", hasher.finalize());

        println!("[Worker {}] Uploaded to IPFS: {}", self.worker_id.0, hash);

        Ok(hash)
    }
}

/// Errors that can occur during segment processing
#[derive(Debug, Clone)]
pub enum ProcessingError {
    InvalidSegment(String),
    IpfsFetchError(String),
    RenderingError(String),
    ProofOfWorkTimeout,
    IpfsUploadError(String),
    ComputeCapacityExceeded,
    Timeout,
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidSegment(s) => write!(f, "Invalid segment: {}", s),
            Self::IpfsFetchError(s) => write!(f, "IPFS fetch error: {}", s),
            Self::RenderingError(s) => write!(f, "Rendering error: {}", s),
            Self::ProofOfWorkTimeout => write!(f, "Proof of work computation timed out"),
            Self::IpfsUploadError(s) => write!(f, "IPFS upload error: {}", s),
            Self::ComputeCapacityExceeded => write!(f, "Compute capacity exceeded"),
            Self::Timeout => write!(f, "Processing timeout"),
        }
    }
}

impl std::error::Error for ProcessingError {}

/// Worker health metrics
#[derive(Debug, Clone)]
pub struct WorkerMetrics {
    pub segments_processed: u64,
    pub total_processing_time: u64,           // milliseconds
    pub failed_segments: u64,
    pub average_proof_difficulty_solved: u32,
    pub last_heartbeat: u64,
}

impl WorkerMetrics {
    pub fn new() -> Self {
        Self {
            segments_processed: 0,
            total_processing_time: 0,
            failed_segments: 0,
            average_proof_difficulty_solved: 0,
            last_heartbeat: 0,
        }
    }

    pub fn record_success(&mut self, processing_time_ms: u64, difficulty: u32) {
        self.segments_processed += 1;
        self.total_processing_time += processing_time_ms;
        self.average_proof_difficulty_solved = (self.average_proof_difficulty_solved 
            + difficulty) / 2;
        self.update_heartbeat();
    }

    pub fn record_failure(&mut self) {
        self.failed_segments += 1;
        self.update_heartbeat();
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.segments_processed + self.failed_segments;
        if total == 0 {
            0.0
        } else {
            (self.segments_processed as f64) / (total as f64)
        }
    }

    pub fn avg_processing_time_ms(&self) -> f64 {
        if self.segments_processed == 0 {
            0.0
        } else {
            (self.total_processing_time as f64) / (self.segments_processed as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let capacity = ComputeCapacity {
            cpu_cores: 8,
            gpu_vram_mb: 2048,
            available_disk_mb: 100_000,
            network_speed_mbps: 100,
        };
        let processor = SegmentProcessor::new(WorkerId("test-worker".into()), capacity);
        assert_eq!(processor.worker_id.0, "test-worker");
    }

    #[test]
    fn test_metrics() {
        let mut metrics = WorkerMetrics::new();
        metrics.record_success(100, 20);
        metrics.record_success(120, 20);
        metrics.record_failure();

        assert_eq!(metrics.segments_processed, 2);
        assert_eq!(metrics.failed_segments, 1);
        assert_eq!(metrics.success_rate(), 2.0 / 3.0);
    }

    #[tokio::test]
    async fn test_segment_validation() {
        let capacity = ComputeCapacity {
            cpu_cores: 8,
            gpu_vram_mb: 2048,
            available_disk_mb: 100_000,
            network_speed_mbps: 100,
        };
        let processor = SegmentProcessor::new(WorkerId("test".into()), capacity);

        let valid_segment = Segment {
            id: SegmentId {
                task_id: common::TaskId(1),
                segment_index: 0,
            },
            task_id: common::TaskId(1),
            segment_index: 0,
            data_hash: "QmTest".into(),
            data_size: 1000,
            parameters: Default::default(),
            assigned_to: None,
            status: SegmentStatus::Assigned,
            proof_of_work: None,
        };

        assert!(processor.validate_segment(&valid_segment).is_ok());
    }
}
