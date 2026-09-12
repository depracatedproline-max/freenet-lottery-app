# Freenet Lottery Web App

A distributed content rendering lottery system built on Freenet. Participants share compute resources to render content segments, with lottery rewards and results sharing for contributors.

## Core Concept

The Freenet Lottery combines:
- **Distributed Processing**: Break large content rendering tasks into segments
- **Freenet Scaffold**: State synchronization across participating nodes
- **Lottery Mechanism**: Reward contributors with lottery entries
- **Incentive Structure**: Lottery losers receive rendered results as compensation

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│         Freenet Lottery Coordinator Node                │
│  (Task Distribution, Aggregation, Lottery Drawing)      │
└────────────────┬────────────────────────────────────────┘
                 │
         ┌───────┼───────┬───────────┐
         │       │       │           │
    ┌────▼──┐ ┌──▼──┐ ┌─▼─────┐ ┌──▼──┐
    │Worker │ │Worker│ │Worker │ │Worker│
    │ Node 1│ │Node 2│ │Node 3 │ │Node N│
    └────┬──┘ └──┬──┘ └─┬─────┘ └──┬──┘
         │       │     │           │
    [Segment] [Segment][Segment] [Segment]
    Rendering Rendering Rendering Rendering
         │       │     │           │
         └───────┼─────┼───────────┘
                 │
         ┌───────▼──────────┐
         │ Result Assembly  │
         │ & Aggregation    │
         └───────┬──────────┘
                 │
         ┌───────▼──────────┐
         │ Lottery Drawing  │
         │ & Distribution   │
         └──────────────────┘
```

## Key Features

1. **Distributed Rendering Engine**
   - Split large content rendering tasks into manageable segments
   - Distribute to participating worker nodes
   - Aggregate results into complete content

2. **Freenet Integration**
   - Uses freenet-scaffold for state synchronization
   - P2P node communication
   - Decentralized task coordination

3. **Lottery System**
   - Each completed segment = lottery entry
   - Winners receive rewards
   - Losers receive completed rendered content as compensation

4. **Incentive Mechanism**
   - Participation rewards: guaranteed content access
   - Win rewards: bonus prizes (credits, priority access, etc.)
   - Community contribution tracking

## Tech Stack

- **Backend**: Rust (Freenet-compatible)
- **Frontend**: TypeScript/React
- **State Sync**: freenet-scaffold
- **P2P Networking**: Freenet protocol
- **AI Skills**: freenet-agent-skills (for automation)

## Getting Started

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed system design.
See [DEVELOPMENT.md](./DEVELOPMENT.md) for setup instructions.

## Related Freenet Projects

- [freenet-scaffold](https://github.com/freenet/freenet-scaffold) - State synchronization
- [freenet-river](https://github.com/freenet/river) - P2P app reference implementation
- [freenet-agent-skills](https://github.com/freenet/freenet-agent-skills) - AI development automation
- [freenet-core](https://github.com/freenet/freenet-core) - Core protocol

## License

MIT
