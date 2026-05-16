use pib_os::api::{api_router, ApiState};
use pib_os::mock_nodes::{LogNode, SleepNode};
use pib_os::zenoh_backend::ZenohBackend;
use pib_os::{AsyncActionNode, NodeStatus, Sequence, TickEngine, Telemetry};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    println!("Starting pib.OS Standalone Runner...");

    // Initialize Zenoh backend (PR-1246)
    let _zenoh_backend = ZenohBackend::new().await;
    println!("Zenoh backend initialized.");

    // Initialize Telemetry for API server
    let telemetry = Telemetry::new();
    let state = Arc::new(ApiState {
        telemetry_tx: telemetry.tx.clone(),
    });

    // Setup API server (PR-1244)
    let app = api_router(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API server listening on 0.0.0.0:3000");

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Setup Mock Nodes (PR-1245)
    let log_node = Box::new(LogNode::new("Starting long running task"));
    let sleep_node = Box::new(SleepNode::new(Duration::from_secs(2)));
    let log_node_2 = Box::new(LogNode::new("Task completed"));

    // Telemetry wrapper for the root sequence so UI has something to observe
    // For simplicity, we just run a sequence of mock nodes.
    let sequence = Box::new(Sequence::new(vec![log_node, sleep_node, log_node_2]));

    // Start TickEngine
    let engine = TickEngine::new(sequence, 10);
    println!("Behavior Tree engine running at 10 Hz...");

    // We can simulate emitting some telemetry while running
    telemetry.report_state("root_sequence", NodeStatus::Running);
    
    let status = engine.run().await;
    println!("Behavior Tree finished with status: {:?}", status);

    telemetry.report_state("root_sequence", status);
    
    // Give it a short time so telemetry messages can be sent out over WS before shutting down
    tokio::time::sleep(Duration::from_millis(100)).await;
}
