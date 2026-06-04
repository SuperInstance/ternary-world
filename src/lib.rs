#![forbid(unsafe_code)]

//! World model for ternary simulations.
//!
//! A `WorldGrid` maps positions to ternary values (−1, 0, +1). `WorldPhysics`
//! applies conservation laws. `WorldTime` drives discrete ticks. `WorldEvent`
//! records agent-environment interactions. `WorldObserver` collects metrics.
//! `WorldSnapshot` captures full state for serialization.

use std::collections::HashMap;

// ── Trit ───────────────────────────────────────────────────────────────────

/// A balanced ternary digit: −1, 0, or +1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

impl Trit {
    pub fn to_i8(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Neg),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Pos),
            _ => None,
        }
    }
}

// ── World Grid ─────────────────────────────────────────────────────────────

/// A 2D grid where each cell holds a ternary value.
#[derive(Debug, Clone)]
pub struct WorldGrid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Trit>,
}

impl WorldGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Trit::Zero; width * height],
        }
    }

    /// Get the linear index for (x, y). Returns None if out of bounds.
    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    /// Get the value at (x, y). Returns None if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> Option<Trit> {
        self.index(x, y).map(|i| self.cells[i])
    }

    /// Set the value at (x, y). Returns false if out of bounds.
    pub fn set(&mut self, x: usize, y: usize, val: Trit) -> bool {
        match self.index(x, y) {
            Some(i) => {
                self.cells[i] = val;
                true
            }
            None => false,
        }
    }

    /// Count of each trit type.
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut neg = 0;
        let mut zero = 0;
        let mut pos = 0;
        for &t in &self.cells {
            match t {
                Trit::Neg => neg += 1,
                Trit::Zero => zero += 1,
                Trit::Pos => pos += 1,
            }
        }
        (neg, zero, pos)
    }

    /// Sum of all cell values as an integer.
    pub fn sum(&self) -> i64 {
        self.cells.iter().map(|t| t.to_i8() as i64).sum()
    }

    /// Total number of cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

// ── World Physics ──────────────────────────────────────────────────────────

/// Conservation laws applied as physics rules.
///
/// The primary law: the total sum of the grid must be conserved (remain zero
/// by default). When a cell is changed, a compensating change is applied to
/// maintain the target sum.
#[derive(Debug, Clone)]
pub struct WorldPhysics {
    /// The target sum the grid should maintain.
    pub target_sum: i64,
}

impl WorldPhysics {
    pub fn new() -> Self {
        Self { target_sum: 0 }
    }

    pub fn with_target_sum(mut self, target: i64) -> Self {
        self.target_sum = target;
        self
    }

    /// Apply a change at (x, y) to `new_val`, then compensate to maintain
    /// the target sum by adjusting (x+1, y) if possible.
    ///
    /// Returns true if the change was applied (including compensation),
    /// false if out of bounds or compensation impossible.
    pub fn apply(&self, grid: &mut WorldGrid, x: usize, y: usize, new_val: Trit) -> bool {
        let old = match grid.get(x, y) {
            Some(v) => v,
            None => return false,
        };
        if old == new_val {
            return true; // no change needed
        }
        let delta = new_val.to_i8() as i64 - old.to_i8() as i64;
        let current_sum = grid.sum();
        let desired_new_sum = current_sum + delta;

        if desired_new_sum == self.target_sum {
            // Direct change maintains conservation
            grid.set(x, y, new_val);
            return true;
        }

        // Try to compensate at (x+1, y)
        let cx = x + 1;
        if cx >= grid.width {
            return false; // can't compensate
        }
        let comp_current = grid.get(cx, y).unwrap().to_i8() as i64;
        let needed = comp_current - delta;
        let comp_new = match Trit::from_i8(needed as i8) {
            Some(t) => t,
            None => return false, // compensation value out of ternary range
        };
        grid.set(x, y, new_val);
        grid.set(cx, y, comp_new);
        true
    }

    /// Check if the grid satisfies conservation (sum == target).
    pub fn is_conserved(&self, grid: &WorldGrid) -> bool {
        grid.sum() == self.target_sum
    }
}

impl Default for WorldPhysics {
    fn default() -> Self {
        Self::new()
    }
}

// ── World Time ─────────────────────────────────────────────────────────────

/// Discrete time driver. Each tick advances the simulation by one step.
#[derive(Debug, Clone)]
pub struct WorldTime {
    pub tick: u64,
}

