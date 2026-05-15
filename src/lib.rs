use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeStatus {
    Success,
    Failure,
    Running,
}

/// Asynchronous Action Node trait for pib.OS behavior trees.
pub trait AsyncActionNode: Send + Sync {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>>;
}

// =====================================================================
// SEQUENCE NODE
// =====================================================================

pub struct Sequence {
    children: Vec<Box<dyn AsyncActionNode>>,
    current_child: AtomicUsize,
}

impl Sequence {
    pub fn new(children: Vec<Box<dyn AsyncActionNode>>) -> Self {
        Self {
            children,
            current_child: AtomicUsize::new(0),
        }
    }
}

impl AsyncActionNode for Sequence {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            let mut current = self.current_child.load(Ordering::SeqCst);

            while current < self.children.len() {
                let status = self.children[current].tick().await;

                match status {
                    NodeStatus::Success => {
                        current += 1;
                        self.current_child.store(current, Ordering::SeqCst);
                    }
                    NodeStatus::Failure => {
                        self.current_child.store(0, Ordering::SeqCst);
                        return NodeStatus::Failure;
                    }
                    NodeStatus::Running => {
                        return NodeStatus::Running;
                    }
                }
            }

            self.current_child.store(0, Ordering::SeqCst);
            NodeStatus::Success
        })
    }
}

// =====================================================================
// SELECTOR NODE (Fallback) - GREEN Phase
// =====================================================================

pub struct Selector {
    children: Vec<Box<dyn AsyncActionNode>>,
    current_child: AtomicUsize,
}

impl Selector {
    pub fn new(children: Vec<Box<dyn AsyncActionNode>>) -> Self {
        Self {
            children,
            current_child: AtomicUsize::new(0),
        }
    }
}

impl AsyncActionNode for Selector {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            let mut current = self.current_child.load(Ordering::SeqCst);

            while current < self.children.len() {
                let status = self.children[current].tick().await;

                match status {
                    NodeStatus::Failure => {
                        // Selector keeps trying the next child if one fails
                        current += 1;
                        self.current_child.store(current, Ordering::SeqCst);
                    }
                    NodeStatus::Success => {
                        // Selector succeeds immediately if ANY child succeeds
                        self.current_child.store(0, Ordering::SeqCst);
                        return NodeStatus::Success;
                    }
                    NodeStatus::Running => {
                        // Pause execution. Resume here next tick.
                        return NodeStatus::Running;
                    }
                }
            }

            // All children failed.
            self.current_child.store(0, Ordering::SeqCst);
            NodeStatus::Failure
        })
    }
}

// =====================================================================
// MOCKS FOR TESTING
// =====================================================================

pub struct ConfigurableMockNode {
    statuses: Vec<NodeStatus>,
    current_tick: AtomicUsize,
}

impl ConfigurableMockNode {
    pub fn new(statuses: Vec<NodeStatus>) -> Self {
        Self {
            statuses,
            current_tick: AtomicUsize::new(0),
        }
    }
}

impl AsyncActionNode for ConfigurableMockNode {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            let idx = self.current_tick.fetch_add(1, Ordering::SeqCst);
            if idx < self.statuses.len() {
                self.statuses[idx]
            } else {
                *self.statuses.last().unwrap_or(&NodeStatus::Failure)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // --- Sequence Node Tests ---

    #[tokio::test]
    async fn test_sequence_returns_success_if_all_succeed() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let sequence = Sequence::new(vec![child1, child2]);
        assert_eq!(sequence.tick().await, NodeStatus::Success);
    }

    #[tokio::test]
    async fn test_sequence_returns_failure_immediately() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let sequence = Sequence::new(vec![child1, child2]);
        assert_eq!(sequence.tick().await, NodeStatus::Failure);
    }

    #[tokio::test]
    async fn test_sequence_returns_running_and_resumes() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Running, NodeStatus::Success]));
        let sequence = Sequence::new(vec![child1, child2]);
        assert_eq!(sequence.tick().await, NodeStatus::Running);
        assert_eq!(sequence.tick().await, NodeStatus::Success);
    }

    // --- Selector Node Tests (Story PR-1221) ---

    #[tokio::test]
    async fn test_selector_returns_failure_if_all_fail() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let selector = Selector::new(vec![child1, child2]);
        
        assert_eq!(selector.tick().await, NodeStatus::Failure, "Selector must return Failure if all children fail.");
    }

    #[tokio::test]
    async fn test_selector_returns_success_immediately() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let child3 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure])); // Should not be reached
        
        let selector = Selector::new(vec![child1, child2, child3]);
        
        assert_eq!(selector.tick().await, NodeStatus::Success, "Selector must return Success immediately after a child succeeds.");
    }

    #[tokio::test]
    async fn test_selector_returns_running_and_resumes() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Running, NodeStatus::Success]));
        
        let selector = Selector::new(vec![child1, child2]);
        
        // Tick 1
        assert_eq!(selector.tick().await, NodeStatus::Running, "Selector must return Running if a child is Running.");
        
        // Tick 2 (Resume)
        assert_eq!(selector.tick().await, NodeStatus::Success, "Selector must resume and eventually return Success.");
    }
}