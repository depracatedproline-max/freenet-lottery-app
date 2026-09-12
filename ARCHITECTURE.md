# Freenet Lottery Web App - Architecture

## System Overview

The Freenet Lottery Web App is a distributed content rendering system that incentivizes compute resource sharing through a lottery mechanism.

## Core Components

### 1. Coordinator Node
**Responsibilities:**
- Task reception and decomposition
- Worker node management and heartbeat monitoring
- Result aggregation and validation
- Lottery drawing and reward distribution
- State synchronization via Freenet Scaffold

**Key Modules:**
- `coordinator/task_manager.rs` - Handles task breakdown and assignment
- `coordinator/worker_registry.rs` - Manages active worker nodes
- `coordinator/result_aggregator.rs` - Combines segment results
- `coordinator/lottery_engine.rs` - Lottery logic and winner selection

### 2. Worker Node
**Responsibilities:**
- Receive content segments from coordinator
- Perform rendering/processing work
- Return processed segments with proof-of-work
- Earn lottery entries for each contribution

**Key Modules:**
- `worker/segment_processor.rs` - Core rendering engine
- `worker/proof_of_work.rs` - Work verification
- `worker/freenet_client.rs` - P2P communication

### 3. Web Frontend
**Responsibilities:**
- User interface for task submission
- Worker node registration
- Lottery participation tracking
- Result viewing and history

**Key Modules:**
- `frontend/pages/submit_task.tsx` - Task submission interface
- `frontend/pages/worker_dashboard.tsx` - Worker statistics
- `frontend/pages/lottery_results.tsx` - Results and winners

### 4. Freenet Scaffold Integration
**Responsibilities:**
- Distributed state management
- Node consensus on lottery outcomes
- Blockchain-style immutable record of transactions

**Pattern:**
```rust
// Mergeable state structures using Freenet Scaffold
struct LotteryState {
    epoch: u64,
    participants: Map<NodeId, ParticipantRecord>,
    lottery_entries: Vec<LotteryEntry>,
    results: Map<TaskId, ProcessingResult>,
}

impl MergeableState for LotteryState {
    fn merge(&mut self, other: Self) { /* ... */ }
}
```

## Content Rendering Workflow

### Phase 1: Task Submission
```
1. User submits content rendering task
   - Content: [Image/Video/Data to render]
   - Parameters: [Resolution, format, processing type]
   - Reward pool: [Lottery prize amount]

2. Coordinator validates task
   - Check content integrity
   - Estimate compute requirements
   - Initialize lottery pool
```

### Phase 2: Task Decomposition
```
1. Break content into N segments
   - Segment 1: [Pixels 0-1000]
   - Segment 2: [Pixels 1001-2000]
   - ...
   - Segment N: [Pixels (N-1)*1000 to N*1000]

2. Create processing metadata
   - Segment hash (for verification)
   - Processing parameters
   - Deadline timestamp
```

### Phase 3: Distribution & Processing
```
1. Broadcast segments to available workers
   - Worker 1 ◄─ Segment 1
   - Worker 2 ◄─ Segment 2
   - Worker 3 ◄─ Segment 3

2. Workers process segments
   - Render content
   - Generate proof-of-work
   - Return results + proof

3. Coordinator validates results
   - Verify proof-of-work
   - Check segment integrity
   - Award lottery entry for each worker
```

### Phase 4: Aggregation & Lottery
```
1. Assemble complete rendered content
   - Merge all segments
   - Validate continuity
   - Store final result

2. Draw lottery
   - Number of entries = number of segments processed
   - Winner: 1 entry drawn
   - Losers: Receive rendered content + participation reward

3. Distribute rewards
   - Winners: Lottery prize from pool
   - All participants: Content access + entry credit
```

## Distributed Processing Examples

### Example 1: Image Rendering
```
Task: Render 4K image (3840x2160 pixels)

Segmentation:
- 16 segments × 16 segments = 256 segments total
- Each segment: 240x135 pixels
- Assigned to 256 available worker nodes

Processing:
- Workers apply filters, scaling, transformations
- Proofs of work: GPU computation hashes
- Time: ~2 seconds per segment

Aggregation:
- Reassemble 256 segments in 16×16 grid
- Verify continuity at segment boundaries
- Output: Complete rendered 4K image
```

### Example 2: Video Frame Rendering
```
Task: Render 100 frames of animation

Segmentation:
- 100 frames → 100 segments (one frame per worker)
- Each frame: 1920x1080, 24-bit color

Processing:
- Workers render individual frames
- Apply effects, transitions, compositing
- Return completed frame buffer

Aggregation:
- Sequence frames in temporal order
- Encode to video format
- Output: Completed video file
```

### Example 3: Data Processing
```
Task: Process 1GB dataset (ML inference)

Segmentation:
- Split into 1000 chunks of 1MB each
- Send to distributed worker pool

Processing:
- Workers run inference on their chunk
- Return predictions + confidence scores

Aggregation:
- Collect all predictions
- Merge results by original data order
- Generate final inference report
```

## Lottery Mechanism

### Entry System
- 1 segment processed = 1 lottery entry
- Multiple workers can participate in same task
- Entries are accumulated in participant's account

### Drawing
```rust
fn draw_lottery(entries: Vec<LotteryEntry>) -> Winner {
    // Weighted by work difficulty
    // Verifiable randomness using VRF
    let random_index = vrf_random() % entries.len();
    entries[random_index].worker_id
}
```

### Rewards Structure
```
Winner: 
  - Base reward: X credits
  - Performance bonus: Y credits (if all segments completed in time)
  - Streak bonus: Z credits (if won N previous times)

Participants (non-winners):
  - Participation reward: Content access + A credits
  - Contribution credit: B credits per segment
  - Reputation increase: +1 trust score
```

## Security Considerations

1. **Proof of Work**
   - Each segment completion requires computational proof
   - Prevents sybil attacks / fake submissions

2. **Content Integrity**
   - Segment hashes verified before aggregation
   - Tampering detected at merge time

3. **Freenet Scaffold Consensus**
   - Lottery results synchronized across all nodes
   - Byzantine-fault-tolerant state merging

4. **Reputation System**
   - Workers with poor results excluded
   - Track completion rate and quality scores

## Scalability

- **Segment Count**: Configurable (10-10,000 segments per task)
- **Worker Pool**: Scales with Freenet network size
- **Throughput**: Multiple tasks can run concurrently
- **Load Balancing**: Coordinator assigns based on worker availability
