# Spreadsheet Integration: ternary-world

> How the ternary world model maps to a living spreadsheet.

## Overview

`ternary-world` provides the primitives: `WorldGrid`, `WorldPhysics`, `WorldTime`, `WorldObserver`, `WorldSnapshot`. Each of these maps directly to a spreadsheet concept. This document provides the concrete integration code.

---

## 1. WorldGrid → Spreadsheet Grid

A `WorldGrid` is a 2D array of `Trit` values (−1, 0, +1). A spreadsheet grid is a 2D array of cells. The mapping is trivial and lossless.

```rust
use ternary_world::{WorldGrid, Trit};

/// A spreadsheet world: a WorldGrid that behaves like a spreadsheet.
///
/// Each cell in the WorldGrid corresponds to a cell in the spreadsheet.
/// Position (x, y) in the WorldGrid = column x, row y in the spreadsheet.
pub struct SpreadsheetWorld {
    /// The underlying world grid
    pub grid: WorldGrid,
    /// Physics engine (conservation laws)
    pub physics: ternary_world::WorldPhysics,
    /// Time driver (tick counter)
    pub time: ternary_world::WorldTime,
    /// Observer (metric collector)
    pub observer: ternary_world::WorldObserver,
}

impl SpreadsheetWorld {
    /// Create a new spreadsheet world with given dimensions.
    pub fn new(columns: usize, rows: usize) -> Self {
        SpreadsheetWorld {
            grid: WorldGrid::new(columns, rows),
            physics: ternary_world::WorldPhysics::new(),
            time: ternary_world::WorldTime::new(),
            observer: ternary_world::WorldObserver::new(),
        }
    }

    /// Get cell value at spreadsheet position (column, row).
    /// Returns None if out of bounds.
    pub fn get_cell(&self, col: usize, row: usize) -> Option<Trit> {
        self.grid.get(col, row)
    }

    /// Set cell value at spreadsheet position (col, row) with conservation.
    /// If conservation is enabled, a compensating change is applied to the
    /// adjacent cell.
    pub fn set_cell(&mut self, col: usize, row: usize, val: Trit) -> bool {
        let result = self.physics.apply(&mut self.grid, col, row, val);
        if result {
            self.observer.record("changes", self.time.now() as f64);
        }
        result
    }

    /// Get the width (number of columns).
    pub fn width(&self) -> usize {
        self.grid.width
    }

    /// Get the height (number of rows).
    pub fn height(&self) -> usize {
        self.grid.height
    }
}
```

### Cell Reference Translation

Spreadsheet uses "A1" notation. `WorldGrid` uses `(x, y)`.

```rust
impl SpreadsheetWorld {
    /// Parse a cell reference like "A1" into WorldGrid coordinates (col, row).
    pub fn parse_ref(reference: &str) -> Option<(usize, usize)> {
        let s = reference.trim();
        let col_end = s.chars().position(|c| c.is_ascii_digit())?;
        let col_str = &s[..col_end];
        let row_str = &s[col_end..];

        let mut col: usize = 0;
        for ch in col_str.chars() {
            if !ch.is_ascii_alphabetic() {
                return None;
            }
            col = col * 26 + (ch.to_ascii_uppercase() as usize - 'A' as usize + 1);
        }
        if col == 0 {
            return None;
        }
        col -= 1;

        let row: usize = row_str.parse().ok()?;
        if row == 0 {
            return None;
        }
        Some((col, row - 1))
    }

    /// Get cell by spreadsheet reference (e.g., "B3").
    pub fn get_by_ref(&self, reference: &str) -> Option<Trit> {
        let (col, row) = Self::parse_ref(reference)?;
        self.get_cell(col, row)
    }
}
```

---

## 2. WorldPhysics → Cell Formulas

`WorldPhysics` enforces conservation laws. In spreadsheet terms, this is the formula engine that ensures invariants hold after recalculation.

### Conservation as a Formula

```rust
impl SpreadsheetWorld {
    /// Check that the entire spreadsheet satisfies conservation.
    /// Equivalent to `=SUM(ALL) == target_sum`.
    pub fn is_conserved(&self) -> bool {
        self.physics.is_conserved(&self.grid)
    }

    /// Get the current grid sum.
    /// Equivalent to `=SUM(A1:ZZ9999)`.
    pub fn sum(&self) -> i64 {
        self.grid.sum()
    }

    /// Get the count of each trit type.
    /// Equivalent to `=COUNTIF(range, -1)`, `=COUNTIF(range, 0)`, `=COUNTIF(range, 1)`.
    pub fn counts(&self) -> (usize, usize, usize) {
        self.grid.counts()
    }
}
```

