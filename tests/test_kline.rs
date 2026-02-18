use mdi::KLine;
use mdi::kline::KLineBuilder;
use mdi::Tick;

#[test]
fn test_kline_builder() {
    let builder = KLineBuilder::new(vec![60, 300]);
    
    // 创建两个 tick，时间戳都在同一分钟内（都在开盘后的30秒内）
    // 使用足够大的基数，在同一K线周期内
    let tick1 = Tick::new("BTCUSDT".to_string(), 1000000, 1000000, 100.0, 1.0, true, 1);
    let tick2 = Tick::new("BTCUSDT".to_string(), 1000030, 1000030, 102.0, 2.0, true, 2);

    builder.process_tick(&tick1);
    builder.process_tick(&tick2);

    let kLine = builder.get_latest_kline("BTCUSDT", 60).unwrap();
    assert_eq!(kLine.close, 102.0);
    assert_eq!(kLine.volume, 3.0);  // 1.0 + 2.0
    assert_eq!(kLine.number_of_trades, 2);
}

#[test]
fn test_kline_stats() {
    let builder = KLineBuilder::standard();
    
    let tick = Tick::new("BTCUSDT".to_string(), 1000000, 1000000, 100.0, 1.0, true, 1);
    builder.process_tick(&tick);

    let stats = builder.get_stats();
    assert_eq!(stats.total_symbols, 1);
    assert_eq!(stats.total_klines, 6); // 6 个周期
}
