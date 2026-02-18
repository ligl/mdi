use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mdi::{
    Tick, KLineBuilder, RingBuffer, TickStorage,
    Distributor,
};
use std::sync::Arc;
use tempfile::TempDir;

// ============ RingBuffer 基准测试 ============

fn bench_ring_buffer_push(c: &mut Criterion) {
    let buffer = RingBuffer::new(100000);
    let tick = Tick::new(
        "BTCUSDT".to_string(),
        1000000,
        1000000,
        100.0,
        1.0,
        true,
        1,
    );

    c.bench_function("ringbuffer_push", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let _ = buffer.push(Tick::new(
                    "BTCUSDT".to_string(),
                    1000000 + i,
                    1000000 + i,
                    100.0 + (i as f64 * 0.01),
                    1.0,
                    true,
                    i as u64,
                ));
            }
            // 清空缓冲区
            while buffer.pop().is_some() {}
        });
    });
}

fn bench_ring_buffer_pop(c: &mut Criterion) {
    c.bench_function("ringbuffer_pop_1000_items", |b| {
        b.iter(|| {
            let buffer = RingBuffer::new(2000);
            
            // 先填充数据
            for i in 0..1000 {
                let _ = buffer.push(Tick::new(
                    "BTCUSDT".to_string(),
                    1000000 + i as u64,
                    1000000 + i as u64,
                    100.0,
                    1.0,
                    true,
                    i as u64,
                ));
            }
            
            // 测试 pop
            let mut count = 0;
            while let Some(_) = buffer.pop() {
                count += 1;
            }
            black_box(count);
        });
    });
}

fn bench_ring_buffer_pop_batch(c: &mut Criterion) {
    let buffer = RingBuffer::new(10000);
    
    c.bench_function("ringbuffer_pop_batch_100", |b| {
        b.iter(|| {
            // 填充数据
            for i in 0..1000 {
                let _ = buffer.push(Tick::new(
                    "BTCUSDT".to_string(),
                    1000000 + i as u64,
                    1000000 + i as u64,
                    100.0,
                    1.0,
                    true,
                    i as u64,
                ));
            }
            
            // 批量弹出
            let batch = buffer.pop_batch(100);
            black_box(batch.len());
            
            // 清空缓冲区
            while buffer.pop().is_some() {}
        });
    });
}

// ============ KLineBuilder 基准测试 ============

fn bench_kline_process_tick(c: &mut Criterion) {
    let builder = Arc::new(KLineBuilder::standard());
    
    c.bench_function("kline_process_single_tick", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let tick = Tick::new(
                    "BTCUSDT".to_string(),
                    1000000 + (i as u64 * 100),
                    1000000 + (i as u64 * 100),
                    100.0 + (i as f64 % 10.0 - 5.0) * 0.1,
                    1.0,
                    true,
                    i as u64,
                );
                black_box(builder.process_tick(&tick));
            }
        });
    });
}

fn bench_kline_get_latest(c: &mut Criterion) {
    let builder = KLineBuilder::standard();
    
    // 先生成数据
    for i in 0..10000 {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + (i as u64 * 100),
            1000000 + (i as u64 * 100),
            100.0 + (i as f64 % 10.0 - 5.0) * 0.1,
            1.0,
            true,
            i as u64,
        );
        builder.process_tick(&tick);
    }
    
    let intervals = vec![60, 300, 900, 3600, 14400, 86400];
    
    for interval in intervals {
        c.bench_with_input(
            BenchmarkId::new("kline_get_latest", interval),
            &interval,
            |b, &interval| {
                b.iter(|| {
                    black_box(builder.get_latest_kline("BTCUSDT", interval))
                });
            },
        );
    }
}

// ============ RocksDB Storage 基准测试 ============

fn bench_storage_write_single(c: &mut Criterion) {
    c.bench_function("storage_write_single_tick", |b| {
        let temp_dir = TempDir::new().unwrap();
        let storage = TickStorage::open(temp_dir.path().join("bench.db")).unwrap();
        
        b.iter(|| {
            for i in 0..100 {
                let tick = Tick::new(
                    "BTCUSDT".to_string(),
                    1000000 + i as u64,
                    1000000 + i as u64,
                    100.0,
                    1.0,
                    true,
                    i as u64,
                );
                let _ = storage.write_tick(&tick);
            }
        });
    });
}

