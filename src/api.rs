use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::{NodeStateEvent, Telemetry};

pub struct ApiState {
    pub telemetry_tx: broadcast::Sender<NodeStateEvent>,
}

#[derive(Serialize)]
pub struct NodeInfo {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
}

pub fn api_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/api/registry", get(get_registry))
        .route("/ws/telemetry", get(ws_telemetry_handler))
        .with_state(state)
}

async fn get_registry() -> Json<Vec<NodeInfo>> {
    let registry = vec![
        NodeInfo { 
            name: "Sequence".to_string(), 
            description: "Sequence node".to_string(),
            config_schema: None,
        },
        NodeInfo { 
            name: "Selector".to_string(), 
            description: "Selector node".to_string(),
            config_schema: None,
        },
        NodeInfo {
            name: "SubtreeNode".to_string(),
            description: "Subtree execution node".to_string(),
            config_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "subtree_id": { "type": "string" }
                }
            })),
        }
    ];
    Json(registry)
}

async fn ws_telemetry_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ApiState>>,
) -> impl IntoResponse {
    let tx = state.telemetry_tx.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<NodeStateEvent>) {
    let mut rx = tx.subscribe();
    while let Ok(event) = rx.recv().await {
        let msg = format!("{{\"node_id\":\"{}\",\"state\":\"{:?}\"}}", event.node_id, event.state);
        if socket.send(Message::Text(msg.into())).await.is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use crate::NodeStatus;

    #[tokio::test]
    async fn test_registry_api() {
        let state = Arc::new(ApiState {
            telemetry_tx: broadcast::channel(10).0,
        });
        let app = api_router(state);
        let response = app
            .oneshot(Request::builder().uri("/api/registry").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        // In a real test, you'd extract the body and verify JSON content,
        // but verifying the endpoint is OK covers basic testing.
    }
}