### Physics Rules as Formula Patterns

| WorldPhysics Rule | Spreadsheet Formula | Effect |
|---|---|---|
| Conservation (sum = 0) | `=CONSERVE(A1:J10, 0)` | Changes compensated by neighbors |
| Conservation (sum = k) | `=CONSERVE(A1:J10, k)` | Changes compensated to maintain k |
| No conservation | `=FREE(A1)` | Cell can change freely |

```rust
impl SpreadsheetWorld {
    /// Apply a physics rule to a range of cells.
    ///
    /// This is the bridge between WorldPhysics and the formula engine.
    /// When a user types `=CONSERVE(A1:J10, 0)`, this method:
    /// 1. Reads the current values in the range
    /// 2. Computes the sum
    /// 3. If sum ≠ target, redistributes to achieve conservation
    pub fn conserve_range(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, target: i64) -> bool {
        // Count current sum in range
        let mut current_sum: i64 = 0;
        for y in y1..=y2 {
            for x in x1..=x2 {
                if let Some(t) = self.grid.get(x, y) {
                    current_sum += t.to_i8() as i64;
                }
            }
        }

        if current_sum == target {
            return true; // Already conserved
        }

        // Redistribute: adjust last cell to hit target
        let diff = (target - current_sum) as i8;
        let adjusted = match Trit::from_i8(diff) {
            Some(t) => t,
            None => return false, // Can't fit in one trit
        };
        self.grid.set(x2, y2, adjusted);

        // Verify
        let new_sum: i64 = (x1..=x2).flat_map(|x| {
            (y1..=y2).filter_map(move |y| {
                self.grid.get(x, y).map(|t| t.to_i8() as i64)
            })
        }).sum();

        new_sum == target
    }
}
```

---

## 3. WorldTime → Recalculation Cycles

Each `WorldTime::advance()` is one recalculation of the spreadsheet. The tick drives the simulation forward.

```rust
impl SpreadsheetWorld {
    /// Run one complete tick (recalculation cycle).
    ///
    /// This is the 6-phase cycle mapped to spreadsheet operations:
    /// 1. Predict  — cache expected values
    /// 2. Perceive — apply physics (conservation)
    /// 3. Surprise — compute deviation metrics
    /// 4. Vibe     — update observer
    /// 5. GC       — (placeholder for cell pruning)
    /// 6. Conserve — verify invariants
    pub fn tick(&mut self) -> u64 {
        let tick = self.time.now();

        // Phase 1: Predict — record pre-tick state
        let pre_sum = self.grid.sum();
        let (neg, zero, pos) = self.grid.counts();

        // Phase 2: Perceive — physics already applied via set_cell()
        // (In a full implementation, formulas would be evaluated here)

        // Phase 3: Surprise — metrics
        let post_sum = self.grid.sum();
        let surprise = (pre_sum - post_sum).unsigned_abs() as f64;
        self.observer.record("surprise", surprise);

        // Phase 4: Vibe — record population metrics
        self.observer.record("population_neg", neg as f64);
        self.observer.record("population_zero", zero as f64);
        self.observer.record("population_pos", pos as f64);
        self.observer.record("total_energy", self.grid.sum() as f64);

        // Phase 5: GC — (reserved for cell pruning strategy)

        // Phase 6: Conserve — verify
        let conserved = self.physics.is_conserved(&self.grid);
        self.observer.record("conserved", if conserved { 1.0 } else { 0.0 });

        self.time.advance()
    }

    /// Get the current tick number.
    pub fn now(&self) -> u64 {
        self.time.now()
    }

    /// Reset the simulation.
    pub fn reset(&mut self) {
        self.time.reset();
        self.grid = WorldGrid::new(self.grid.width, self.grid.height);
        self.observer = ternary_world::WorldObserver::new();
    }

    /// Run N ticks and return the observer's final metrics.
    pub fn run(&mut self, ticks: u64) -> &ternary_world::WorldObserver {
        for _ in 0..ticks {
            self.tick();
        }
        &self.observer
    }
}
```

---

## 4. Complete SpreadsheetWorld Implementation

