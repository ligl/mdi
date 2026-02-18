use mdi::{Tick, KLine};
use mdi::storage::TickStorage;
use tempfile::TempDir;

#[test]
fn test_write_and_read_tick() {
    let temp_dir = TempDir::new().unwrap();
    let storage = TickStorage::open(temp_dir.path().join("test.db")).unwrap();

    let tick = Tick::new(
        "BTCUSDT".to_string(),
        1000000,
        1000000,
        100.0,
        1.0,
        true,
        1,
    );

    storage.write_tick(&tick).unwrap();
    let read_tick = storage.read_tick("BTCUSDT", 1).unwrap();
    assert!(read_tick.is_some());
    assert_eq!(read_tick.unwrap().price, 100.0);
}

#[test]
fn test_write_and_read_kline() {
    let temp_dir = TempDir::new().unwrap();
    let storage = TickStorage::open(temp_dir.path().join("test.db")).unwrap();

    let kline = KLine::new("BTCUSDT".to_string(), 1000, 60, 100.0);
    storage.write_kline(&kline).unwrap();

    let read_kline = storage.read_kline("BTCUSDT", 60, 1000).unwrap();
    assert!(read_kline.is_some());
    assert_eq!(read_kline.unwrap().open, 100.0);
}
