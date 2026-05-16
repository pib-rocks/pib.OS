# pib.OS - The Reactive Behavior Tree Middleware

**pib.OS** is an open-source, Rust-based operating system and middleware tailored for the `pib` (printable intelligent bot) robotics platform.

It aims to democratize robotics by replacing rigid, callback-based systems with a highly reactive, node-based **Behavior Tree (BT)** architecture. Designed with a *Visual-First* mindset, pib.OS serves as the invisible, memory-safe backend for drag-and-drop robot programming.

## Features

### Core Behavior Tree Engine
*   **Asynchronous Execution:** Every action node is a native `Future`. `pib.OS` evaluates the behavior tree asynchronously, allowing true parallel execution of I/O bounds without blocking the entire tree.
*   **Control Nodes:** Complete set of control flow nodes including `Sequence` (fails on first failure), `Selector` (succeeds on first success), and `Parallel` (spawns concurrent executions with configurable success thresholds).
*   **Decorator Nodes:** Modifiers like `Inverter` and `Timeout` to manipulate the status of child nodes or handle hanging tasks.
*   **Condition Nodes:** Instantaneous read nodes to make synchronous routing decisions without yielding.

### State & Memory Management
*   **Zero-Copy Blackboard:** A lock-free, concurrent data bus utilizing interior mutability to distribute sensor data efficiently across hundreds of nodes.
*   **Scoped Blackboards:** Supports local scopes for sub-trees, isolating internal data while mapping explicit keys up to the parent Blackboard via Port Mapping.

### Network Interoperability & API
*   **Live Telemetry via WebSockets:** Exposes an Axum-based WebSocket server (`/ws/telemetry`) that streams real-time `NodeStateEvent`s. This allows frontends (like pib.Cerebra) to visualize the exact state of the behavior tree during runtime.
*   **Dynamic Node Registry API:** A REST endpoint (`GET /api/registry`) that enumerates all available nodes and their required data ports, allowing dynamic UI toolbox rendering.
*   **JSON Tree Parser:** Dynamically deserializes and constructs complex Behavior Trees and `ScopedBlackboard` configurations at runtime using `serde_json`.
*   **Network Bridging:** Built-in traits to bridge `pib.OS` to networks like ROS2 or Zenoh (e.g., `NetworkPublisherNode`, `NetworkSubscriberBridge`).

## Installation

### Prerequisites
*   [Rust Toolchain](https://rustup.rs/) (edition 2021 or newer)
*   `cargo` package manager

### Getting the Source Code
Clone the repository to your local machine:
```bash
git clone https://github.com/pib-rocks/pib.OS.git
cd pib.OS
```

### Building the Project
Build the library and its dependencies:
```bash
cargo build --release
```

## Usage

## Usage

With the new standalone runner, you can execute behavior trees directly via the command line.

### Running pib.OS (Standalone)

Start the execution engine alongside the API and Telemetry server:
```bash
cargo run --bin pib_os
```
This automatically starts the Behavior Tree Engine and the WebSocket API server on `ws://0.0.0.0:3000/ws/telemetry`.

### Mock Nodes for UI Testing
`pib.OS` now includes interactive mock nodes (`SleepNode` and `LogNode`) designed specifically to test live telemetry. These nodes artificially yield (pause) execution so you can visually observe the "Running" state streaming into the Cerebra UI in real-time.

### Zenoh Network Backend
The middleware is fully integrated with [Eclipse Zenoh](https://zenoh.io/). Real hardware data and commands can be routed seamlessly between the Behavior Tree's Zero-Copy Blackboard and the Zenoh network using the `ZenohBackend` implementation.


## Graphical User Interface (pib.Cerebra MVP)

The project includes a Visual-First editor MVP built with React, TypeScript, and Vite. This frontend serves as the control center (dashboard) for designing and monitoring robot behaviors.

### UI Features
*   **Drag & Drop Editor:** Visually construct behavior trees by connecting action, condition, and control flow nodes.
*   **Dynamic Node Toolbox:** (Planned Integration) Dynamically populates available nodes by querying the `pib.OS` backend registry (`/api/registry`).
*   **JSON Export:** Serializes the visual tree and local scoped variable mappings (Port Mapping) into a JSON format that the Rust backend can parse and execute.
*   **Live Telemetry:** (Planned Integration) Connects to the backend WebSocket (`/ws/telemetry`) to animate the behavior tree, showing real-time execution states (Running, Success, Failure) of each node.

### Starting the GUI
Make sure you have Node.js and `npm` installed.

1. Navigate to the `ui` directory:
   ```bash
   cd ui
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Start the development server:
   ```bash
   npm run dev
   ```
This will launch the Vite development server, usually accessible at `http://localhost:5173`.

## Running Tests

`pib.OS` is developed strictly using **Test-Driven Development (TDD)** (RED-GREEN-REFACTOR). The test suite includes unit tests for all nodes, the blackboard, parser, and the HTTP/WebSocket APIs.

To execute the entire test suite:

```bash
cargo test
```

This will automatically download all necessary development dependencies, compile the project, and run all test modules to verify the integrity of the middleware.

### End-to-End Tests (Playwright)

The UI component uses Playwright for End-to-End testing.
To run the E2E tests:

```bash
cd ui
npx playwright test --project=chromium
```
### Benchmarks & Regression Testing
Performance is critical for the Behavior Tree Engine. We use `criterion` to benchmark core components. To run the benchmark suite and protect against regressions:

```bash
cargo bench
```

We specifically track:
* **Blackboard**: Read/write latency, especially for scoped variables mapping.
* **Tree Engine**: Ticking large tree topologies, like a 100-child Sequence node.

Ensure regressions do not exceed 5% on the primary operations.