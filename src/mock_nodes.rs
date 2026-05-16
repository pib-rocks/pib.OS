use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use crate::{AsyncActionNode, NodeStatus};

pub struct SleepNode {
    duration: Duration,
}

impl SleepNode {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl AsyncActionNode for SleepNode {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            tokio::time::sleep(self.duration).await;
            NodeStatus::Success
        })
    }
}

pub struct LogNode {
    message: String,
}

impl LogNode {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl AsyncActionNode for LogNode {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            println!("LOG: {}", self.message);
            NodeStatus::Success
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_sleep_node() {
        let node = SleepNode::new(Duration::from_millis(50));
        let start = Instant::now();
        let status = node.tick().await;
        assert_eq!(status, NodeStatus::Success);
        assert!(start.elapsed().as_millis() >= 50);
    }

    #[tokio::test]
    async fn test_log_node() {
        let node = LogNode::new("Test message");
        let status = node.tick().await;
        assert_eq!(status, NodeStatus::Success);
    }
}