Putting it all together:

```rust
use ternary_world::{WorldGrid, WorldPhysics, WorldTime, WorldObserver, WorldSnapshot, Trit};

/// A ternary world that IS a spreadsheet.
///
/// This struct unifies the world model with spreadsheet semantics.
/// Every cell is a position. Every physics rule is a formula.
/// Every tick is a recalculation.
pub struct SpreadsheetWorld {
    pub grid: WorldGrid,
    pub physics: WorldPhysics,
    pub time: WorldTime,
    pub observer: WorldObserver,
}

impl SpreadsheetWorld {
    pub fn new(columns: usize, rows: usize) -> Self {
        Self {
            grid: WorldGrid::new(columns, rows),
            physics: WorldPhysics::new(),
            time: WorldTime::new(),
            observer: WorldObserver::new(),
        }
    }

    pub fn with_target_sum(mut self, target: i64) -> Self {
        self.physics = WorldPhysics::new().with_target_sum(target);
        self
    }

    // --- Cell Operations (spreadsheet-style) ---

    pub fn get_cell(&self, col: usize, row: usize) -> Option<Trit> {
        self.grid.get(col, row)
    }

    pub fn set_cell(&mut self, col: usize, row: usize, val: Trit) -> bool {
        self.physics.apply(&mut self.grid, col, row, val)
    }

    pub fn get_by_ref(&self, reference: &str) -> Option<Trit> {
        let (col, row) = Self::parse_ref(reference)?;
        self.grid.get(col, row)
    }

    // --- Range Operations (spreadsheet-style) ---

    pub fn sum_range(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> i64 {
        let mut sum = 0i64;
        for y in y1..=y2.min(self.grid.height - 1) {
            for x in x1..=x2.min(self.grid.width - 1) {
                if let Some(t) = self.grid.get(x, y) {
                    sum += t.to_i8() as i64;
                }
            }
        }
        sum
    }

    pub fn entropy(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
        let mut counts = [0usize; 3];
        let mut total = 0;
        for y in y1..=y2.min(self.grid.height - 1) {
            for x in x1..=x2.min(self.grid.width - 1) {
                if let Some(t) = self.grid.get(x, y) {
                    match t {
                        Trit::Neg => counts[0] += 1,
                        Trit::Zero => counts[1] += 1,
                        Trit::Pos => counts[2] += 1,
                    }
                    total += 1;
                }
            }
        }
        if total == 0 {
            return 0.0;
        }
        let n = total as f64;
        counts.iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f64 / n;
                -p * p.log2()
            })
            .sum()
    }

    // --- Tick (recalculation cycle) ---

    pub fn tick(&mut self) -> u64 {
        let (neg, zero, pos) = self.grid.counts();
        self.observer.record("neg", neg as f64);
        self.observer.record("zero", zero as f64);
        self.observer.record("pos", pos as f64);
        self.observer.record("sum", self.grid.sum() as f64);
        self.observer.record("conserved", if self.physics.is_conserved(&self.grid) { 1.0 } else { 0.0 });
        self.time.advance()
    }

    pub fn run(&mut self, ticks: u64) -> &WorldObserver {
        for _ in 0..ticks {
            self.tick();
        }
        &self.observer
    }

    // --- Snapshot ---

    pub fn snapshot(&self) -> WorldSnapshot {
        WorldSnapshot::capture(
            self.time.now(),
            &self.grid,
            &std::collections::HashMap::new(),
            &[],
        )
    }

    // --- Utilities ---

    pub fn parse_ref(reference: &str) -> Option<(usize, usize)> {
        let s = reference.trim();
        let col_end = s.chars().position(|c| c.is_ascii_digit())?;
        let col_str = &s[..col_end];
        let row_str = &s[col_end..];
        let mut col: usize = 0;
        for ch in col_str.chars() {
            if !ch.is_ascii_alphabetic() { return None; }
            col = col * 26 + (ch.to_ascii_uppercase() as usize - 'A' as usize + 1);
        }
        if col == 0 { return None; }
        col -= 1;
        let row: usize = row_str.parse().ok()?;
        if row == 0 { return None; }
        Some((col, row - 1))
    }

    pub fn width(&self) -> usize { self.grid.width }
    pub fn height(&self) -> usize { self.grid.height }
    pub fn now(&self) -> u64 { self.time.now() }
    pub fn is_conserved(&self) -> bool { self.physics.is_conserved(&self.grid) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spreadsheet_world_new() {
        let sw = SpreadsheetWorld::new(10, 5);
        assert_eq!(sw.width(), 10);
        assert_eq!(sw.height(), 5);
        assert!(sw.is_conserved()); // All zeros, sum = 0
    }

    #[test]
    fn set_cell_with_conservation() {
        let mut sw = SpreadsheetWorld::new(3, 3);
        assert!(sw.set_cell(0, 0, Trit::Pos));
        // Compensation at (1, 0) should be Neg
        assert_eq!(sw.get_cell(1, 0), Some(Trit::Neg));
        assert!(sw.is_conserved());
    }

    #[test]
    fn parse_refs() {
        assert_eq!(SpreadsheetWorld::parse_ref("A1"), Some((0, 0)));
        assert_eq!(SpreadsheetWorld::parse_ref("B3"), Some((1, 2)));
        assert_eq!(SpreadsheetWorld::parse_ref("AA1"), Some((26, 0)));
    }

    #[test]
    fn tick_advances_time() {
        let mut sw = SpreadsheetWorld::new(3, 3);
        assert_eq!(sw.now(), 0);
        sw.tick();
        assert_eq!(sw.now(), 1);
    }

    #[test]
    fn run_ticks() {
        let mut sw = SpreadsheetWorld::new(3, 3);
        sw.run(100);
        assert_eq!(sw.now(), 100);
        assert_eq!(sw.observer.total_points(), 600); // 6 metrics × 100 ticks
    }

    #[test]
    fn entropy_calculation() {
        let mut sw = SpreadsheetWorld::new(3, 1);
        sw.grid.set(0, 0, Trit::Neg).unwrap();
        sw.grid.set(1, 0, Trit::Zero).unwrap();
        sw.grid.set(2, 0, Trit::Pos).unwrap();
        let e = sw.entropy(0, 0, 2, 0);
        assert!((e - 1.585).abs() < 0.01); // log2(3) ≈ 1.585
    }

    #[test]
    fn sum_range() {
        let mut sw = SpreadsheetWorld::new(3, 1);
        // Note: physics.apply handles conservation, so direct grid.set for testing
        assert!(sw.grid.set(0, 0, Trit::Pos));
        assert!(sw.grid.set(1, 0, Trit::Pos));
        assert!(sw.grid.set(2, 0, Trit::Neg));
        assert_eq!(sw.sum_range(0, 0, 2, 0), 1); // 1 + 1 - 1
    }
}
```

