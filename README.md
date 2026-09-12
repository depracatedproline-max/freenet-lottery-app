# Freenet Lottery Web App

A **distributed content rendering lottery system** built on Freenet where computing power is shared to render content, and everyone wins—but in different ways.

## The Core Insight

**Problem Solved:**
- ❌ No sponsor/logistics coordination needed
- ❌ No real-world delivery complications
- ❌ No inventory management
- ✅ Pure software rewards that scale infinitely
- ✅ Both winners AND losers get value
- ✅ Perfect for digital content rendering

## The Win-Win Model

```
Participant contributes compute to render content segment
         ↓
    [Lottery Drawing]
         ↙          ↘
    WINNER          LOSER
         ↓              ↓
  Get rendered    Get free access
  content FIRST   to same rendered
  + Premium       content (after
    access        winner gets it)
  + Credits       + Participation
  + Priority      credits + Trust
    queue access  score increase
```

### Real Scenario

**Alice submits**: "Please render my 4K short film (10GB, 10 hours compute time)"

**What happens:**
1. Film broken into 100 segments (1 hour compute each)
2. 100 participants from Freenet network volunteer compute
3. Each renders their segment in parallel (1-2 hours real-time)
4. 100 lottery entries drawn (one per segment)
5. **Winner**: Gets their completed film ASAP + priority rendering queue for future projects
6. **Losers (99 others)**: Get free access to Alice's finished film after 24 hours + points toward their own rendering

**Alice wins too**: Her film got rendered for free. She can optionally pay to skip lottery waiting, give tips, or get instant access, but doesn't have to.

## Software Rewards Breakdown

### For Winners
- ✨ **Rendered Content** - Immediate access to completed rendering
- 🎯 **Priority Queue** - Next render jobs done first
- ⭐ **Reputation Badge** - "Lucky Contributor" status
- 💳 **Render Credits** - Free/discounted rendering for their own projects
- 🏆 **Leaderboard Placement** - Community recognition

### For Losers (Non-Winners)
- 📥 **Content Access** - Same rendered file, 24-48hr delayed release
- 📊 **Contribution Points** - Accumulate toward free rendering
- 🔓 **Access Tier Unlock** - Better content library as you participate
- 📈 **Trust Score** - Increases participation privileges
- 🎁 **Participation Badges** - Track involvement history

### For Everyone
- 🤝 **Network Effect** - More participants = more rendering capacity = faster results
- 💾 **File Seeding** - Earn credits for hosting completed renders on Freenet
- 🔄 **Content Remixing** - Combine other renders into new projects

## Tech Stack

- **Backend**: Rust (Freenet-compatible)
- **Frontend**: TypeScript/React
- **State Sync**: freenet-scaffold (distributed consensus)
- **P2P Networking**: Freenet protocol
- **Rendering Engine**: Segment-based task decomposition

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│  User Submits Content to Render                 │
│  (Video, Image, 3D, ML inference, etc)          │
└────────────────────┬────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        ↓                         ↓
    TASK COORDINATOR         FREENET NETWORK
    (Splits into             (Distributed
     segments)                participants)
        │                         │
        └────────────┬────────────┘
                     │
        ┌────────────┴────────────┐
        ↓                         ↓
    SEGMENT PROCESSING        LOTTERY DRAWING
    (100 workers render       (Verifiable random
     100 segments parallel)    winner selection)
        │                         │
        └────────────┬────────────┘
                     │
        ┌────────────┴────────────┐
        ↓                         ↓
    RESULT AGGREGATION         REWARD DISTRIBUTION
    (Merge segments →           (Credits, access,
     complete file)             reputation)
```

## Key Features

### 1. Distributed Rendering Engine
- Break large tasks into N parallel segments
- Assign to available worker nodes
- Proof-of-work validation for each segment
- Automatic aggregation when complete

### 2. Lottery System
- Each completed segment = 1 lottery entry
- Verifiable randomness (VRF-based)
- Instant payout to winner
- Transparent leaderboard

### 3. Freenet Integration
- Decentralized task coordination
- State synchronization via freenet-scaffold
- Byzantine-fault-tolerant consensus on winners
- No central server required

### 4. Content Library
- Rendered files stay on Freenet permanently
- Participants can seed for extra credits
- Build portfolio of community renders
- Remix/derivative works supported

### 5. Incentive System
- No real-world logistics
- No sponsor coordination
- Pure software value exchange
- Scales infinitely

## Getting Started

See [DEVELOPMENT.md](./DEVELOPMENT.md) for local setup.

### Quick Start: Local Network

```bash
# Terminal 1: Coordinator
cd backend/coordinator
cargo run -- --local-mode

# Terminal 2: Worker 1
cd backend/worker
cargo run -- --id worker-1

# Terminal 3: Worker 2
cd backend/worker
cargo run -- --id worker-2

# Terminal 4: Frontend
cd frontend
npm run dev
```

## Example Rendering Tasks

### Image Rendering
- 4K photo processing (blur, color grade, filters)
- 256 segments × 256 workers = 15 seconds

### Video Rendering
- 100-frame animation (3D, effects, encoding)
- 100 segments × 100 workers = parallel rendering

### AI Inference
- Large dataset processing (classification, detection)
- 1000 chunks × 1000 workers = distributed ML

### Audio Processing
- Music mastering (compression, EQ, effects)
- Multiple track segments processed in parallel

## Related Freenet Projects

- [freenet-scaffold](https://github.com/freenet/freenet-scaffold) - State synchronization
- [freenet-river](https://github.com/freenet/river) - P2P app architecture
- [freenet-agent-skills](https://github.com/freenet/freenet-agent-skills) - AI automation
- [freenet-core](https://github.com/freenet/freenet-core) - Network protocol

## Why This Works (Unlike Physical Lottery)

| Aspect | Physical Lottery | Freenet Lottery |
|--------|------------------|-----------------|
| **Rewards** | Physical goods (logistics nightmare) | Digital files (instant delivery) |
| **Scale** | Limited by inventory | Infinite scalability |
| **Sponsor Coordination** | Required (complex) | Not needed |
| **Winner Fulfillment** | Days/weeks | Instant |
| **Secondary Value** | Losers get nothing | Losers get content access |
| **Repeatability** | One-time event | Continuous operation |
| **Cost** | High (shipping, handling) | Negligible (already rendering) |
| **Fairness** | Trust-based | Blockchain-verifiable |

## License

MIT
