/// 本地性能基准测试
/// 运行方式：cargo run --example perf_test --release

use mdi::{Tick, KLineBuilder, RingBuffer, TickStorage};
use std::time::Instant;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          MDI 系统性能基准测试 (Performance Benchmarks)        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // ========== 1. RingBuffer 性能测试 ==========
    println!("┌─ RingBuffer 性能 ─────────────────────────────────────────────┐");
    bench_ring_buffer_push();
    bench_ring_buffer_pop();
    bench_ring_buffer_pop_batch();
    println!("└───────────────────────────────────────────────────────────────┘\n");

    // ========== 2. KLineBuilder 性能测试 ==========
    println!("┌─ KLineBuilder 性能 ───────────────────────────────────────────┐");
    bench_kline_builder();
    println!("└───────────────────────────────────────────────────────────────┘\n");

    // ========== 3. RocksDB Storage 性能测试 ==========
    println!("┌─ RocksDB Storage 性能 ────────────────────────────────────────┐");
    bench_storage()?;
    println!("└───────────────────────────────────────────────────────────────┘\n");

    // ========== 4. 完整管道性能测试 ==========
    println!("┌─ 完整管道性能 ────────────────────────────────────────────────┐");
    bench_full_pipeline()?;
    println!("└───────────────────────────────────────────────────────────────┘\n");

    // ========== 5. 并发测试 ==========
    println!("┌─ 并发性能测试 ────────────────────────────────────────────────┐");
    bench_concurrent()?;
    println!("└───────────────────────────────────────────────────────────────┘\n");

    println!("✓ 所有基准测试完成！\n");

    Ok(())
}

fn bench_ring_buffer_push() {
    let buffer = RingBuffer::new(1000000);
    let num_ops = 10_000_000u64;

    let start = Instant::now();
    for i in 0..num_ops {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + i,
            1000000 + i,
            100.0 + (i as f64 % 10.0 - 5.0) * 0.1,
            1.0,
            true,
            i,
        );
        let _ = buffer.push(tick);
    }
    let elapsed = start.elapsed();

    let ops_per_sec = num_ops as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / num_ops as f64;

    println!(
        "  RingBuffer::push()");
    println!("    Operations: {}", format_number(num_ops));
    println!("    Time:       {:.3}s", elapsed.as_secs_f64());
    println!("    Throughput: {:.2} ops/sec", format_number_f(ops_per_sec));
    println!("    Per-op:     {:.2} ns (latency)", ns_per_op);
}

fn bench_ring_buffer_pop() {
    let buffer = RingBuffer::new(1000000);
    let num_ops = 1_000_000u64;

    // 先填充
    for i in 0..num_ops {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + i,
            1000000 + i,
            100.0,
            1.0,
            true,
            i,
        );
        let _ = buffer.push(tick);
    }

    let start = Instant::now();
    let mut pop_count = 0u64;
    while let Some(_) = buffer.pop() {
        pop_count += 1;
    }
    let elapsed = start.elapsed();

    let ops_per_sec = pop_count as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / pop_count as f64;

    println!(
        "  RingBuffer::pop()");
    println!("    Operations: {}", format_number(pop_count));
    println!("    Time:       {:.3}s", elapsed.as_secs_f64());
    println!("    Throughput: {:.2} ops/sec", format_number_f(ops_per_sec));
    println!("    Per-op:     {:.2} ns (latency)", ns_per_op);
}

fn bench_ring_buffer_pop_batch() {
    let buffer = RingBuffer::new(1000000);
    let num_batches = 100_000u64;
    let batch_size = 100usize;

    // 先填充
    for i in 0..(num_batches * batch_size as u64) {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + i,
            1000000 + i,
            100.0,
            1.0,
            true,
            i,
        );
        let _ = buffer.push(tick);
    }

    let start = Instant::now();
    let mut total_popped = 0;
    for _ in 0..num_batches {
        let batch = buffer.pop_batch(batch_size);
        total_popped += batch.len();
    }
    let elapsed = start.elapsed();

    let ops_per_sec = total_popped as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / total_popped as f64;

    println!(
        "  RingBuffer::pop_batch({})", batch_size);
    println!("    Time:        {:.3}s", elapsed.as_secs_f64());
    println!("    Throughput:  {:.2} items/sec", format_number_f(ops_per_sec));
    println!("    Per-item:    {:.2} ns (latency)", ns_per_op);
}

fn bench_kline_builder() {
    let builder = KLineBuilder::standard();
    let num_ticks = 10_000_000u64;

    println!("  KLineBuilder::process_tick()");
    
    let start = Instant::now();
    for i in 0..num_ticks {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + (i * 100),
            1000000 + (i * 100),
            100.0 + (i as f64 % 10.0 - 5.0) * 0.1,
            (1.0 + (i as f64 % 9.0) * 0.01),
            i % 2 == 0,
            i,
        );
        builder.process_tick(&tick);
    }
    let elapsed = start.elapsed();

    let ops_per_sec = num_ticks as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / num_ticks as f64;

    println!("    Operations: {}", format_number(num_ticks));
    println!("    Time:       {:.3}s", elapsed.as_secs_f64());
    println!("    Throughput: {:.2} ticks/sec", format_number_f(ops_per_sec));
    println!("    Per-op:     {:.2} ns (latency)", ns_per_op);

    let stats = builder.get_stats();
    println!("    K-Lines:    {} (6 intervals)", stats.total_klines);
}