---

## 5. Usage Examples

### Creating a World

```rust
// A 26×100 spreadsheet (columns A-Z, rows 1-100)
let mut world = SpreadsheetWorld::new(26, 100);

// Set cell A1 to +1 (compensated by B1 → -1)
world.set_cell(0, 0, Trit::Pos);

// Check conservation
assert!(world.is_conserved());
```

### Running a Simulation

```rust
let mut world = SpreadsheetWorld::new(10, 10);

// Initialize with random values (ignoring conservation for setup)
for y in 0..10 {
    for x in 0..10 {
        let val = match (x + y) % 3 {
            0 => Trit::Neg,
            1 => Trit::Zero,
            _ => Trit::Pos,
        };
        world.grid.set(x, y, val);
    }
}

// Run 1000 ticks
let observer = world.run(1000);

// Check metrics
println!("Average energy: {:?}", observer.avg("sum"));
println!("Conservation rate: {:?}", observer.avg("conserved"));
```

### Taking Snapshots

```rust
let mut world = SpreadsheetWorld::new(5, 5);
world.set_cell(0, 0, Trit::Pos);
world.tick();

let snapshot = world.snapshot();
println!("Snapshot at tick {}", snapshot.tick);
```

---

## Summary

| ternary-world Concept | Spreadsheet Concept | Bridge Method |
|---|---|---|
| `WorldGrid` | Cell grid | `SpreadsheetWorld::grid` |
| `WorldPhysics` | Conservation formulas | `set_cell()` with compensation |
| `WorldTime` | Recalculation counter | `tick()` / `run(n)` |
| `WorldObserver` | Metrics dashboard | `observer.record()` / `observer.avg()` |
| `WorldSnapshot` | Save state | `snapshot()` |
| `Trit` | Cell value (−1, 0, +1) | Direct mapping |
