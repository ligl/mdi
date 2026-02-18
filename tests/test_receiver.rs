use mdi::receiver::TickReceiver;

#[test]
fn test_parse_tick() {
    let receiver = TickReceiver::new("BTCUSDT".to_string(), 1000);

    let json = r#"{
        "E": 1234567890,
        "T": 1234567890,
        "p": "100.50",
        "q": "1.5",
        "m": true,
        "t": 12345
    }"#;

    let tick = receiver.parse_tick(json).unwrap();
    assert_eq!(tick.symbol, "BTCUSDT");
    assert_eq!(tick.price, 100.50);
    assert_eq!(tick.quantity, 1.5);
    assert_eq!(tick.is_buyer_maker, true);
}
