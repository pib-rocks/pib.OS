use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use pib_os::{Blackboard, ScopedBlackboard, BlackboardValue, AsyncActionNode, NodeStatus, Sequence};

struct SuccessNode;

impl AsyncActionNode for SuccessNode {
    fn tick(&self) -> Pin<Box<dyn Future<Output = NodeStatus> + Send + '_>> {
        Box::pin(async { NodeStatus::Success })
    }
}

fn bench_blackboard(c: &mut Criterion) {
    let mut group = c.benchmark_group("blackboard");
    
    group.bench_function("scoped_read_write", |b| {
        let parent = Blackboard::new();
        let mut mapping = HashMap::new();
        mapping.insert("local_key".to_string(), "parent_key".to_string());
        let scoped = ScopedBlackboard::new(parent, mapping);
        
        b.iter(|| {
            scoped.set("local_key", BlackboardValue::Int(black_box(42)));
            let val = scoped.get("local_key");
            black_box(val);
        });
    });
    
    group.finish();
}

fn bench_tree_tick(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("tree");
    
    group.bench_function("sequence_tick_100_children", |b| {
        // We have to recreate the sequence because tick() might need mutable state for Sequence? 
        // Wait, Sequence::tick(&self) doesn't need mutable state, it uses AtomicUsize.
        let mut children: Vec<Box<dyn AsyncActionNode>> = Vec::new();
        for _ in 0..100 {
            children.push(Box::new(SuccessNode));
        }
        let sequence = Sequence::new(children);
        
        b.to_async(&rt).iter(|| async {
            // Need to reset the sequence current_child to 0 if it stores state
            // Let's see if Sequence stores state or if we can recreate it, or just use it. 
            // Wait, Sequence stores current_child as AtomicUsize! 
            // If it succeeds, it might keep it at 100 or reset it. Let's see.
            let status = sequence.tick().await;
            black_box(status);
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_blackboard, bench_tree_tick);
criterion_main!(benches);