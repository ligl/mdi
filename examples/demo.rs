/// 演示示例：本地数据流模拟
/// 
/// 这个示例演示系统的核心功能，无需连接真实的 Binance WebSocket
/// 包括：
/// - K线合并
/// - Lock-free 分发
/// - RocksDB 存储
/// - CPU Affinity 线程绑定

use mdi::{
    Tick, KLineBuilder, Distributor, TickStorage, RingBuffer,
    CpuAffinity, ThreadBuilder,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MDI Market Data System Demo ===\n");

    // 1. CPU Affinity 演示
    println!("CPU Configuration:");
    println!("  Total CPUs: {}", CpuAffinity::num_cpus());
    println!("  Physical CPUs: {}", CpuAffinity::num_physical_cpus());
    println!();

    // 2. 初始化核心组件
    println!("Initializing components...");
    let kline_builder = Arc::new(KLineBuilder::standard());
    let distributor = Arc::new(Distributor::new(1000));
    let storage = Arc::new(TickStorage::open("./data/demo.db")?);
    let ring_buffer = Arc::new(RingBuffer::new(10000));

    // 3. 生成模拟数据
    println!("Generating simulated tick data...\n");
    let num_ticks = 1000;
    let mut ticks = Vec::new();

    let base_price = 100.0;
    for i in 0..num_ticks {
        let price = base_price + (i as f64 % 10.0 - 5.0) * 0.1;
        let quantity = 0.1 + (i as f64 % 9.0) * 0.01;
        let timestamp = 1000000 + (i as u64 * 100); // 每个 tick 间隔 100ms

        let tick = Tick::new(
            "BTCUSDT".to_string(),
            timestamp,
            timestamp,
            price,
            quantity,
            i % 2 == 0,
            i as u64,
        );
        ticks.push(tick);
    }

    // 4. 处理数据：推送到 RingBuffer，然后构建 K 线
    println!("Processing ticks (building klines)...");
    let start = Instant::now();
    let mut stored_ticks = Vec::new();

    for tick in &ticks {
        // 推送到 ring buffer
        let _ = ring_buffer.push(tick.clone());

        // 构建 K 线
        let klines = kline_builder.process_tick(tick);

        // 分发 K 线
        for kline in klines {
            let _ = distributor.broadcast_kline(kline, false);
        }

        stored_ticks.push(tick.clone());

        // 每 500 条 tick 存储一次
        if stored_ticks.len() >= 500 {
            storage.write_ticks(&stored_ticks)?;
            stored_ticks.clear();
        }
    }

    // 存储剩余数据
    if !stored_ticks.is_empty() {
        storage.write_ticks(&stored_ticks)?;
    }

    let elapsed = start.elapsed();
    println!(
        "✓ Processed {} ticks in {:.3}s ({:.0} ticks/sec)\n",
        num_ticks,
        elapsed.as_secs_f64(),
        num_ticks as f64 / elapsed.as_secs_f64()
    );

    // 5. 显示 K 线统计
    let stats = kline_builder.get_stats();
    println!("K-Line Statistics:");
    println!("  Total Symbols: {}", stats.total_symbols);
    println!("  Total K-Lines: {}", stats.total_klines);
    println!("  Intervals: {:?} seconds", stats.intervals);
    println!();

    // 6. 读取和显示 K 线数据
    println!("Latest K-Lines for BTCUSDT:");
    for interval in &[60, 300, 900, 3600] {
        if let Some(kline) = kline_builder.get_latest_kline("BTCUSDT", *interval) {
            println!(
                "  {}s: O={:.4} H={:.4} L={:.4} C={:.4} V={:.4} T={}",
                interval, kline.open, kline.high, kline.low, kline.close,
                kline.volume, kline.number_of_trades
            );
        }
    }
    println!();

    // 7. 显示存储数据
    let stored = storage.read_ticks_by_symbol("BTCUSDT", 5)?;
    println!("Last 5 stored ticks:");
    for tick in stored.iter().rev().take(5) {
        println!(
            "  ID={} Price={:.4} Qty={:.4} Time={}",
            tick.trade_id, tick.price, tick.quantity, tick.timestamp
        );
    }
    println!();

    // 8. RingBuffer 演示
    println!("RingBuffer Status:");
    println!("  Capacity: {}", ring_buffer.capacity());
    println!("  Current Size: {}", ring_buffer.len());
    println!("  Usage: {:.2}%", ring_buffer.usage_percent());
    println!();

    // 9. 分发器演示
    println!("Distributor Status:");
    for interval in &[60, 300, 900, 3600] {
        let subs = distributor.subscriber_count("BTCUSDT", *interval);
        println!("  BTCUSDT:{}s - Subscribers: {}", interval, subs);
    }
    println!();

    // 10. CPU Affinity 线程演示
    println!("Testing CPU Affinity with thread spawning...");
    let handles: Vec<_> = (0..4)
        .map(|i| {
            ThreadBuilder::new()
                .cpu(i % CpuAffinity::num_cpus())
                .name(format!("worker-{}", i))
                .spawn(move || {
                    // 计算一些东西，演示线程绑定工作
                    let mut sum = 0u64;
                    for j in 0..1000000 {
                        sum = sum.wrapping_add(j);
                    }
                    println!("  Thread {} completed (CPU affinity bound)", i);
                    sum
                })
        })
        .collect();

    for handle in handles {
        let _ = handle.join();
    }
    println!();

    println!("=== Demo Completed Successfully ===");
    println!("Data saved to: ./data/demo.db");

    Ok(())
}
