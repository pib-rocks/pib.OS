use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use tokio::sync::broadcast;
use crate::{AsyncActionNode, NodeStatus, BlackboardValue, Blackboard};

/// Abstraction for a Network Backend (e.g. Zenoh or ROS2 DDS)
pub trait NetworkBackend: Send + Sync {
    fn publish(&self, topic: &str, payload: BlackboardValue) -> Result<(), ()>;
    fn subscribe(&self, topic: &str) -> broadcast::Receiver<BlackboardValue>;
}

pub struct NetworkPublisherNode {
    backend: Arc<dyn NetworkBackend>,
    topic: String,
    payload: BlackboardValue,
}

impl NetworkPublisherNode {
    pub fn new(backend: Arc<dyn NetworkBackend>, topic: String, payload: BlackboardValue) -> Self {
        Self { backend, topic, payload }
    }
}

impl AsyncActionNode for NetworkPublisherNode {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            match self.backend.publish(&self.topic, self.payload.clone()) {
                Ok(_) => NodeStatus::Success,
                Err(_) => NodeStatus::Failure,
            }
        })
    }
}

pub struct NetworkSubscriberBridge {
    backend: Arc<dyn NetworkBackend>,
    blackboard: Blackboard,
}

impl NetworkSubscriberBridge {
    pub fn new(backend: Arc<dyn NetworkBackend>, blackboard: Blackboard) -> Self {
        Self { backend, blackboard }
    }

    pub fn start(&self, topic: &str, bb_key: &str) {
        let mut rx = self.backend.subscribe(topic);
        let bb_clone = self.blackboard.clone();
        let key_clone = bb_key.to_string();
        
        tokio::spawn(async move {
            while let Ok(value) = rx.recv().await {
                bb_clone.set(&key_clone, value);
            }
        });
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Mutex;

    pub struct MockNetworkBackend {
        pub tx: broadcast::Sender<BlackboardValue>,
        pub published_messages: Mutex<Vec<(String, BlackboardValue)>>,
    }

    impl MockNetworkBackend {
        pub fn new() -> Self {
            let (tx, _) = broadcast::channel(100);
            Self { tx, published_messages: Mutex::new(Vec::new()) }
        }
    }

    impl NetworkBackend for MockNetworkBackend {
        fn publish(&self, topic: &str, payload: BlackboardValue) -> Result<(), ()> {
            self.published_messages.lock().unwrap().push((topic.to_string(), payload));
            Ok(())
        }
        fn subscribe(&self, _topic: &str) -> broadcast::Receiver<BlackboardValue> {
            self.tx.subscribe()
        }
    }

    #[tokio::test]
    async fn test_network_publisher_node_sends_data_and_succeeds() {
        let backend = Arc::new(MockNetworkBackend::new());
        let node = NetworkPublisherNode::new(backend.clone(), "cmd_vel".to_string(), BlackboardValue::Int(100));
        
        let status = node.tick().await;
        
        assert_eq!(status, NodeStatus::Success, "Publisher node must return Success after sending");
        
        let messages = backend.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "cmd_vel");
        assert_eq!(messages[0].1, BlackboardValue::Int(100));
    }

    #[tokio::test]
    async fn test_network_subscriber_bridge_updates_blackboard() {
        let backend = Arc::new(MockNetworkBackend::new());
        let bb = Blackboard::new();
        let bridge = NetworkSubscriberBridge::new(backend.clone(), bb.clone());
        
        bridge.start("sensor/ros2_lidar", "lidar_dist");
        
        let _ = backend.tx.send(BlackboardValue::Float(5.5));
        
        tokio::task::yield_now().await;
        
        assert_eq!(bb.get("lidar_dist"), Some(BlackboardValue::Float(5.5)), "Subscriber bridge must write network data to blackboard");
    }
}
