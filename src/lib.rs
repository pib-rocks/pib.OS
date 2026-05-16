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


// =====================================================================
// TICK ENGINE - RED Phase
// =====================================================================

pub struct TickEngine {
    root: Box<dyn AsyncActionNode>,
    tick_interval: Duration,
}

impl TickEngine {
    pub fn new(root: Box<dyn AsyncActionNode>, hz: u32) -> Self {
        Self {
            root,
            tick_interval: Duration::from_millis(1000 / hz as u64),
        }
    }

    pub async fn run(&self) -> NodeStatus {
        let mut interval = tokio::time::interval(self.tick_interval);

        loop {
            // Wait for the next scheduled tick according to Hz
            interval.tick().await;

            let status = self.root.tick().await;

            match status {
                NodeStatus::Success | NodeStatus::Failure => {
                    // Tree completed its logic, halt the engine
                    return status;
                }
                NodeStatus::Running => {
                    // Tree is still working (e.g. driving), wait for next tick
                    continue;
                }
            }
        }
    }
}


// =====================================================================
// BLACKBOARD - RED Phase
// =====================================================================
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq)]
pub enum BlackboardValue {
    Int(i32),
    Float(f64),
    Text(String),
    Bool(bool),
}

#[derive(Clone)]
pub struct Blackboard {
    data: Arc<RwLock<HashMap<String, BlackboardValue>>>,
}

impl Blackboard {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: &str, value: BlackboardValue) {
        // acquire write lock, this will block until all active reads finish
        if let Ok(mut map) = self.data.write() {
            map.insert(key.to_string(), value);
        }
    }

    pub fn get(&self, key: &str) -> Option<BlackboardValue> {
        // acquire read lock, multiple threads can read concurrently
        if let Ok(map) = self.data.read() {
            map.get(key).cloned()
        } else {
            None
        }
    }
}


// =====================================================================
// SCOPED BLACKBOARD - RED Phase
// =====================================================================

#[derive(Clone)]
pub struct ScopedBlackboard {
    parent: Blackboard,
    local: Blackboard,
    mapping: HashMap<String, String>, // Maps local_key -> parent_key
}

impl ScopedBlackboard {
    pub fn new(parent: Blackboard, mapping: HashMap<String, String>) -> Self {
        Self {
            parent,
            local: Blackboard::new(),
            mapping,
        }
    }

    pub fn set(&self, key: &str, value: BlackboardValue) {
        if let Some(parent_key) = self.mapping.get(key) {
            self.parent.set(parent_key, value);
        } else {
            self.local.set(key, value);
        }
    }

    pub fn get(&self, key: &str) -> Option<BlackboardValue> {
        if let Some(parent_key) = self.mapping.get(key) {
            self.parent.get(parent_key)
        } else {
            self.local.get(key)
        }
    }
}


// =====================================================================
// PUB/SUB BRIDGE - RED Phase
// =====================================================================
use tokio::sync::mpsc;

pub struct PubSubBridge {
    blackboard: Blackboard,
}

impl PubSubBridge {
    pub fn new(blackboard: Blackboard) -> Self {
        Self { blackboard }
    }

