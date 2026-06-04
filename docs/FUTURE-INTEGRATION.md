# Future Integration: ternary-world

## Current State

ternary-world provides a world model for ternary simulations. `WorldGrid` maps 2D positions to `Trit` values (Neg/Zero/Pos) with `get()`, `set()`, `counts()`, and `sum()`. `WorldPhysics` enforces conservation laws: `apply(grid, x, y, new_val)` changes a cell and compensates at (x+1, y) to maintain `target_sum` (default 0). `WorldTime` drives discrete ticks. `WorldEvent` records agent-environment interactions at positions. `WorldObserver` collects named metric time series with `record()`, `avg()`, and `get()`. `WorldSnapshot` captures full state for serialization.

## Integration Opportunities

### Spreadsheet-as-World Model (Primary Integration)

The `WorldGrid` IS the spreadsheet-as-world model from the Cross-Pollination Report. Each grid cell maps to a spreadsheet cell:

- `WorldGrid::set(x, y, Trit::Pos)` = writing a value to cell A1
- `WorldPhysics::apply()` = the formula engine — changes maintain conservation laws
- `WorldEvent` = cell change audit trail (who changed what, when)
- `WorldObserver` = spreadsheet metrics (column averages, totals)

The `WorldPhysics::target_sum = 0` conservation law becomes the spreadsheet's conservation of mass: the total value across the sheet is preserved. When one cell goes up, another goes down. `ternary-spreadsheet`'s `FormulaEngine` can use `WorldPhysics` as the physics backend.

### ternary-cell → WorldGrid as CellGrid

`WorldGrid` and `CellGrid` (from ternary-cell) are complementary:

- `WorldGrid` = the environment (positions hold Trit values, physics-enforced)
- `CellGrid` = the agents (positions hold TernaryCells with tick cycles)

Integration: overlay a `CellGrid` on a `WorldGrid`. Cells read from `WorldGrid` for perception, write to `WorldGrid` for action. `WorldPhysics::apply()` enforces conservation after each cell tick. The `Tissue` level coordinate becomes `WorldTime::advance()` + `CellGrid::tick_all()` + `WorldPhysics::is_conserved()`.

### PLATO Room as WorldGrid Slice

Each PLATO room owns a rectangular region of the world grid. `Room::environment` stores the grid bounds. `RoomCoordinator::transfer()` moves an agent from one grid region to another. `WorldSnapshot::capture()` serializes the room's grid region for PLATO tile sync. The `Door` between rooms defines whether grid regions are adjacent (agents can see neighbors across room boundaries) or isolated.

### ternary-compiler → Grid State Compilation

A converged `WorldGrid` (from `WorldObserver::avg("surprise")` approaching zero) can be compiled to an ESP32 lookup table. The grid's `counts()` at each position become the compiled policy. Pipeline: evolve grid on DGX → verify conservation with `WorldPhysics::is_conserved()` → compile with `ternary-compiler` → flash to ESP32.

## Potential in Mature Systems

ternary-world becomes the "physics engine" of the spreadsheet-as-world product. Every spreadsheet cell is a `WorldGrid` position. Every formula is a `WorldPhysics::apply()` call. Sort = natural selection (reorder by fitness column). Filter = extinction (remove below threshold). The conservation law ensures the spreadsheet never "loses energy" — total value is preserved across all operations.

## Cross-Pollination Ideas

- **WorldPhysics → ternary-thermodynamics**: Replace the simple neighbor-compensation with proper thermodynamic conservation. `WorldPhysics::target_sum` becomes a free energy constraint, and compensation follows the gradient of minimum entropy production.
- **WorldObserver → PLATO tiles**: `WorldObserver::record("surprise", value)` generates tiles when surprise exceeds a threshold. High-surprise regions become knowledge tiles for PLATO.
- **WorldGrid → strategy-ecology Lotka-Volterra**: Grid positions represent ecological niches. `Trit::Pos` = prey, `Trit::Neg` = predator, `Trit::Zero` = empty. `WorldPhysics` becomes the population dynamics engine.

## Dependencies for Next Steps

1. `WorldGrid` ↔ `CellGrid` overlay integration (agent-environment coupling)
2. `WorldPhysics` → `ternary-spreadsheet::FormulaEngine` bridge
3. `WorldSnapshot` serialization format for PLATO tile sync
4. Grid region slicing for PLATO room assignments
5. `ternary-compiler` integration for converged grid → lookup table compilation