fn bench_storage() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let storage = TickStorage::open(temp_dir.path().join("bench.db"))?;

    println!("  TickStorage::write_tick() (单条写入)");
    let num_ops = 100_000u64;
    let start = Instant::now();

    for i in 0..num_ops {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + i,
            1000000 + i,
            100.0,
            1.0,
            true,
            i,
        );
        storage.write_tick(&tick)?;
    }

    let elapsed = start.elapsed();
    let ops_per_sec = num_ops as f64 / elapsed.as_secs_f64();
    let us_per_op = (elapsed.as_secs_f64() * 1_000_000.0) / num_ops as f64;

    println!("    Operations: {}", format_number(num_ops));
    println!("    Time:       {:.3}s", elapsed.as_secs_f64());
    println!("    Throughput: {:.2} ops/sec", format_number_f(ops_per_sec));
    println!("    Per-op:     {:.2} µs (latency)", us_per_op);

    // 批量写入测试
    let temp_dir2 = TempDir::new()?;
    let storage2 = TickStorage::open(temp_dir2.path().join("bench2.db"))?;

    println!("\n  TickStorage::write_ticks() (批量写入)");
    let num_batches = 1000u64;
    let batch_size = 100usize;

    let start = Instant::now();
    for batch_id in 0..num_batches {
        let mut ticks = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            let tick = Tick::new(
                "BTCUSDT".to_string(),
                1000000 + (batch_id * batch_size as u64) as u64 + i as u64,
                1000000 + (batch_id * batch_size as u64) as u64 + i as u64,
                100.0,
                1.0,
                true,
                (batch_id * batch_size as u64) as u64 + i as u64,
            );
            ticks.push(tick);
        }
        storage2.write_ticks(&ticks)?;
    }

    let elapsed = start.elapsed();
    let total_items = num_batches * batch_size as u64;
    let ops_per_sec = total_items as f64 / elapsed.as_secs_f64();
    let us_per_op = (elapsed.as_secs_f64() * 1_000_000.0) / total_items as f64;

    println!("    Total items: {}", format_number(total_items));
    println!("    Time:        {:.3}s", elapsed.as_secs_f64());
    println!("    Throughput:  {:.2} ops/sec", format_number_f(ops_per_sec));
    println!("    Per-item:    {:.2} µs (latency)", us_per_op);

    Ok(())
}

fn bench_full_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let buffer = RingBuffer::new(1000000);
    let builder = KLineBuilder::standard();
    let num_ticks = 1_000_000u64;

    println!("  完整管道：RingBuffer → KLineBuilder");

    let start = Instant::now();
    for i in 0..num_ticks {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + (i * 100),
            1000000 + (i * 100),
            100.0 + (i as f64 % 10.0 - 5.0) * 0.1,
            (1.0 + (i as f64 % 9.0) * 0.01),
            i % 2 == 0,
            i,
        );

        // 推送到 buffer
        buffer.push(tick.clone())?;

        // 处理 K线
        builder.process_tick(&tick);

        // 偶尔从 buffer 中弹出
        if i % 100 == 0 {
            while let Some(_) = buffer.pop() {}
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = num_ticks as f64 / elapsed.as_secs_f64();
    let ns_per_op = (elapsed.as_nanos() as f64) / num_ticks as f64;

    println!("    Ticks:      {}", format_number(num_ticks));
    println!("    Time:       {:.3}s", elapsed.as_secs_f64());
    println!("    Throughput: {:.2} ticks/sec", format_number_f(ops_per_sec));
    println!("    Per-tick:   {:.2} ns", ns_per_op);

    let stats = builder.get_stats();
    println!("    K-Lines:    {} total", stats.total_klines);
    println!("    Buffer:     {}/{}", buffer.len(), buffer.capacity());

    Ok(())
}

fn bench_concurrent() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    use std::thread;

    let buffer = Arc::new(RingBuffer::new(1000000));
    let num_threads = 8;
    let ops_per_thread = 1_000_000u64;

    println!("  并发测试：{} 个线程，各 {} 操作", num_threads, format_number(ops_per_thread));

    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let buffer_clone = Arc::clone(&buffer);
        let handle = thread::spawn(move || {
            for i in 0..ops_per_thread {
                let tick = Tick::new(
                    "BTCUSDT".to_string(),
                    1000000 + (thread_id as u64 * ops_per_thread) + i,
                    1000000 + (thread_id as u64 * ops_per_thread) + i,
                    100.0,
                    1.0,
                    true,
                    (thread_id as u64 * ops_per_thread) + i,
                );
                let _ = buffer_clone.push(tick);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    let total_ops = (num_threads as u64) * ops_per_thread;
    let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();

    println!("    Total ops:  {}", format_number(total_ops));
    println!("    Time:       {:.3}s", elapsed.as_secs_f64());
    println!("    Throughput: {:.2} ops/sec", format_number_f(ops_per_sec));
    println!("    Buffer:     {} items", buffer.len());

    Ok(())
}

// ============ 工具函数 ============

fn format_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

fn format_number_f(n: f64) -> String {
    if n >= 1_000_000_000.0 {
        format!("{:.2}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("{:.2}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.2}K", n / 1_000.0)
    } else {
        format!("{:.2}", n)
    }
}