    /// Spawns a background task that listens to a channel (simulating Pub/Sub)
    /// and updates the blackboard key automatically.
    pub fn subscribe(&self, _topic: &str, bb_key: &str) -> mpsc::Sender<BlackboardValue> {
        let (tx, mut rx) = mpsc::channel(100);
        
        let bb_clone = self.blackboard.clone();
        let key_clone = bb_key.to_string();
        
        // GREEN PHASE: Spawn a lightweight Tokio task that listens forever
        // and instantly mirrors incoming pub/sub messages to the Blackboard
        tokio::spawn(async move {
            while let Some(value) = rx.recv().await {
                bb_clone.set(&key_clone, value);
            }
        });
        
        tx
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

    // --- Tick Engine Tests (Story PR-1213) ---

    #[tokio::test]
    async fn test_engine_halts_on_success() {
        let root = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Running, NodeStatus::Success]));
        let engine = TickEngine::new(root, 100);
        
        assert_eq!(engine.run().await, NodeStatus::Success, "Engine must halt and return Success when tree completes");
    }

    #[tokio::test]
    async fn test_engine_ticks_at_given_rate() {
        // Node takes 3 ticks to succeed: Running, Running, Success
        let root = Box::new(ConfigurableMockNode::new(vec![NodeStatus::Running, NodeStatus::Running, NodeStatus::Success]));
        let engine = TickEngine::new(root, 10); // 10Hz = 100ms per tick
        
        let start = tokio::time::Instant::now();
        engine.run().await;
        let elapsed = start.elapsed();
        
        // 3 ticks at 100ms each should take at least ~200ms (interval fires immediately for the first tick)
        assert!(elapsed.as_millis() >= 200, "Engine finished too quickly: {} ms", elapsed.as_millis());
        assert!(elapsed.as_millis() < 400, "Engine took too long: {} ms", elapsed.as_millis());
    }

    // --- Blackboard Tests (Story PR-1215) ---

    #[tokio::test]
    async fn test_blackboard_returns_none_for_missing_key() {
        let bb = Blackboard::new();
        // This will pass in RED phase because we hardcoded None
        assert_eq!(bb.get("missing"), None, "Must return None for missing keys");
    }

    #[tokio::test]
    async fn test_blackboard_concurrent_reads() {
        let bb = Blackboard::new();
        bb.set("X", BlackboardValue::Int(42));

        let mut handles = vec![];
        for _ in 0..100 {
            let bb_clone = bb.clone();
            handles.push(tokio::spawn(async move {
                bb_clone.get("X")
            }));
        }

        for handle in handles {
            let res = handle.await.unwrap();
            assert_eq!(res, Some(BlackboardValue::Int(42)), "Concurrent reads must return the correct value");
        }
    }

    #[tokio::test]
    async fn test_blackboard_read_write_consistency() {
        let bb = Blackboard::new();
        bb.set("Y", BlackboardValue::Bool(false));

        let bb_write = bb.clone();
        let write_handle = tokio::spawn(async move {
            bb_write.set("Y", BlackboardValue::Bool(true));
        });

        let bb_read = bb.clone();
        let read_handle = tokio::spawn(async move {
            bb_read.get("Y")
        });

        write_handle.await.unwrap();
        let read_res = read_handle.await.unwrap();
        
        assert!(read_res == Some(BlackboardValue::Bool(false)) || read_res == Some(BlackboardValue::Bool(true)), "Read during write must not panic or tear");
        
        // After write is done, it MUST be true.
        assert_eq!(bb.get("Y"), Some(BlackboardValue::Bool(true)), "Final value must be the written one");
    }

    // --- Scoped Blackboard Tests (Story PR-1216) ---

    #[tokio::test]
    async fn test_scoped_bb_isolates_unmapped_keys() {
        let parent = Blackboard::new();
        let scoped = ScopedBlackboard::new(parent.clone(), std::collections::HashMap::new());
        
        scoped.set("local_only", BlackboardValue::Int(1));
        parent.set("parent_only", BlackboardValue::Int(2));
        
        assert_eq!(scoped.get("local_only"), Some(BlackboardValue::Int(1)));
        assert_eq!(parent.get("local_only"), None, "Parent must not see isolated local keys");
        
        assert_eq!(scoped.get("parent_only"), None, "Scoped bb must not implicitly see parent keys unless mapped");
    }

    #[tokio::test]
    async fn test_scoped_bb_maps_keys_to_parent() {
        let parent = Blackboard::new();
        let mut mapping = std::collections::HashMap::new();
        mapping.insert("local_in".to_string(), "global_out".to_string());
        
        let scoped = ScopedBlackboard::new(parent.clone(), mapping);
        
        // Write to mapped local key
        scoped.set("local_in", BlackboardValue::Text("Hello".to_string()));
        
        // Read from parent global key
        assert_eq!(parent.get("global_out"), Some(BlackboardValue::Text("Hello".to_string())), "Write to mapped local key must reflect in parent global key");
        
        // Write to parent global key
        parent.set("global_out", BlackboardValue::Text("World".to_string()));
        
        // Read from mapped local key
        assert_eq!(scoped.get("local_in"), Some(BlackboardValue::Text("World".to_string())), "Read from mapped local key must fetch from parent global key");
    }

    // --- Pub/Sub Bridge Tests (Story PR-1217) ---

    #[tokio::test]
    async fn test_pubsub_bridge_updates_blackboard() {
        let bb = Blackboard::new();
        let bridge = PubSubBridge::new(bb.clone());
        
        let tx = bridge.subscribe("sensor/lidar", "lidar_distance");
        
        // Simulate an incoming pub/sub message
        tx.send(BlackboardValue::Float(2.5)).await.unwrap();
        
        // Yield execution to allow background task to process the message
        tokio::task::yield_now().await;
        
        assert_eq!(bb.get("lidar_distance"), Some(BlackboardValue::Float(2.5)), "Pub/Sub bridge must automatically update the blackboard key");
    }
}
