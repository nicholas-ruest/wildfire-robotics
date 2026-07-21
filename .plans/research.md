# Wildfire Robotics

## National Program:
- Wildfire Robotics Canada

## Control Platform:
- Wildfire Robotics Command

## Robot Fleet:
- Wildfire Robotics Fleet

## Remote Bases:
- Wildfire Robotics Stations

## Autonomous Transport:
- Wildfire Robotics Carriers

## Direct-Attack Machines:
- Wildfire Robotics Firefighters

---

## Description:
Wildfire Robotics is an AI-controlled fleet of autonomous machines designed to predict wildfire threats, prepare forest containment zones, and directly fight fires in conditions too dangerous for humans.

---

# Project Breakdown with GitHub URLs and Package/Crate Identifiers

## The Rust "brain" core
- **RuVector** — [github.com/ruvnet/RuVector](https://github.com/ruvnet/RuVector) — Rust-native vector/graph/GNN memory database. Install via `npm install ruvector` (Node.js bindings with OnnxEmbedder/VectorDB) or the Rust crates directly (`ruvector-core`, `ruvector-graph`, `ruvector-collections`, `ruvector-hybrid`, `ruvector-postgres`, `ruvector-lsm-ann`, `rvlite` for edge/embedded).
- **ruv-FANN** — [github.com/ruvnet/ruv-FANN](https://github.com/ruvnet/ruv-FANN) — Rust FANN rewrite plus the Neuro-Divergent forecasting suite and ruv-swarm. Install with `npx ruv-swarm@latest init --claude`, `npm install -g ruv-swarm`, or `cargo install ruv-swarm-cli`.
- **rvm** — [github.com/ruvnet/rvm](https://github.com/ruvnet/rvm) — Bare-metal Rust runtime for agent workloads (no VMs/containers). It's a 14-crate workspace built from source with commands like `cargo check` / `cargo build --target aarch64-unknown-none -p rvm-kernel --release`. Crates include: *rvm-types, rvm-hal, rvm-cap, rvm-witness, rvm-proof, rvm-partition, rvm-sched, rvm-memory, rvm-coherence, rvm-boot, rvm-wasm, rvm-security, rvm-kernel, rvm-gpu*. Not currently published to crates.io as a single package; vendor/depend on it via git.
- **daa** — [github.com/ruvnet/daa](https://github.com/ruvnet/daa) — Rust SDK for self-governing agents plus the Prime distributed ML framework. Crates: *daa-orchestrator*, *daa-rules*, *daa-economy*, *daa-ai*, *daa-chain*, *daa-compute*, *daa-swarm* (several marked "coming soon"), plus *daa-prime-core*, *daa-prime-dht*, *daa-prime-trainer*, *daa-prime-coordinator*, *daa-prime-cli* added to Cargo.toml.
- **sublinear-time-solver** — [github.com/ruvnet/sublinear-time-solver](https://github.com/ruvnet/sublinear-time-solver) — Rust+WASM sparse linear solver. Install with `npx sublinear-time-solver mcp` (or serve), or `npm install sublinear-time-solver`.

## The actual "robot brain" component
- **agentic-robotics** — [github.com/ruvnet/agentic-robotics](https://github.com/ruvnet/agentic-robotics) — Rust-core robotics middleware with TypeScript/Node bindings, ROS2 bridge, MCP server, swarm coordination. Install with `npm install -g agentic-robotics` or `npm install agentic-robotics`. Scoped packages include: @agentic-robotics/core, @agentic-robotics/cli, @agentic-robotics/mcp. Platform binaries: @agentic-robotics/linux-x64-gnu (published), @agentic-robotics/linux-arm64-gnu, @agentic-robotics/darwin-x64, @agentic-robotics/darwin-arm64 (coming soon). Underlying crates: agentic-robotics-core, agentic-robotics-rt, agentic-robotics-mcp, agentic-robotics embedded., agentic–roboto–rics-node.
—
github.com…