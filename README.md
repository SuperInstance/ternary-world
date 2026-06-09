# ternary-world: World model for ternary simulations

A grid where each cell holds a ternary value (−1, 0, +1). Physics rules enforce conservation laws. Discrete time drives simulation ticks. An observer collects metrics. Snapshots capture full state.

## Why This Exists

Simulations need a world model that ties together agents, environments, and time. Using ternary values on the grid means every cell is directly compatible with the ternary math ecosystem — you can feed grid state into ternary-matrix, ternary-kalman, or ternary-entropy without conversion. Conservation physics ensures that changes to the grid don't violate invariants.

## Core Concepts

- **Trit** — A balanced ternary digit: Neg (−1), Zero (0), Pos (+1). The fundamental unit of grid state.
- **WorldGrid** — A 2D grid of trits. Width × height. Supports get/set by (x, y) coordinates, counts of each trit type, and integer sum.
- **WorldPhysics** — Conservation law enforcement. When a cell changes, a compensating change is applied to an adjacent cell to maintain a target sum (default: 0). If compensation is impossible, the change is rejected.
- **WorldTime** — A simple tick counter. Advance, read, reset. No wall-clock dependency.
- **WorldEvent** — Records agent-environment interactions: tick, agent ID, position, old/new values, description.
- **WorldObserver** — Collects named metric series. Record (name, value) pairs and compute averages.
- **WorldSnapshot** — Captures tick, grid, agent positions, and events for save/restore.

## Quick Start

```toml
[dependencies]
ternary-world = "0.1"
```

```rust
use ternary_world::*;

// Create a 4x4 grid with conservation physics
let physics = WorldPhysics::new();
let mut grid = WorldGrid::new(4, 4);

// Apply a change — setting (1,0) to Pos requires compensation at (2,0)
physics.apply(&mut grid, 1, 0, Trit::Pos);
assert!(physics.is_conserved(&grid));
assert_eq!(grid.get(2, 0), Some(Trit::Neg)); // compensation

// Track time and metrics
let mut time = WorldTime::new();
let mut observer = WorldObserver::new();
time.advance();
observer.record("sum", grid.sum() as f64);
assert_eq!(observer.avg("sum"), Some(0.0));
```

## API Overview

| Type | Description |
|------|-------------|
| `Trit` | Enum: Neg, Zero, Pos. Converts to/from i8. |
| `WorldGrid` | 2D grid of trits with get/set, counts, sum. |
| `WorldPhysics` | Conservation enforcement with compensating changes. |
| `WorldTime` | Discrete tick counter: advance, now, reset. |
| `WorldEvent` | Agent-environment interaction record. |
| `WorldObserver` | Named metric collection with averages. |
| `WorldSnapshot` | Full state capture for serialization. |

## How It Works

The grid is a flat `Vec<Trit>` with index = `y * width + x`. Get/set do bounds checking. Counts and sum iterate the full vector.

Conservation physics works by computing the delta (new value − old value) and checking if the resulting grid sum would match the target. If not, it tries to compensate at (x+1, y) by adjusting that cell's value by −delta. If the compensation value falls outside {-1, 0, +1}, the change is rejected entirely.

This is a simplified conservation model — real physics would distribute compensation across neighbors. The single-cell approach is deterministic and fast.

Time is a bare counter. The observer is a `HashMap<String, Vec<f64>>`. Snapshots clone everything.

## Known Limitations

- Conservation compensation only uses the cell to the right (x+1, y). If that cell is at the right edge or the compensation value exceeds ternary range, the change fails. This means not all grid configurations are reachable.
- Grid size is fixed at creation. No dynamic resizing.
- Observer metrics grow unbounded — no automatic pruning or windowing.
- WorldSnapshot clones the entire grid, which is O(width × height). For large grids, this is expensive.
- No multi-agent collision detection — the grid doesn't track agent positions directly (that's in the snapshot struct as a separate map).

## Use Cases

- **Cellular automata** — Each cell is ternary. Conservation physics creates interesting emergent behavior where patterns can't grow without compensating elsewhere.
- **Resource simulation** — Neg = depleted, Zero = neutral, Pos = abundant. Conservation means total resources stay constant.
- **Terrain generation** — Neg = water, Zero = plains, Pos = mountains. Physics ensures elevation is conserved.
- **Opinion dynamics** — Grid cells represent population segments. Trits are against/neutral/for. Conservation keeps total opinion balanced.

## Ecosystem Context

`ternary-world` is the simulation layer. Agents from `ternary-agent` operate on the grid. Rooms from `ternary-room` can map to grid regions. Ensigns from `ternary-ensign` can provide specialized behavior for grid interactions. The trit type is compatible with all ternary math crates.

## License

MIT

## See Also
- **ternary-room** — related
- **ternary-ecosystem** — related
- **ternary-cell** — related
- **ternary-life** — related
- **ternary-agent** — related

