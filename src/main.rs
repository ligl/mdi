use mdi::{
    TickReceiver, KLineBuilder, Distributor, TickStorage, CpuAffinity, Result as MdiResult,
};
use tokio::task::JoinHandle;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber;

#[tokio::main]
async fn main() -> MdiResult<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("=== MDI Market Data System Started ===");
    tracing::info!("CPU Cores: {}", CpuAffinity::num_cpus());

    // 配置参数
    let symbol = "BTCUSDT";
    let buffer_capacity = 100000;
    let db_path = "./data/mdi.db";

    // 1. 创建核心组件
    tracing::info!("Initializing components...");
    
    let receiver = Arc::new(TickReceiver::new(
        symbol.to_string(),
        buffer_capacity,
    ));
    
    let kline_builder = Arc::new(KLineBuilder::standard());
    let distributor = Arc::new(Distributor::new(1000));
    let storage = Arc::new(TickStorage::open(db_path)?);
    
    let tick_buffer = receiver.buffer();

    // 2. 启动 Binance WebSocket 接收器（后台任务）
    tracing::info!("Starting Binance WebSocket receiver for {}...", symbol);
    
    let receiver_clone = Arc::clone(&receiver);
    let receiver_handle: JoinHandle<MdiResult<()>> = tokio::spawn(async move {
        receiver_clone.start().await
    });

    // 3. 启动 K 线处理任务
    tracing::info!("Starting KLine processor...");
    
    let kline_builder_clone = Arc::clone(&kline_builder);
    let distributor_clone = Arc::clone(&distributor);
    let storage_clone = Arc::clone(&storage);
    let buffer_clone = tick_buffer.clone();
    
    let processor_handle: JoinHandle<()> = tokio::spawn(async move {
        let mut last_storage_time = std::time::Instant::now();
        let storage_interval = Duration::from_secs(60); // 每 60 秒写入一次
        let mut tick_batch = Vec::with_capacity(1000);
        
        loop {
            // 批量处理 tick（每次最多 1000 个）
            while let Some(tick) = buffer_clone.pop() {
                // 处理 K 线
                let klines = kline_builder_clone.process_tick(&tick);
                
                // 分发 K 线
                for kline in klines {
                    let is_closed = false; // 简化处理
                    distributor_clone.broadcast_kline(kline, is_closed);
                }
                
                tick_batch.push(tick);
            }

            // 批量写入存储
            if !tick_batch.is_empty() && last_storage_time.elapsed() > storage_interval {
                if let Err(e) = storage_clone.write_ticks(&tick_batch) {
                    tracing::warn!("Failed to write ticks to storage: {}", e);
                }
                tick_batch.clear();
                last_storage_time = std::time::Instant::now();
            }

            // 短暂休眠，避免 busy loop
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    // 4. 启动订阅者示例（消费 K 线数据）
    tracing::info!("Starting KLine subscribers...");
    
    let distributor_clone = Arc::clone(&distributor);
    let subscriber_handle: JoinHandle<()> = tokio::spawn(async move {
        let intervals = vec![60, 300, 900]; // 1m, 5m, 15m
        let mut receivers = Vec::new();

        // 创建订阅者
        for interval in &intervals {
            let rx = distributor_clone.subscribe(symbol, *interval);
            receivers.push((interval, rx));
        }

        let mut count = 0;
        loop {
            // 异步等待任何一个 K 线更新
            for (interval, rx) in receivers.iter_mut() {
                match rx.try_recv() {
                    Ok(event) => {
                        count += 1;
                        if count % 100 == 0 {
                            tracing::info!(
                                "KLine {}s: time={}, price={:.2}, volume={:.8}, trades={}",
                                interval,
                                event.kline.timestamp,
                                event.kline.close,
                                event.kline.volume,
                                event.kline.number_of_trades
                            );
                        }
                    }
                    Err(_) => {}
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    // 5. 启动监控任务
    let kline_builder_clone = Arc::clone(&kline_builder);
    let buffer_clone = tick_buffer.clone();
    
    let monitor_handle: JoinHandle<()> = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;

            let stats = kline_builder_clone.get_stats();
            let buffer_usage = buffer_clone.usage_percent();

            tracing::info!(
                "=== System Status ===\n\
                 Symbols: {}\n\
                 KLines: {}\n\
                 Buffer Usage: {:.2}%\n\
                 Buffer Size: {}/{}",
                stats.total_symbols,
                stats.total_klines,
                buffer_usage,
                buffer_clone.len(),
                buffer_clone.capacity()
            );
        }
    });

    // 等待任意任务完成（通常是接收器）
    tokio::select! {
        res = receiver_handle => {
            if let Err(e) = res {
                tracing::error!("Receiver task error: {}", e);
            }
        }
        _ = processor_handle => {
            tracing::info!("Processor task completed");
        }
        _ = subscriber_handle => {
            tracing::info!("Subscriber task completed");
        }
        _ = monitor_handle => {
            tracing::info!("Monitor task completed");
        }
    }

    Ok(())
}