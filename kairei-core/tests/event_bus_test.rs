use kairei_core::event_bus::{Event, EventBus};
use kairei_core::event_registry::EventType;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_concurrent_subscribers() {
    let bus = EventBus::new(16);
    let received_count = Arc::new(AtomicUsize::new(0));
    let subscriber_count = 5;
    let event_count = 10;

    // 複数のサブスクライバーを起動
    let mut handles = vec![];
    for i in 0..subscriber_count {
        let (mut event_rx, _) = bus.subscribe();
        let received_count = received_count.clone();

        let handle = tokio::spawn(async move {
            // 各サブスクライバーは異なる処理時間を持つ
            let process_time = Duration::from_millis((i + 1) * 100);

            while let Ok(_event) = event_rx.recv().await {
                // 重い処理をシミュレート
                sleep(process_time).await;
                received_count.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    // イベントを連続して送信
    for i in 0..event_count {
        let event = Event {
            event_type: EventType::Custom(format!("test_{}", i)),
            ..Default::default()
        };
        bus.publish(event).await.unwrap();
    }

    // 少し待機して処理を確認
    sleep(Duration::from_secs(8)).await;

    // 期待される総受信数: subscriber_count * event_count
    assert_eq!(
        u64::try_from(received_count.load(Ordering::SeqCst)).unwrap(),
        subscriber_count * event_count
    );
}

#[tokio::test]
async fn test_slow_subscriber_doesnt_block_others() {
    let bus = EventBus::new(16);
    let fast_received = Arc::new(AtomicUsize::new(0));
    let slow_received = Arc::new(AtomicUsize::new(0));

    // 遅いサブスクライバー
    let (mut slow_rx, _) = bus.subscribe();
    let slow_count = slow_received.clone();
    tokio::spawn(async move {
        while slow_rx.recv().await.is_ok() {
            // 重い処理をシミュレート
            sleep(Duration::from_millis(500)).await;
            slow_count.fetch_add(1, Ordering::SeqCst);
        }
    });

    // 速いサブスクライバー
    let (mut fast_rx, _) = bus.subscribe();
    let fast_count = fast_received.clone();
    tokio::spawn(async move {
        while fast_rx.recv().await.is_ok() {
            // 軽い処理
            sleep(Duration::from_millis(10)).await;
            fast_count.fetch_add(1, Ordering::SeqCst);
        }
    });

    // イベントを連続して送信
    for i in 0..5 {
        let event = Event {
            event_type: EventType::Custom(format!("test_{}", i)),
            ..Default::default()
        };
        bus.publish(event).await.unwrap();
        sleep(Duration::from_millis(100)).await;
    }

    // 少し待機
    sleep(Duration::from_secs(1)).await;

    // 速いサブスクライバーは全てのイベントを処理済み
    assert_eq!(fast_received.load(Ordering::SeqCst), 5);
    // 遅いサブスクライバーも処理は進んでいる
    assert!(slow_received.load(Ordering::SeqCst) > 0);
}

#[tokio::test]
async fn test_broadcast_overflow_behavior() {
    let bus = EventBus::new(4); // 小さめのバッファ
    let (mut rx1, _) = bus.subscribe();

    // バッファより多めのイベントを高速で送信
    for i in 0..20 {
        let event = Event {
            event_type: EventType::Custom(format!("test_{}", i)),
            ..Default::default()
        };
        bus.publish(event).await.unwrap();
    }

    let mut received = vec![];
    while let Ok(event) = rx1.recv().await {
        received.push(event.event_type.clone());
    }

    assert_eq!(received.len(), 0); // バッファオーバーフロー時は受信できない
}
