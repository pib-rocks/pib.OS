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

Since `pib.OS` acts as a middleware library, it is typically embedded into your robotic control application. 

### Starting the Engine and API
To start the Behavior Tree execution engine alongside the API and Telemetry server:

```rust
use pib_os::api::start_api_server;
use pib_os::parser::parse_tree;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // 1. Load a Behavior Tree configuration (e.g., exported from pib.Cerebra)
    let json_config = r#"{
        "root": { "type": "Sequence", "children": [] }
    }"#;
    
    // 2. Parse the tree
    let tree = parse_tree(json_config).expect("Invalid tree format");
    
    // 3. Start the API and WebSocket server on port 3000
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API and Telemetry Server running on ws://0.0.0.0:3000/ws/telemetry");
    
    // Serve the API endpoints
    start_api_server(listener).await.unwrap();
}
```
*(Note: The `start_api_server` and engine integration is handled asynchronously by the `tokio` runtime.)*

## Running Tests

`pib.OS` is developed strictly using **Test-Driven Development (TDD)** (RED-GREEN-REFACTOR). The test suite includes unit tests for all nodes, the blackboard, parser, and the HTTP/WebSocket APIs.

To execute the entire test suite:

```bash
cargo test
```

This will automatically download all necessary development dependencies, compile the project, and run all test modules to verify the integrity of the middleware.