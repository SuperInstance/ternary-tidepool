#![forbid(unsafe_code)]

//! ternary-tidepool: Small protected environments for agent experimentation.
//!
//! Provides isolated sandbox environments where agents can be tested with
//! configurable conditions, observed without side effects, drained/reset,
//! and spawned for experiments. Inspired by Oracle1's Tide Pool interconnection.

use std::time::Instant;

/// Unique identifier for an agent in the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(u64);

impl AgentId {
    pub fn new(id: u64) -> Self {
        AgentId(id)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Ternary state for an agent: negative (-1), neutral (0), or positive (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Negative,
    Neutral,
    Positive,
}

impl AgentState {
    pub fn to_ternary(self) -> i8 {
        match self {
            AgentState::Negative => -1,
            AgentState::Neutral => 0,
            AgentState::Positive => 1,
        }
    }
}

/// Boundary that keeps agents contained within the pool.
#[derive(Debug, Clone)]
pub struct PoolBoundary {
    strict: bool,
    violation_count: u32,
}

impl PoolBoundary {
    pub fn new(strict: bool) -> Self {
        PoolBoundary {
            strict,
            violation_count: 0,
        }
    }

    /// Check if an agent's position is within bounds.
    /// Returns true if the agent is contained, false if it crossed the boundary.
    pub fn check(&mut self, x: i64, y: i64, limit: i64) -> bool {
        if x.abs() <= limit && y.abs() <= limit {
            true
        } else {
            self.violation_count += 1;
            false
        }
    }

    pub fn is_strict(&self) -> bool {
        self.strict
    }

    pub fn violation_count(&self) -> u32 {
        self.violation_count
    }
}

/// Configurable environment conditions for the pool.
#[derive(Debug, Clone)]
pub struct PoolConditions {
    temperature: i8, // -1 cold, 0 normal, +1 hot
    pressure: i8,    // -1 low, 0 normal, +1 high
    turbulence: u8,  // 0-100
    max_agents: usize,
}

impl PoolConditions {
    pub fn new() -> Self {
        PoolConditions {
            temperature: 0,
            pressure: 0,
            turbulence: 0,
            max_agents: 100,
        }
    }

    pub fn with_temperature(mut self, t: i8) -> Self {
        self.temperature = t.clamp(-1, 1);
        self
    }

    pub fn with_pressure(mut self, p: i8) -> Self {
        self.pressure = p.clamp(-1, 1);
        self
    }

    pub fn with_turbulence(mut self, t: u8) -> Self {
        self.turbulence = t.min(100);
        self
    }

    pub fn with_max_agents(mut self, n: usize) -> Self {
        self.max_agents = n;
        self
    }

    pub fn temperature(&self) -> i8 {
        self.temperature
    }

    pub fn pressure(&self) -> i8 {
        self.pressure
    }

    pub fn turbulence(&self) -> u8 {
        self.turbulence
    }

    pub fn max_agents(&self) -> usize {
        self.max_agents
    }
}

impl Default for PoolConditions {
    fn default() -> Self {
        Self::new()
    }
}

/// A single agent in the pool.
#[derive(Debug, Clone)]
pub struct PoolAgent {
    id: AgentId,
    state: AgentState,
    position: (i64, i64),
    created_at: Instant,
}

impl PoolAgent {
    pub fn new(id: AgentId, state: AgentState) -> Self {
        PoolAgent {
            id,
            state,
            position: (0, 0),
            created_at: Instant::now(),
        }
    }

    pub fn id(&self) -> AgentId {
        self.id
    }

    pub fn state(&self) -> AgentState {
        self.state
    }

    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
    }

    pub fn position(&self) -> (i64, i64) {
        self.position
    }

    pub fn move_to(&mut self, x: i64, y: i64) {
        self.position = (x, y);
    }
}

/// Observes the pool without affecting it.
#[derive(Debug, Clone)]
pub struct PoolObserver {
    observations: Vec<(AgentId, AgentState, Instant)>,
    max_observations: usize,
}

impl PoolObserver {
    pub fn new() -> Self {
        PoolObserver {
            observations: Vec::new(),
            max_observations: 500,
        }
    }

    /// Record an observation.
    pub fn observe(&mut self, id: AgentId, state: AgentState) {
        if self.observations.len() >= self.max_observations {
            self.observations.remove(0);
        }
        self.observations.push((id, state, Instant::now()));
    }

    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Count agents in each state.
    pub fn tally(&self) -> (usize, usize, usize) {
        let mut neg = 0;
        let mut neu = 0;
        let mut pos = 0;
        for (_, state, _) in &self.observations {
            match state {
                AgentState::Negative => neg += 1,
                AgentState::Neutral => neu += 1,
                AgentState::Positive => pos += 1,
            }
        }
        (neg, neu, pos)
    }

