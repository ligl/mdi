/// 简洁性能测试 - 无需等待编译
/// cargo run --example quick_bench --release

use mdi::{Tick, KLineBuilder, RingBuffer};
use std::time::Instant;

fn main() {
    println!("\n╔════════════════════════════════════════════╗");
    println!("║   MDI 快速性能测试 (Quick Benchmark)      ║");
    println!("╚════════════════════════════════════════════╝\n");

    // 1. RingBuffer 测试
    println!("1️⃣  RingBuffer 性能");
    println!("   ├─ push 操作...");
    bench_push();
    println!("   ├─ pop 操作...");
    bench_pop();
    println!("   └─ pop_batch 操作...");
    bench_pop_batch();
    println!();

    // 2. KLineBuilder 测试
    println!("2️⃣  KLineBuilder 性能");
    println!("   └─ process_tick...");
    bench_kline();
    println!();

    // 3. 完整流程
    println!("3️⃣  完整处理流程");
    println!("   └─ buffer + kline...");
    bench_pipeline();
    println!();

    println!("✅ 基准测试完成！\n");
}

fn bench_push() {
    let buffer = RingBuffer::new(10_000_000);
    let num_iterations = 100_000;

    let start = Instant::now();
    for i in 0..num_iterations {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + i,
            1000000 + i,
            100.0 + (i as f64 % 10.0 - 5.0) * 0.01,
            1.0,
            true,
            i as u64,
        );
        let _ = buffer.push(tick);
    }
    let elapsed = start.elapsed();

    let tps = num_iterations as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / num_iterations as f64;

    println!("      📊 {} 次 push", num_iterations);
    println!("         耗时: {:.3}s", elapsed.as_secs_f64());
    println!("         吞吐: {:.2} ops/s", tps);
    println!("         延迟: {:.1} ns/op", ns_per_op);
}

fn bench_pop() {
    let buffer = RingBuffer::new(200_000);
    let num_items = 100_000;

    // 填充
    for i in 0..num_items {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + i as u64,
            1000000 + i as u64,
            100.0,
            1.0,
            true,
            i as u64,
        );
        let _ = buffer.push(tick);
    }

    let start = Instant::now();
    let mut count = 0;
    while let Some(_) = buffer.pop() {
        count += 1;
    }
    let elapsed = start.elapsed();

    let tps = count as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / count as f64;

    println!("      📊 {} 次 pop", count);
    println!("         耗时: {:.3}s", elapsed.as_secs_f64());
    println!("         吞吐: {:.2} ops/s", tps);
    println!("         延迟: {:.1} ns/op", ns_per_op);
}

fn bench_pop_batch() {
    let buffer = RingBuffer::new(1_000_000);
    let num_items = 1_000_000;
    let batch_size = 100;

    // 填充
    for i in 0..num_items {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + i as u64,
            1000000 + i as u64,
            100.0,
            1.0,
            true,
            i as u64,
        );
        let _ = buffer.push(tick);
    }

    let start = Instant::now();
    let mut total_popped = 0;
    loop {
        let batch = buffer.pop_batch(batch_size);
        if batch.is_empty() {
            break;
        }
        total_popped += batch.len();
    }
    let elapsed = start.elapsed();

    let tps = total_popped as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / total_popped as f64;

    println!("      📊 {} 项 (批量 {})", total_popped, batch_size);
    println!("         耗时: {:.3}s", elapsed.as_secs_f64());
    println!("         吞吐: {:.2} ops/s", tps);
    println!("         延迟: {:.1} ns/op", ns_per_op);
}

fn bench_kline() {
    let builder = KLineBuilder::standard();
    let num_ticks = 1_000_000;

    let start = Instant::now();
    for i in 0..num_ticks {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + (i as u64 * 100),
            1000000 + (i as u64 * 100),
            100.0 + (i as f64 % 20.0 - 10.0) * 0.05,
            0.1 + (i as f64 % 9.0) * 0.01,
            i % 2 == 0,
            i as u64,
        );
        builder.process_tick(&tick);
    }
    let elapsed = start.elapsed();

    let tps = num_ticks as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / num_ticks as f64;
    let stats = builder.get_stats();

    println!("      📊 {} 个 ticks", num_ticks);
    println!("         耗时: {:.3}s", elapsed.as_secs_f64());
    println!("         吞吐: {:.2} ticks/s", tps);
    println!("         延迟: {:.1} ns/tick", ns_per_op);
    println!("         结果: {} 个 K线 (6个周期)", stats.total_klines);
}

fn bench_pipeline() {
    let buffer = RingBuffer::new(1_000_000);
    let builder = KLineBuilder::standard();
    let num_ticks = 1_000_000;

    let start = Instant::now();
    let mut pop_count = 0;

    for i in 0..num_ticks {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + (i as u64 * 100),
            1000000 + (i as u64 * 100),
            100.0 + (i as f64 % 20.0 - 10.0) * 0.05,
            0.1 + (i as f64 % 9.0) * 0.01,
            i % 2 == 0,
            i as u64,
        );

        // 推送到 buffer
        let _ = buffer.push(tick.clone());

        // 处理 K线
        builder.process_tick(&tick);

        // 每 1000 个 tick 弹出一次
        if i % 1000 == 0 {
            while let Some(_) = buffer.pop() {
                pop_count += 1;
            }
        }
    }

    // 清空缓冲区
    while let Some(_) = buffer.pop() {
        pop_count += 1;
    }

    let elapsed = start.elapsed();
    let tps = num_ticks as f64 / elapsed.as_secs_f64();
    let ns_per_tick = (elapsed.as_nanos() as f64) / num_ticks as f64;
    let stats = builder.get_stats();

    println!("      📊 {} 个 ticks", num_ticks);
    println!("         耗时: {:.3}s", elapsed.as_secs_f64());
    println!("         吞吐: {:.2} ticks/s", tps);
    println!("         延迟: {:.1} ns/tick", ns_per_tick);
    println!("         缓冲: 入 {} | 出 {}", num_ticks, pop_count);
    println!("         K线: {} 个 (6个周期)", stats.total_klines);
    println!("         缓冲区使用: {:.2}%", buffer.usage_percent());
}
