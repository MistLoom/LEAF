use super::*;
use ability_support::HostEvent;
use futures_util::poll;
use std::pin::pin;
use tokio::sync::mpsc;

struct TestListener(mpsc::UnboundedReceiver<BusEvent>);

#[async_trait::async_trait]
impl EventListener for TestListener {
    async fn next(&mut self) -> Option<BusEvent> {
        self.0.recv().await
    }
}

fn host_event(sequence: u64) -> BusEvent {
    BusEvent::Host(HostEvent {
        name: host_event::INVOKE_AFTER.into(),
        at: 0,
        discovery: None,
        detail: json!({"sequence": sequence}),
    })
}

fn ability_event(sequence: u64) -> BusEvent {
    BusEvent::Ability(AbilityEvent {
        instance: InstanceId::new("turn").unwrap(),
        name: "started".into(),
        at: 0,
        payload: json!({"session_id": "test-session", "turn_id": sequence}),
    })
}

#[tokio::test]
async fn host_events_survive_late_subscription_and_reconnection() {
    let (events, listener) = mpsc::unbounded_channel();
    let (tx, initial_receiver) = broadcast::channel(16);
    drop(initial_receiver);
    let mut forwarding = pin!(forward_host_events(TestListener(listener), tx.clone()));

    // Polling to Pending ensures each queued event was processed before changing subscribers.
    events.send(host_event(1)).unwrap();
    assert!(poll!(&mut forwarding).is_pending());

    for sequence in [2, 4] {
        let mut client = tx.subscribe();
        events.send(host_event(sequence)).unwrap();
        assert!(poll!(&mut forwarding).is_pending());
        let line: Value = serde_json::from_str(&client.try_recv().unwrap()).unwrap();
        assert_eq!(line["channel"], "host");
        assert_eq!(line["name"], host_event::INVOKE_AFTER);
        assert_eq!(line["detail"]["sequence"], sequence);

        drop(client);
        events.send(host_event(sequence + 1)).unwrap();
        assert!(poll!(&mut forwarding).is_pending());
    }

    drop(events);
    assert!(poll!(&mut forwarding).is_ready());
}

struct UserSink(mpsc::UnboundedSender<UserEvent>);

#[async_trait::async_trait]
impl UserEventSink for UserSink {
    async fn on_user_event(&self, event: &UserEvent) {
        self.0.send(event.clone()).unwrap();
    }
}

#[tokio::test]
async fn ability_events_keep_translating_without_clients_and_resume_after_reconnection() {
    let host = AbilityHost::new();
    let (user_tx, mut user_events) = mpsc::unbounded_channel();
    host.events()
        .subscribe_user(Arc::new(UserSink(user_tx)))
        .await;
    let (events, listener) = mpsc::unbounded_channel();
    let (tx, initial_receiver) = broadcast::channel(16);
    drop(initial_receiver);
    let mut forwarding = pin!(forward_ability_events(
        TestListener(listener),
        host,
        tx.clone()
    ));

    for sequence in 1..=4 {
        // Start disconnected, then connect, disconnect, and reconnect.
        let mut client = (sequence % 2 == 0).then(|| tx.subscribe());
        events.send(ability_event(sequence)).unwrap();
        assert!(poll!(&mut forwarding).is_pending());

        let translated = user_events.try_recv().unwrap();
        assert_eq!(translated.name, "turn.started");
        assert_eq!(translated.payload["turn_id"], sequence);

        if let Some(client) = client.as_mut() {
            let line: Value = serde_json::from_str(&client.try_recv().unwrap()).unwrap();
            assert_eq!(line["channel"], "ability");
            assert_eq!(line["instance"], "turn");
            assert_eq!(line["name"], "started");
            assert_eq!(line["detail"]["turn_id"], sequence);
        }
    }

    drop(events);
    assert!(poll!(&mut forwarding).is_ready());
}
