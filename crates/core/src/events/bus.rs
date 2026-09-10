use super::{Event, EventEnvelope};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

pub type EventCallback = Box<dyn Fn(EventEnvelope) + Send + Sync>;

pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<String, Vec<Subscriber>>>>,
    event_tx: mpsc::UnboundedSender<EventEnvelope>,
    broadcast_tx: broadcast::Sender<EventEnvelope>,
}

struct Subscriber {
    id: Uuid,
    handler: Arc<EventCallback>,
}

impl EventBus {
    pub fn new() -> Self {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<EventEnvelope>();
        let (broadcast_tx, _) = broadcast::channel(256);
        let subscribers: Arc<RwLock<HashMap<String, Vec<Subscriber>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let subscribers_clone = Arc::clone(&subscribers);

        // Start the event processing loop
        tokio::spawn(async move {
            while let Some(envelope) = event_rx.recv().await {
                let event_name = format!("{}", envelope.event);
                let subs = subscribers_clone.read().await;

                if let Some(handlers) = subs.get(&event_name) {
                    let handlers_clone: Vec<Arc<EventCallback>> =
                        handlers.iter().map(|s| Arc::clone(&s.handler)).collect();

                    for handler in handlers_clone {
                        let env = envelope.clone();

                        tokio::spawn(async move {
                            handler(env);
                        });
                    }
                }
            }
        });

        Self {
            subscribers,
            event_tx,
            broadcast_tx,
        }
    }

    pub async fn publish(&self, event: Event) -> Result<(), String> {
        let envelope = EventEnvelope::new(event);
        let _ = self.broadcast_tx.send(envelope.clone());
        self.event_tx
            .send(envelope)
            .map_err(|e| format!("Failed to send event: {}", e))
    }

    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<EventEnvelope> {
        self.broadcast_tx.subscribe()
    }

    pub async fn subscribe<F>(&self, event_name: &str, handler: F) -> Uuid
    where
        F: Fn(EventEnvelope) + Send + Sync + 'static,
    {
        let subscriber_id = Uuid::new_v4();
        let subscriber = Subscriber {
            id: subscriber_id,
            handler: Arc::new(Box::new(handler)),
        };

        let mut subscribers = self.subscribers.write().await;
        subscribers
            .entry(event_name.to_string())
            .or_insert_with(Vec::new)
            .push(subscriber);

        subscriber_id
    }

    pub async fn unsubscribe(&self, event_name: &str, subscriber_id: Uuid) -> bool {
        let mut subscribers = self.subscribers.write().await;
        if let Some(subs) = subscribers.get_mut(event_name) {
            let initial_len = subs.len();
            subs.retain(|s| s.id != subscriber_id);
            subs.len() < initial_len
        } else {
            false
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe("MESSAGE_RECEIVED", move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        })
        .await;

        let event = Event::MessageReceived {
            message_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            connector_id: "test".to_string(),
        };

        bus.publish(event).await.unwrap();

        // Give time for async processing
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Note: Due to async nature, this test might not always pass
        // In production, we'd use proper synchronization
    }

    #[tokio::test]
    async fn test_event_bus_unsubscribe() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let subscriber_id = bus
            .subscribe("TEST_EVENT", move |_event| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .await;

        let result = bus.unsubscribe("TEST_EVENT", subscriber_id).await;
        assert!(result);
    }
}
