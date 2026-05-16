use std::sync::Arc;
use tokio::sync::broadcast;
use crate::network::NetworkBackend;
use crate::BlackboardValue;

pub struct ZenohBackend {
    session: Arc<zenoh::Session>,
}

impl ZenohBackend {
    pub async fn new() -> Self {
        let config = zenoh::Config::default();
        let session = Arc::new(zenoh::open(config).await.unwrap());
        Self { session }
    }
}

impl NetworkBackend for ZenohBackend {
    fn publish(&self, topic: &str, payload: BlackboardValue) -> Result<(), ()> {
        let session = self.session.clone();
        let topic = topic.to_string();
        
        // Convert payload to string
        let value_str = match payload {
            BlackboardValue::Int(i) => i.to_string(),
            BlackboardValue::Float(f) => f.to_string(),
            BlackboardValue::Text(t) => t,
            BlackboardValue::Bool(b) => b.to_string(),
        };

        tokio::spawn(async move {
            let _ = session.put(&topic, value_str).await;
        });

        Ok(())
    }

    fn subscribe(&self, topic: &str) -> broadcast::Receiver<BlackboardValue> {
        let (tx, rx) = broadcast::channel(100);
        let session = self.session.clone();
        let topic = topic.to_string();

        tokio::spawn(async move {
            if let Ok(subscriber) = session.declare_subscriber(&topic).await {
                while let Ok(sample) = subscriber.recv_async().await {
                    let value_str = match sample.payload().try_to_string() {
                        Ok(s) => s.into_owned(),
                        Err(_) => continue,
                    };

                    let bb_val = if let Ok(i) = value_str.parse::<i32>() {
                        BlackboardValue::Int(i)
                    } else if let Ok(f) = value_str.parse::<f64>() {
                        BlackboardValue::Float(f)
                    } else if let Ok(b) = value_str.parse::<bool>() {
                        BlackboardValue::Bool(b)
                    } else {
                        BlackboardValue::Text(value_str)
                    };

                    let _ = tx.send(bb_val);
                }
            }
        });

        rx
    }
}
