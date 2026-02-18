use mdi::Tick;
use mdi::queue::RingBuffer;

#[test]
fn test_ring_buffer_basic() {
    let buffer = RingBuffer::new(1000);
    
    let tick = Tick::new(
        "BTCUSDT".to_string(),
        1000,
        1000,
        100.0,
        1.0,
        true,
        1,
    );
    
    assert!(buffer.push(tick.clone()).is_ok());
    assert_eq!(buffer.len(), 1);
    
    let popped = buffer.pop();
    assert!(popped.is_some());
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_ring_buffer_batch() {
    let buffer = RingBuffer::new(100);
    
    for i in 0..10 {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            i * 1000,
            i * 1000,
            100.0 + i as f64,
            1.0,
            true,
            i as u64,
        );
        buffer.push(tick).unwrap();
    }
    
    let batch = buffer.pop_batch(5);
    assert_eq!(batch.len(), 5);
    assert_eq!(buffer.len(), 5);
}

#[test]
fn test_ring_buffer_capacity() {
    let buffer = RingBuffer::new(2);
    
    let tick = Tick::new("BTCUSDT".to_string(), 1000, 1000, 100.0, 1.0, true, 1);
    
    assert!(buffer.push(tick.clone()).is_ok());
    assert!(buffer.push(tick.clone()).is_ok());
    assert!(buffer.push(tick).is_err()); // 超容量
}
