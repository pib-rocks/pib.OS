use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::future::join_all;

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
// SELECTOR NODE (Fallback)
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
                        current += 1;
                        self.current_child.store(current, Ordering::SeqCst);
                    }
                    NodeStatus::Success => {
                        self.current_child.store(0, Ordering::SeqCst);
                        return NodeStatus::Success;
                    }
                    NodeStatus::Running => {
                        return NodeStatus::Running;
                    }
                }
            }

            self.current_child.store(0, Ordering::SeqCst);
            NodeStatus::Failure
        })
    }
}

// =====================================================================
// PARALLEL NODE - GREEN Phase
// =====================================================================

pub struct Parallel {
    children: Vec<Box<dyn AsyncActionNode>>,
    success_threshold: usize,
}

impl Parallel {
    pub fn new(children: Vec<Box<dyn AsyncActionNode>>, success_threshold: usize) -> Self {
        Self {
            children,
            success_threshold,
        }
    }
}

impl AsyncActionNode for Parallel {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            // Wir sammeln alle Futures der Kinder
            let mut futures = Vec::new();
            for child in &self.children {
                futures.push(child.tick());
            }

            // Wir führen alle gleichzeitig (!) aus
            let results = join_all(futures).await;

            let mut successes = 0;
            let mut failures = 0;

            for status in results {
                match status {
                    NodeStatus::Success => successes += 1,
                    NodeStatus::Failure => failures += 1,
                    NodeStatus::Running => {}
                }
            }

            let max_allowed_failures = self.children.len().saturating_sub(self.success_threshold);

            if successes >= self.success_threshold {
                // Threshold erreicht: Der Parallel-Knoten ist erfolgreich!
                NodeStatus::Success
            } else if failures > max_allowed_failures {
                // Rechnerisch unmöglich, den Threshold noch zu erreichen.
                NodeStatus::Failure
            } else {
                // Wir warten noch auf Kinder.
                NodeStatus::Running
            }
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


// =====================================================================
// DECORATOR NODES: Inverter & Timeout - RED Phase
// =====================================================================

pub struct Inverter {
    child: Box<dyn AsyncActionNode>,
}

impl Inverter {
    pub fn new(child: Box<dyn AsyncActionNode>) -> Self {
        Self { child }
    }
}

impl AsyncActionNode for Inverter {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            let status = self.child.tick().await;
            match status {
                NodeStatus::Success => NodeStatus::Failure,
                NodeStatus::Failure => NodeStatus::Success,
                NodeStatus::Running => NodeStatus::Running,
            }
        })
    }
}

use std::time::Duration;

pub struct Timeout {
    child: Box<dyn AsyncActionNode>,
    duration: Duration,
}

impl Timeout {
    pub fn new(child: Box<dyn AsyncActionNode>, duration: Duration) -> Self {
        Self { child, duration }
    }
}

impl AsyncActionNode for Timeout {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            // tokio::time::timeout cancels the internal future if duration is reached
            let result = tokio::time::timeout(self.duration, self.child.tick()).await;
            
            match result {
                Ok(status) => status, // Child finished in time
                Err(_) => NodeStatus::Failure, // Timeout occurred!
            }
        })
    }
}


// =====================================================================
// CONDITION NODE - RED Phase
// =====================================================================

pub struct Condition {
    predicate: Box<dyn Fn() -> bool + Send + Sync>,
}

impl Condition {
    pub fn new<F>(predicate: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Box::new(predicate),
        }
    }
}

impl AsyncActionNode for Condition {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async move {
            if (self.predicate)() {
                NodeStatus::Success
            } else {
                NodeStatus::Failure
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

    // --- Selector Node Tests ---

    #[tokio::test]
    async fn test_selector_returns_failure_if_all_fail() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let selector = Selector::new(vec![child1, child2]);
        assert_eq!(selector.tick().await, NodeStatus::Failure);
    }

    #[tokio::test]
    async fn test_selector_returns_success_immediately() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let child3 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure])); 
        let selector = Selector::new(vec![child1, child2, child3]);
        assert_eq!(selector.tick().await, NodeStatus::Success);
    }

    #[tokio::test]
    async fn test_selector_returns_running_and_resumes() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Running, NodeStatus::Success]));
        let selector = Selector::new(vec![child1, child2]);
        assert_eq!(selector.tick().await, NodeStatus::Running);
        assert_eq!(selector.tick().await, NodeStatus::Success);
    }

    // --- Parallel Node Tests (Story PR-1222) ---

    #[tokio::test]
    async fn test_parallel_returns_success_when_threshold_reached() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let child3 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Running]));
        
        let parallel = Parallel::new(vec![child1, child2, child3], 2);
        
        assert_eq!(parallel.tick().await, NodeStatus::Success, "Parallel must return Success when M children succeed.");
    }

    #[tokio::test]
    async fn test_parallel_returns_failure_when_impossible_to_reach_threshold() {
        let child1 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let child2 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        let child3 = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Running]));
        
        let parallel = Parallel::new(vec![child1, child2, child3], 2);
        
        assert_eq!(parallel.tick().await, NodeStatus::Failure, "Parallel must return Failure when (N - M + 1) children fail.");
    }

    // --- Decorator Tests (Story PR-1223) ---

    #[tokio::test]
    async fn test_inverter_swaps_success_and_failure() {
        let success_child = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Success]));
        let failure_child = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Failure]));
        
        let inverter1 = Inverter::new(success_child);
        let inverter2 = Inverter::new(failure_child);
        
        assert_eq!(inverter1.tick().await, NodeStatus::Failure, "Inverter must return Failure if child succeeds");
        assert_eq!(inverter2.tick().await, NodeStatus::Success, "Inverter must return Success if child fails");
    }

    #[tokio::test]
    async fn test_inverter_passes_running() {
        let running_child = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Running]));
        let inverter = Inverter::new(running_child);
        
        assert_eq!(inverter.tick().await, NodeStatus::Running, "Inverter must pass Running transparently");
    }

    struct SleepNode;
    impl AsyncActionNode for SleepNode {
        fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
            Box::pin(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                NodeStatus::Success
            })
        }
    }

    #[tokio::test]
    async fn test_timeout_returns_failure_if_child_hangs() {
        // Child takes 50ms, timeout is 10ms
        let child = Box::new(SleepNode);
        let timeout_node = Timeout::new(child, std::time::Duration::from_millis(10));
        
        assert_eq!(timeout_node.tick().await, NodeStatus::Failure, "Timeout must return Failure if child takes too long");
    }

    // --- Condition Tests (Story PR-1224) ---

    #[tokio::test]
    async fn test_condition_returns_success_when_true() {
        let condition = Condition::new(|| true);
        assert_eq!(condition.tick().await, NodeStatus::Success, "Condition must return Success if predicate is true");
    }

    #[tokio::test]
    async fn test_condition_returns_failure_when_false() {
        let condition = Condition::new(|| false);
        assert_eq!(condition.tick().await, NodeStatus::Failure, "Condition must return Failure if predicate is false");
    }
}
