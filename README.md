# ternary-tidepool: Small protected environments for agent experimentation

`TidePool`, `PoolBoundary`, `PoolConditions`, `PoolObserver`, `PoolDrain`, and `PoolSpawning` — a sandbox for testing agents before releasing them into the fleet.

## Why This Exists

Before an agent goes live in the fleet, you want to know how it behaves. Will it stay in bounds? How does it respond to different conditions? Does it interact well with other agents? This crate provides isolated "tide pools" — small, configurable environments where agents can be spawned, observed, and drained without affecting the real fleet. Inspired by Oracle1's Tide Pool interconnection layer.

## Core Concepts

- **TidePool** — The main sandbox. Contains agents, a boundary, conditions, an observer, and a spawner. Named for identification.
- **PoolBoundary** — Keeps agents contained. Checks positions against a limit. In strict mode, agents can't move out of bounds. Tracks violation count.
- **PoolConditions** — Configurable environment: temperature (-1/0/+1), pressure (-1/0/+1), turbulence (0-100), and max agent count.
- **PoolObserver** — Watches the pool without affecting it. Records observations (agent ID + state + timestamp) and can tally state distributions.
- **PoolDrain** — Resets the pool. Records how many agents were removed.
- **PoolSpawning** — Factory for creating test agents with auto-incrementing IDs.
- **AgentState** — Ternary state for agents: Negative (-1), Neutral (0), Positive (+1).

## Quick Start

```toml
[dependencies]
ternary-tidepool = "0.1"
```

```rust
use ternary_tidepool::{
    TidePool, PoolConditions, AgentState, PoolObserver
};

// Create a pool with custom conditions
let mut pool = TidePool::new("experiment-42")
    .with_conditions(PoolConditions::new().with_max_agents(3).with_turbulence(50))
    .with_boundary_limit(10);

// Spawn agents
let id1 = pool.spawn(AgentState::Positive).unwrap();
let id2 = pool.spawn(AgentState::Negative).unwrap();

// Move an agent
assert!(pool.move_agent(id1, 5, 5));  // in bounds
assert!(!pool.move_agent(id2, 100, 0)); // out of bounds

// Check state distribution
let (neg, neu, pos) = pool.count_by_state();
assert_eq!((neg, neu, pos), (1, 0, 1));

// Drain when done
let drain = pool.drain();
assert_eq!(drain.drained_count(), 2);
```

## API Overview

| Type | What it is |
|------|-----------|
| `TidePool` | Main sandbox environment for agent experiments |
| `PoolBoundary` | Containment check with strict mode and violation counting |
| `PoolConditions` | Configurable environment (temperature, pressure, turbulence, capacity) |
| `PoolObserver` | Read-only observer that records agent states |
| `PoolDrain` | Reset command that tracks how many agents were removed |
| `PoolSpawning` | Factory for creating test agents with auto-incrementing IDs |
| `PoolAgent` | An agent in the pool with ID, state, and position |
| `AgentId` | Unique identifier for a pool agent |
| `AgentState` | Ternary state: Negative, Neutral, Positive |

## How It Works

A `TidePool` owns its agent pool, boundary, observer, and spawner. When you `spawn` an agent, the spawner assigns an auto-incrementing ID, the observer records the initial state, and the agent is added to the pool. Spawning fails if the pool is at capacity.

`move_agent` checks the boundary before moving. In strict mode (default), agents that would leave the boundary stay put. The boundary tracks violation count even in non-strict mode, so you can detect misbehavior.

`drain` removes all agents and clears observations, returning a `PoolDrain` that records the count. This is a clean reset — the pool is ready for a new experiment.

`PoolObserver` is intentionally read-only. It records `(AgentId, AgentState, Instant)` tuples and can tally the distribution of states. It's bounded at 500 observations (oldest dropped first).

## Known Limitations

- Pools are entirely in-memory and single-threaded. No concurrency support.
- `PoolObserver` uses `Instant` which is not serializable across process restarts.
- Boundary checks use axis-aligned distance (Manhattan-style), not Euclidean.
- No agent-to-agent interaction modeling — agents are independent.
- Pool conditions (temperature, pressure, turbulence) are stored but not automatically applied to agent behavior. The consuming code must interpret them.

## Use Cases

- **Strategy testing**: Spawn agents with different initial states and observe which strategies converge to positive outcomes.
- **Boundary stress testing**: Set a tight boundary and measure how often agents violate it under different conditions.
- **Capacity planning**: Set `max_agents` and verify that spawning correctly fails at the limit.
- **Pre-release validation**: Before deploying an agent to the real fleet, run it in a tide pool to verify it doesn't drift, overflow, or misbehave.

## Ecosystem Context

Part of the SuperInstance ternary fleet library. This is the *testing* layer. Agents trained or validated in tide pools graduate to the real fleet, where they interact with `ternary-helm` (navigation), `ternary-anchor` (stability), and `ternary-current` (information flow). Inspired by Oracle1's Tide Pool concept for interconnection testing.

## License

MIT

## See Also
- **ternary-reef** — related
- **ternary-harbor** — related
- **ternary-room** — related
- **ternary-ecosystem** — related
- **ternary-cell** — related

