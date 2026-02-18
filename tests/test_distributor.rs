use mdi::KLine;
use mdi::distributor::Distributor;

#[tokio::test]
async fn test_distributor_broadcast() {
    let distributor = Distributor::new(100);

    // 创建两个订阅者
    let mut rx1 = distributor.subscribe("BTCUSDT", 60);
    let mut rx2 = distributor.subscribe("BTCUSDT", 60);

    assert_eq!(distributor.subscriber_count("BTCUSDT", 60), 2);

    // 发送 K 线
    let kline = KLine::new("BTCUSDT".to_string(), 1000, 60, 100.0);
    let receiver_count = distributor.broadcast_kline(kline.clone(), false);
    assert_eq!(receiver_count, 2);

    // 验证两个订阅者都收到了消息
    let msg1 = rx1.recv().await.unwrap();
    let msg2 = rx2.recv().await.unwrap();

    assert_eq!(msg1.kline.symbol, "BTCUSDT");
    assert_eq!(msg2.kline.symbol, "BTCUSDT");
    assert!(!msg1.is_closed);
    assert!(!msg2.is_closed);
}

#[tokio::test]
async fn test_distributor_multiple_symbols() {
    let distributor = Distributor::new(100);

    let mut rx1 = distributor.subscribe("BTCUSDT", 60);
    let mut rx2 = distributor.subscribe("ETHUSDT", 60);

    assert_eq!(distributor.subscriber_count("BTCUSDT", 60), 1);
    assert_eq!(distributor.subscriber_count("ETHUSDT", 60), 1);

    let kline1 = KLine::new("BTCUSDT".to_string(), 1000, 60, 100.0);
    let kline2 = KLine::new("ETHUSDT".to_string(), 1000, 60, 50.0);

    distributor.broadcast_kline(kline1, false);
    distributor.broadcast_kline(kline2, false);

    let msg1 = rx1.recv().await.unwrap();
    let msg2 = rx2.recv().await.unwrap();

    assert_eq!(msg1.kline.symbol, "BTCUSDT");
    assert_eq!(msg2.kline.symbol, "ETHUSDT");
}