impl WorldTime {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    /// Advance one tick.
    pub fn advance(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    /// Current tick.
    pub fn now(&self) -> u64 {
        self.tick
    }

    /// Reset to zero.
    pub fn reset(&mut self) {
        self.tick = 0;
    }
}

impl Default for WorldTime {
    fn default() -> Self {
        Self::new()
    }
}

// ── World Event ────────────────────────────────────────────────────────────

/// An interaction between an agent and the environment.
#[derive(Debug, Clone)]
pub struct WorldEvent {
    pub tick: u64,
    pub agent_id: u64,
    pub x: usize,
    pub y: usize,
    pub old_val: Trit,
    pub new_val: Trit,
    pub description: String,
}

// ── World Observer ─────────────────────────────────────────────────────────

/// Collects metrics from the simulation over time.
#[derive(Debug, Clone)]
pub struct WorldObserver {
    metrics: HashMap<String, Vec<f64>>,
}

impl WorldObserver {
    pub fn new() -> Self {
        Self { metrics: HashMap::new() }
    }

    /// Record a metric value.
    pub fn record(&mut self, name: &str, value: f64) {
        self.metrics.entry(name.to_string()).or_default().push(value);
    }

    /// Get all recorded values for a metric.
    pub fn get(&self, name: &str) -> &[f64] {
        self.metrics.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Average of a metric. Returns None if no values.
    pub fn avg(&self, name: &str) -> Option<f64> {
        let vals = self.metrics.get(name)?;
        if vals.is_empty() {
            return None;
        }
        Some(vals.iter().sum::<f64>() / vals.len() as f64)
    }

    /// Number of metrics being tracked.
    pub fn metric_count(&self) -> usize {
        self.metrics.len()
    }

    /// Total number of data points across all metrics.
    pub fn total_points(&self) -> usize {
        self.metrics.values().map(|v| v.len()).sum()
    }
}

impl Default for WorldObserver {
    fn default() -> Self {
        Self::new()
    }
}

// ── World Snapshot ─────────────────────────────────────────────────────────

/// Full state of the world at a point in time.
#[derive(Debug, Clone)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub grid: WorldGrid,
    pub agent_positions: HashMap<u64, (usize, usize)>,
    pub events: Vec<WorldEvent>,
}

impl WorldSnapshot {
    /// Create a snapshot from the current world state.
    pub fn capture(
        tick: u64,
        grid: &WorldGrid,
        agent_positions: &HashMap<u64, (usize, usize)>,
        events: &[WorldEvent],
    ) -> Self {
        Self {
            tick,
            grid: grid.clone(),
            agent_positions: agent_positions.clone(),
            events: events.to_vec(),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trit_conversions() {
        assert_eq!(Trit::Neg.to_i8(), -1);
        assert_eq!(Trit::Zero.to_i8(), 0);
        assert_eq!(Trit::Pos.to_i8(), 1);
        assert_eq!(Trit::from_i8(-1), Some(Trit::Neg));
        assert_eq!(Trit::from_i8(0), Some(Trit::Zero));
        assert_eq!(Trit::from_i8(1), Some(Trit::Pos));
        assert_eq!(Trit::from_i8(2), None);
    }

    #[test]
    fn grid_new_defaults() {
        let g = WorldGrid::new(3, 2);
        assert_eq!(g.width, 3);
        assert_eq!(g.height, 2);
        assert_eq!(g.len(), 6);
        assert_eq!(g.sum(), 0);
    }

    #[test]
    fn grid_get_set() {
        let mut g = WorldGrid::new(3, 3);
        assert!(g.set(1, 2, Trit::Pos));
        assert_eq!(g.get(1, 2), Some(Trit::Pos));
        assert_eq!(g.get(0, 0), Some(Trit::Zero));
        assert!(!g.set(5, 5, Trit::Neg)); // out of bounds
        assert_eq!(g.get(5, 5), None);
    }

    #[test]
    fn grid_counts() {
        let mut g = WorldGrid::new(3, 1);
        g.set(0, 0, Trit::Neg);
        g.set(1, 0, Trit::Zero);
        g.set(2, 0, Trit::Pos);
        let (neg, zero, pos) = g.counts();
        assert_eq!((neg, zero, pos), (1, 1, 1));
    }

    #[test]
    fn grid_sum() {
        let mut g = WorldGrid::new(3, 1);
        g.set(0, 0, Trit::Pos);
        g.set(1, 0, Trit::Pos);
        g.set(2, 0, Trit::Neg);
        assert_eq!(g.sum(), 1); // 1 + 1 - 1
    }

    #[test]
    fn grid_index_out_of_bounds() {
        let g = WorldGrid::new(2, 2);
        assert!(g.index(0, 0).is_some());
        assert!(g.index(2, 0).is_none());
        assert!(g.index(0, 2).is_none());
    }

    #[test]
    fn physics_direct_change_conserved() {
        let physics = WorldPhysics::new();
        let mut grid = WorldGrid::new(2, 1);
        grid.set(0, 0, Trit::Pos);
        grid.set(1, 0, Trit::Neg);
        // grid sum = 0, changing (0,0) to Zero with delta -1
        // compensation at (1,0): currently -1, needs -1 - (-1) = 0
        assert!(physics.apply(&mut grid, 0, 0, Trit::Zero));
        assert!(physics.is_conserved(&grid));
    }

    #[test]
    fn physics_compensated_change() {
        let physics = WorldPhysics::new();
        let mut grid = WorldGrid::new(2, 1);
        // All zeros, sum=0. Set (0,0) to Pos (+1 delta)
        // Compensate at (1,0): currently 0, needs 0 - 1 = -1 → Neg
        assert!(physics.apply(&mut grid, 0, 0, Trit::Pos));
        assert_eq!(grid.get(0, 0), Some(Trit::Pos));
        assert_eq!(grid.get(1, 0), Some(Trit::Neg));
        assert!(physics.is_conserved(&grid));
    }

    #[test]
    fn physics_no_compensation_possible() {
        let physics = WorldPhysics::new();
        let mut grid = WorldGrid::new(1, 1); // no neighbor to compensate
        assert!(!physics.apply(&mut grid, 0, 0, Trit::Pos));
    }

    #[test]
    fn physics_is_conserved() {
        let physics = WorldPhysics::new();
        let grid = WorldGrid::new(4, 4);
        assert!(physics.is_conserved(&grid));
    }

    #[test]
    fn physics_nonzero_target() {
        let physics = WorldPhysics::new().with_target_sum(2);
        let mut grid = WorldGrid::new(2, 1);
        grid.set(0, 0, Trit::Pos);
        grid.set(1, 0, Trit::Pos);
        assert!(physics.is_conserved(&grid));
    }

    #[test]
    fn time_advance() {
        let mut t = WorldTime::new();
        assert_eq!(t.now(), 0);
        t.advance();
        assert_eq!(t.now(), 1);
        t.advance();
        assert_eq!(t.now(), 2);
    }

    #[test]
    fn time_reset() {
        let mut t = WorldTime::new();
        t.advance();
        t.advance();
        t.reset();
        assert_eq!(t.now(), 0);
    }

    #[test]
    fn observer_record_and_avg() {
        let mut obs = WorldObserver::new();
        obs.record("energy", 1.0);
        obs.record("energy", 2.0);
        obs.record("energy", 3.0);
        assert_eq!(obs.get("energy"), &[1.0, 2.0, 3.0]);
        assert_eq!(obs.avg("energy"), Some(2.0));
        assert_eq!(obs.avg("missing"), None);
    }

    #[test]
    fn observer_counts() {
        let mut obs = WorldObserver::new();
        obs.record("a", 1.0);
        obs.record("b", 2.0);
        obs.record("a", 3.0);
        assert_eq!(obs.metric_count(), 2);
        assert_eq!(obs.total_points(), 3);
    }

    #[test]
    fn snapshot_capture() {
        let grid = WorldGrid::new(2, 2);
        let mut positions = HashMap::new();
        positions.insert(1u64, (0usize, 0usize));
        let snap = WorldSnapshot::capture(5, &grid, &positions, &[]);
        assert_eq!(snap.tick, 5);
        assert_eq!(snap.agent_positions.len(), 1);
    }

    #[test]
    fn world_event_fields() {
        let e = WorldEvent {
            tick: 1,
            agent_id: 42,
            x: 3,
            y: 4,
            old_val: Trit::Zero,
            new_val: Trit::Pos,
            description: "agent moved".into(),
        };
        assert_eq!(e.agent_id, 42);
        assert_eq!(e.new_val, Trit::Pos);
    }

    #[test]
    fn grid_empty() {
        let g = WorldGrid::new(0, 0);
        assert!(g.is_empty());
    }

    #[test]
    fn time_default() {
        let t = WorldTime::default();
        assert_eq!(t.now(), 0);
    }

    #[test]
    fn physics_apply_no_change() {
        let physics = WorldPhysics::new();
        let mut grid = WorldGrid::new(2, 1);
        // Setting to same value (Zero) should succeed trivially
        assert!(physics.apply(&mut grid, 0, 0, Trit::Zero));
        assert!(physics.is_conserved(&grid));
    }
}
