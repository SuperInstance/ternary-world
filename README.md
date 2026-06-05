# ternary-world

**World model for ternary simulations**

[![ternary](https://img.shields.io/badge/ecosystem-ternary-blue)](https://github.com/orgs/SuperInstance/repositories?q=ternary)
[![tests](https://img.shields.io/badge/tests-20-green)]()

## Overview

World model for ternary simulations.

A `WorldGrid` maps positions to ternary values (−1, 0, +1). `WorldPhysics`
applies conservation laws. `WorldTime` drives discrete ticks. `WorldEvent`
records agent-environment interactions. `WorldObserver` collects metrics.
`WorldSnapshot` captures full state for serialization.

## Architecture

- **`WorldGrid`** — core data structure
- **`WorldPhysics`** — core data structure
- **`WorldTime`** — core data structure
- **`WorldEvent`** — core data structure
- **`WorldObserver`** — core data structure
- **`WorldSnapshot`** — core data structure
- **`Trit`** — state enumeration

### Key Functions

- `to_i8()`
- `from_i8()`
- `new()`
- `index()`
- `get()`
- `set()`
- `counts()`
- `sum()`
- `len()`
- `is_empty()`
- ... and 15 more

## Why Ternary?

The balanced ternary system {-1, 0, +1} (also known as Z₃) is the mathematically optimal discrete encoding:
- **More expressive than binary**: three states capture positive, neutral, and negative
- **Natural for decisions**: accept/reject/abstain, buy/hold/sell, agree/disagree/neutral
- **Self-balancing**: the 0 state acts as a universal screen, preventing pathological lock-in
- **Z₃ cyclic dynamics**: rock-paper-scissors is the only natural coordination mechanism

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | 511 |
| Test count | 20 |
| Public types | 7 |
| Public functions | 25 |

## Ecosystem

This crate is part of the **[SuperInstance Ternary Fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)**:

- **[ternary-core](https://github.com/SuperInstance/ternary-core)** — shared traits and Z₃ arithmetic
- **[ternary-grid](https://github.com/SuperInstance/ternary-grid)** — spatial grid with {-1, 0, +1} cells
- **[ternary-graph](https://github.com/SuperInstance/ternary-graph)** — ternary-weighted graph algorithms
- **[ternary-automata](https://github.com/SuperInstance/ternary-automata)** — three-state cellular automata
- **[ternary-compiler](https://github.com/SuperInstance/ternary-compiler)** — expression compiler and optimizer

200+ crates. 4,300+ tests. One pattern.

## Research Context

The ternary approach connects to several active research areas:
- **Ternary Neural Networks** (TNNs): weights constrained to {-1, 0, +1} for efficient inference
- **Huawei's ternary chip**: 7nm ternary silicon with 60% less power consumption
- **Active inference**: free energy minimization naturally maps to ternary action selection
- **Cyclic dominance**: RPS dynamics maintain biodiversity in spatial ecology
- **Z₃ group theory**: the only algebraic group on three elements is cyclic addition mod 3

## Usage

```toml
[dependencies]
ternary-world = "0.1.0"
```

```rust
use ternary_world;
```

## License

MIT