fn bench_storage_write_batch(c: &mut Criterion) {
    c.bench_function("storage_write_batch_100", |b| {
        let temp_dir = TempDir::new().unwrap();
        let storage = TickStorage::open(temp_dir.path().join("bench.db")).unwrap();
        
        b.iter(|| {
            let mut ticks = Vec::new();
            for i in 0..100 {
                let tick = Tick::new(
                    "BTCUSDT".to_string(),
                    1000000 + i as u64,
                    1000000 + i as u64,
                    100.0,
                    1.0,
                    true,
                    i as u64,
                );
                ticks.push(tick);
            }
            let _ = storage.write_ticks(&ticks);
        });
    });
}

fn bench_storage_read(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let storage = TickStorage::open(temp_dir.path().join("bench.db")).unwrap();
    
    // 写入测试数据
    for i in 0..1000 {
        let tick = Tick::new(
            "BTCUSDT".to_string(),
            1000000 + i as u64,
            1000000 + i as u64,
            100.0,
            1.0,
            true,
            i as u64,
        );
        let _ = storage.write_tick(&tick);
    }
    
    c.bench_function("storage_read_1000_ticks", |b| {
        b.iter(|| {
            let _ = storage.read_ticks_by_symbol("BTCUSDT", 1000);
        });
    });
}

// ============ Distributor 基准测试 ============

#[tokio::main]
async fn bench_distributor_broadcast() {
    use std::time::Instant;
    
    let distributor = Distributor::new(10000);
    
    // 创建多个订阅者
    let mut receivers = Vec::new();
    for _ in 0..10 {
        receivers.push(distributor.subscribe("BTCUSDT", 60));
    }
    
    println!("\n=== Distributor Broadcast Benchmark ===");
    
    let start = Instant::now();
    let num_events = 100000;
    
    for i in 0..num_events {
        let kline = mdi::KLine::new(
            "BTCUSDT".to_string(),
            1000 + (i as u64 * 100),
            60,
            100.0 + (i as f64 % 10.0 - 5.0) * 0.1,
        );
        distributor.broadcast_kline(kline, false);
    }
    
    let elapsed = start.elapsed();
    let tps = num_events as f64 / elapsed.as_secs_f64();
    
    println!("Broadcasted {} events in {:.3}s", num_events, elapsed.as_secs_f64());
    println!("Throughput: {:.0} events/sec", tps);
    println!("Per-event latency: {:.2} µs", (elapsed.as_secs_f64() * 1_000_000.0) / num_events as f64);
}

// ============ 综合基准测试 ============

fn bench_full_pipeline(c: &mut Criterion) {
    c.bench_function("full_pipeline_1000_ticks", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let buffer = RingBuffer::new(10000);
                let kline_builder = KLineBuilder::standard();
                
                // 生成 1000 个 tick
                for i in 0..1000 {
                    let tick = Tick::new(
                        "BTCUSDT".to_string(),
                        1000000 + (i as u64 * 100),
                        1000000 + (i as u64 * 100),
                        100.0 + (i as f64 % 10.0 - 5.0) * 0.1,
                        (1.0 + (i as f64 % 9.0) * 0.01),
                        i % 2 == 0,
                        i as u64,
                    );
                    
                    // 推送到 buffer
                    let _ = buffer.push(tick.clone());
                    
                    // 处理 K线
                    black_box(kline_builder.process_tick(&tick));
                }
                
                // 读取所有 K线
                let klines = kline_builder.get_klines("BTCUSDT", 60);
                black_box(klines.len());
            });
    });
}

// ============ 基准测试组 ============

criterion_group!(
    benches,
    bench_ring_buffer_push,
    bench_ring_buffer_pop,
    bench_ring_buffer_pop_batch,
    bench_kline_process_tick,
    bench_kline_get_latest,
    bench_storage_write_single,
    bench_storage_write_batch,
    bench_storage_read,
    bench_full_pipeline,
);

criterion_main!(benches);