    pub fn clear(&mut self) {
        self.observations.clear();
    }
}

impl Default for PoolObserver {
    fn default() -> Self {
        Self::new()
    }
}

/// Resets the pool to empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolDrain {
    drained_count: usize,
}

impl PoolDrain {
    pub fn new() -> Self {
        PoolDrain { drained_count: 0 }
    }

    /// Record that n agents were drained.
    pub fn record(&mut self, n: usize) {
        self.drained_count += n;
    }

    pub fn drained_count(&self) -> usize {
        self.drained_count
    }
}

impl Default for PoolDrain {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating test agents.
#[derive(Debug, Clone)]
pub struct PoolSpawning {
    next_id: u64,
}

impl PoolSpawning {
    pub fn new() -> Self {
        PoolSpawning { next_id: 1 }
    }

    /// Spawn a single agent with the given state.
    pub fn spawn(&mut self, state: AgentState) -> PoolAgent {
        let id = AgentId::new(self.next_id);
        self.next_id += 1;
        PoolAgent::new(id, state)
    }

    /// Spawn n agents with the given state.
    pub fn spawn_many(&mut self, count: usize, state: AgentState) -> Vec<PoolAgent> {
        (0..count).map(|_| self.spawn(state)).collect()
    }

    pub fn next_id(&self) -> u64 {
        self.next_id
    }
}

impl Default for PoolSpawning {
    fn default() -> Self {
        Self::new()
    }
}

/// The main tide pool: an isolated environment for agent experimentation.
#[derive(Debug)]
pub struct TidePool {
    name: String,
    conditions: PoolConditions,
    boundary: PoolBoundary,
    boundary_limit: i64,
    agents: Vec<PoolAgent>,
    observer: PoolObserver,
    spawning: PoolSpawning,
}

impl TidePool {
    pub fn new(name: impl Into<String>) -> Self {
        TidePool {
            name: name.into(),
            conditions: PoolConditions::new(),
            boundary: PoolBoundary::new(true),
            boundary_limit: 100,
            agents: Vec::new(),
            observer: PoolObserver::new(),
            spawning: PoolSpawning::new(),
        }
    }

    pub fn with_conditions(mut self, conditions: PoolConditions) -> Self {
        self.conditions = conditions;
        self
    }

    pub fn with_boundary_limit(mut self, limit: i64) -> Self {
        self.boundary_limit = limit;
        self
    }

    /// Spawn an agent into the pool.
    /// Returns the agent's ID, or None if at capacity.
    pub fn spawn(&mut self, state: AgentState) -> Option<AgentId> {
        if self.agents.len() >= self.conditions.max_agents() {
            return None;
        }
        let agent = self.spawning.spawn(state);
        let id = agent.id();
        self.observer.observe(id, state);
        self.agents.push(agent);
        Some(id)
    }

    /// Move an agent and check boundary.
    pub fn move_agent(&mut self, id: AgentId, x: i64, y: i64) -> bool {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.id() == id) {
            let contained = self.boundary.check(x, y, self.boundary_limit);
            if contained || !self.boundary.is_strict() {
                agent.move_to(x, y);
            }
            contained
        } else {
            false
        }
    }

    /// Drain the pool, removing all agents.
    pub fn drain(&mut self) -> PoolDrain {
        let mut drain = PoolDrain::new();
        drain.record(self.agents.len());
        self.agents.clear();
        self.observer.clear();
        drain
    }

