use mdi::{Tick, KLine};

#[test]
fn test_tick_kline_key() {
    let tick = Tick::new("BTCUSDT".to_string(), 1000000, 1000000, 100.0, 1.0, true, 1);
    // (1000000 ms / 1000) / 60 * 60 = 1000 / 60 * 60 = 16 * 60 = 960
    assert_eq!(tick.kline_key_for_period(60), 960);
}

#[test]
fn test_kline_update() {
    let mut kline = KLine::new("BTCUSDT".to_string(), 0, 60, 100.0);
    let tick1 = Tick::new("BTCUSDT".to_string(), 0, 0, 105.0, 10.0, true, 1);
    let tick2 = Tick::new("BTCUSDT".to_string(), 500, 500, 103.0, 5.0, false, 2);

    kline.update(&tick1);
    kline.update(&tick2);

    assert_eq!(kline.high, 105.0);
    assert_eq!(kline.low, 100.0);
    assert_eq!(kline.close, 103.0);
    assert_eq!(kline.volume, 15.0);
    assert_eq!(kline.number_of_trades, 2);
}
