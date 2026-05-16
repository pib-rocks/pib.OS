# pib.OS - The Reactive Behavior Tree Middleware

**pib.OS** is the open-source, rust-based operating system and middleware for the `pib` robotics project. 

It aims to democratize robotics by replacing rigid, callback-based systems with a highly reactive, node-based **Behavior Tree (BT)** architecture. Designed with a *Visual-First* mindset, pib.OS serves as the invisible, memory-safe backend for drag-and-drop robot programming.

## Vision: The "Unity-Moment" for Robotics
1. **Visual-First (No-Code/Low-Code):** Build robotic behaviors by connecting logical blocks.
2. **Safety by Design:** Written entirely in Rust. No Segfaults, no Race Conditions.
3. **True Asynchronicity:** Every action node is a native `Future`, allowing true parallel execution of I/O bounds without blocking the tree.
4. **Hardware Abstraction:** "Bring your own robot" - hardware acts purely as reactive data providers.

## Current Architecture (Epic: The Reactive Core)
We are currently building the fundamental Behavior Tree node structures using strict **Test-Driven Development (TDD)**.


### Implemented Node Types & Features
*   **`AsyncActionNode` Trait**: The foundational trait requiring a `tick()` function that yields a pinned `Future` resolving to a `NodeStatus` (`Success`, `Failure`, `Running`).
*   **`Sequence` Node**: Evaluates children sequentially. Yields `Failure` if any child fails.
*   **`Selector` Node**: Evaluates children sequentially but acts as a fallback. Yields `Success` as soon as any child succeeds.
*   **`Parallel` Node**: Spawns and awaits multiple children *concurrently* using `futures::join_all`. Evaluates success based on a configurable `success_threshold`.
*   **Decorator Nodes (`Inverter`, `Timeout`)**: Modifies the behavior of children (e.g., hard-canceling a hanging I/O Future via Tokio's timeout).
*   **`Condition` Node**: Synchronously reads values (e.g., from the Blackboard) to make instant routing decisions without yielding.

### Tick Engine & Memory
*   **`TickEngine`**: The core loop using `tokio::time::interval` to guarantee precise Hz execution rates.
*   **Zero-Copy `Blackboard`**: Lock-free concurrent data bus utilizing `Arc<RwLock<HashMap>>` to distribute sensor data safely among hundreds of nodes.
*   **`ScopedBlackboard`**: Data isolation allowing sub-trees to have local scopes while mapping explicit keys up to the parent Blackboard.

## Getting Started


### Prerequisites
*   [Rust Toolchain](https://rustup.rs/) (edition 2021)
*   Cargo

### Build & Test
The core is built using a strict TDD approach. To run the test suite:
```bash
cargo test
```

## Contributing (TDD Workflow)
We enforce a RED-GREEN-REFACTOR workflow for all logic additions:
1.  **Write a test** that fails according to the acceptance criteria.
2.  **Implement the logic** to make the test pass.
3.  **Refactor** the code for safety and performance (Zero-Copy).