    /// Count agents by state.
    pub fn count_by_state(&self) -> (usize, usize, usize) {
        let mut neg = 0;
        let mut neu = 0;
        let mut pos = 0;
        for a in &self.agents {
            match a.state() {
                AgentState::Negative => neg += 1,
                AgentState::Neutral => neu += 1,
                AgentState::Positive => pos += 1,
            }
        }
        (neg, neu, pos)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn observer(&self) -> &PoolObserver {
        &self.observer
    }

    pub fn conditions(&self) -> &PoolConditions {
        &self.conditions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_id() {
        let id = AgentId::new(42);
        assert_eq!(id.value(), 42);
    }

    #[test]
    fn agent_state_ternary() {
        assert_eq!(AgentState::Negative.to_ternary(), -1);
        assert_eq!(AgentState::Neutral.to_ternary(), 0);
        assert_eq!(AgentState::Positive.to_ternary(), 1);
    }

    #[test]
    fn pool_boundary_check_inside() {
        let mut b = PoolBoundary::new(true);
        assert!(b.check(5, 5, 10));
        assert_eq!(b.violation_count(), 0);
    }

    #[test]
    fn pool_boundary_check_outside() {
        let mut b = PoolBoundary::new(true);
        assert!(!b.check(15, 5, 10));
        assert_eq!(b.violation_count(), 1);
    }

    #[test]
    fn pool_conditions_builder() {
        let c = PoolConditions::new()
            .with_temperature(1)
            .with_pressure(-1)
            .with_turbulence(50)
            .with_max_agents(10);
        assert_eq!(c.temperature(), 1);
        assert_eq!(c.pressure(), -1);
        assert_eq!(c.turbulence(), 50);
        assert_eq!(c.max_agents(), 10);
    }

    #[test]
    fn pool_conditions_clamped() {
        let c = PoolConditions::new().with_temperature(5);
        assert_eq!(c.temperature(), 1);
    }

    #[test]
    fn pool_agent_create_and_move() {
        let mut a = PoolAgent::new(AgentId::new(1), AgentState::Neutral);
        assert_eq!(a.position(), (0, 0));
        a.move_to(10, 20);
        assert_eq!(a.position(), (10, 20));
    }

    #[test]
    fn pool_agent_set_state() {
        let mut a = PoolAgent::new(AgentId::new(1), AgentState::Neutral);
        a.set_state(AgentState::Positive);
        assert_eq!(a.state(), AgentState::Positive);
    }

    #[test]
    fn pool_observer_observe_and_tally() {
        let mut o = PoolObserver::new();
        o.observe(AgentId::new(1), AgentState::Negative);
        o.observe(AgentId::new(2), AgentState::Neutral);
        o.observe(AgentId::new(3), AgentState::Positive);
        let (neg, neu, pos) = o.tally();
        assert_eq!((neg, neu, pos), (1, 1, 1));
        assert_eq!(o.observation_count(), 3);
    }

    #[test]
    fn pool_drain() {
        let mut d = PoolDrain::new();
        d.record(5);
        d.record(3);
        assert_eq!(d.drained_count(), 8);
    }

    #[test]
    fn pool_spawning_spawn() {
        let mut s = PoolSpawning::new();
        let a = s.spawn(AgentState::Positive);
        assert_eq!(a.id().value(), 1);
        assert_eq!(s.next_id(), 2);
    }

    #[test]
    fn pool_spawning_spawn_many() {
        let mut s = PoolSpawning::new();
        let agents = s.spawn_many(5, AgentState::Neutral);
        assert_eq!(agents.len(), 5);
        assert_eq!(s.next_id(), 6);
    }

    #[test]
    fn tidepool_spawn() {
        let mut pool = TidePool::new("test");
        let id = pool.spawn(AgentState::Positive).unwrap();
        assert_eq!(id.value(), 1);
        assert_eq!(pool.agent_count(), 1);
    }

    #[test]
    fn tidepool_spawn_at_capacity() {
        let mut pool = TidePool::new("test")
            .with_conditions(PoolConditions::new().with_max_agents(2));
        pool.spawn(AgentState::Neutral);
        pool.spawn(AgentState::Neutral);
        assert!(pool.spawn(AgentState::Neutral).is_none());
    }

    #[test]
    fn tidepool_move_agent() {
        let mut pool = TidePool::new("test");
        let id = pool.spawn(AgentState::Neutral).unwrap();
        assert!(pool.move_agent(id, 10, 10));
    }

    #[test]
    fn tidepool_move_agent_out_of_bounds() {
        let mut pool = TidePool::new("test").with_boundary_limit(5);
        let id = pool.spawn(AgentState::Neutral).unwrap();
        assert!(!pool.move_agent(id, 100, 0));
    }

    #[test]
    fn tidepool_drain() {
        let mut pool = TidePool::new("test");
        pool.spawn(AgentState::Positive);
        pool.spawn(AgentState::Negative);
        let drain = pool.drain();
        assert_eq!(drain.drained_count(), 2);
        assert_eq!(pool.agent_count(), 0);
    }

    #[test]
    fn tidepool_count_by_state() {
        let mut pool = TidePool::new("test");
        pool.spawn(AgentState::Negative);
        pool.spawn(AgentState::Negative);
        pool.spawn(AgentState::Neutral);
        pool.spawn(AgentState::Positive);
        let (neg, neu, pos) = pool.count_by_state();
        assert_eq!((neg, neu, pos), (2, 1, 1));
    }

    #[test]
    fn tidepool_name() {
        let pool = TidePool::new("experiment-1");
        assert_eq!(pool.name(), "experiment-1");
    }

    #[test]
    fn tidepool_conditions_applied() {
        let pool = TidePool::new("test")
            .with_conditions(PoolConditions::new().with_temperature(1).with_max_agents(5));
        assert_eq!(pool.conditions().temperature(), 1);
        assert_eq!(pool.conditions().max_agents(), 5);
    }
}
