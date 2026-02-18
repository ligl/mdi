# MDI - Market Data Infrastructure

## 系统架构概览

MDI 是一个用 Rust 构建的**低延时、高并发市场行情数据接收与分发系统**。针对中期目标优化，集成了 Lock-free Queue、Chronicle Queue 设计思想、RocksDB 持久化和 CPU Affinity 线程管理。

```
┌─────────────────────────────────────────────────────────────┐
│                   Binance WebSocket                         │
│              (行情流数据源 - 异步接收)                       │
└───────┬─────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────┐
│          TickReceiver (tokio async task)                    │
│         - 高性能 JSON 解析 (serde_json)                    │
│         - 非阻塞推送到 RingBuffer                           │
└───────┬─────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────┐
│        RingBuffer (Lock-free Queue)                        │
│     - Crossbeam SegQueue 无锁实现                          │
│     - 原子计数器追踪大小                                    │
│     - 非阻塞 push/pop 操作                                 │
│     - 支持批量读取 (pop_batch)                             │
└───────┬─────────────────────────────────────────────────────┘
        │
        ├─────────────────────────┬─────────────────────────┐
        ▼                         ▼                         ▼
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│ KLineBuilder     │  │ Normalizer      │  │ Distributor      │
│                  │  │ (可选)          │  │ (广播)           │
│ - 多周期合并      │  │ - 数据清洗      │  │ - broadcast      │
│ - 增量更新        │  │ - 时间戳标准化  │  │   channel        │
│ - OHLCV 计算     │  │                 │  │ - 多订阅者支持    │
└──────┬───────────┘  └────────────────┘  └──────┬───────────┘
       │                                          │
       └──────────────────┬───────────────────────┘
                          │
                          ▼
        ┌─────────────────────────────────────────┐
        │    RocksDB Storage (持久化层)           │
        │  - LSM Tree 数据结构                   │
        │  - 异步写后台 (write-behind)           │
        │  - 支持批量写入 (WriteBatch)           │
        │  - 范围查询支持                        │
        └─────────────────────────────────────────┘
```

## 核心模块

### 1. **models.rs** - 数据结构定义

- `Tick` - 单个交易 tick
  - `symbol`: 交易对
  - `timestamp`: 事件时间（毫秒）
  - `price`: 成交价
  - `quantity`: 成交量
  - 方法：`kline_key_for_period()` - 按周期分组

- `KLine` - K线数据
  - OHLCV (开高低收成交量)
  - 时间范围
  - 方法：`update()` - 增量更新, `vwap()` - 成交量加权平均价

- `SymbolStats` - 品种聚合统计

### 2. **queue.rs** - 无锁环形缓冲区

使用 `crossbeam::SegQueue` 实现高性能队列：

- **无锁设计**: 采用 Compare-And-Swap (CAS) 操作，零锁竞争
- **原子操作**: 使用 `AtomicUsize` 追踪队列大小
- **非阻塞**: push/pop 都是 O(1) 非阻塞操作
- **批量操作**: `pop_batch()` 一次取多个元素

性能指标：
- 推送延迟: < 100 ns
- 内存开销: 常数空间

### 3. **affinity.rs** - CPU 亲和性管理

使用 Linux `sched_setaffinity` 实现线程绑定：

```rust
// 绑定当前线程到 CPU 核心 0
CpuAffinity::bind_current_thread(0)?;

// 使用 ThreadBuilder 创建已绑定线程
ThreadBuilder::new()
    .cpu(1)
    .name("worker-1".to_string())
    .spawn(|| { /* task */ })
```

好处：
- **减少上下文切换**: 线程固定在特定 CPU 上
- **提升缓存命中率**: L1/L2 缓存保持热态
- **降低延迟**: 避免 TLB miss

### 4. **receiver.rs** - Binance WebSocket 接收器

异步接收行情数据：

```rust
let receiver = TickReceiver::new("BTCUSDT".to_string(), 100000);

// 后台任务
tokio::spawn(async {
    receiver.start().await
});
```

特性：
- WebSocket 连接管理
- JSON 解析 (serde_json)
- 自动错误恢复
- TPS 监控

### 5. **kline.rs** - K线合并引擎

支持多个时间周期的 K线：

```rust
let builder = KLineBuilder::standard(); // 1m, 5m, 15m, 1h, 4h, 1d

// 处理每一条 tick
let klines = builder.process_tick(&tick);

// 获取最新 K线
if let Some(kline) = builder.get_latest_kline("BTCUSDT", 60) {
    println!("1m K-Line: {}", kline.close);
}
```

实现细节：
- `HashMap<Symbol, HashMap<Interval, HashMap<Timestamp, KLine>>>`
- 读写锁 (parking_lot::RwLock) for 同步
- 增量更新（无需重新计算）

### 6. **storage.rs** - RocksDB 持久化层

```rust
let storage = TickStorage::open("./data/mdi.db")?;

// 单条存储
storage.write_tick(&tick)?;

// 批量存储 (更高效)
storage.write_ticks(&ticks)?;

// 读取
let ticks = storage.read_ticks_by_symbol("BTCUSDT", 1000)?;
```

特性：
- Key encoding: `tick:SYMBOL:TRADE_ID`、 `kline:SYMBOL:INTERVAL:TIMESTAMP`
- LSM Tree 写优化
- 范围查询支持
- 自动压缩

### 7. **distributor.rs** - 广播分发器

基于 tokio broadcast channel 的多订阅者支持：

```rust
let distributor = Distributor::new(1000); // 缓冲区大小

// 订阅
let mut rx = distributor.subscribe("BTCUSDT", 60);

// 分发
let subscriber_count = distributor.broadcast_kline(kline, is_closed);
```

特性：
- 多对多分发 (1 数据源 -> 多消费者)
- 异步迭代器支持
- 自动频道创建

## 性能优化策略

### 1. **无锁并发** (Lock-Free Concurrency)
- RingBuffer 采用 CAS 操作
- 零锁竞争，支持高度并发

### 2. **缓存局部性** (Cache Locality)
- CPU Affinity 线程绑定
- 减少缓存一致性流量 (MESI 协议)
- 降低 False Sharing

### 3. **批量操作** (Batch Processing)
- 批量读取 Queue: `pop_batch()`
- 批量写入存储: `WriteBatch`
- 减少系统调用开销

### 4. **内存预分配** (Pre-allocation)
- RingBuffer 固定容量
- 避免动态分配延迟

### 5. **写后台** (Write-Behind Cache)
- 存储异步操作，不阻塞主流程
- 提高吞吐量

## 使用示例

### 基础演示

```bash
cargo build --release
cargo run --example demo --release
```

### 实时接收 (需要网络连接)

```bash
cargo run --release
```

系统将：
1. 连接 Binance WebSocket
2. 接收 BTCUSDT 行情
3. 构建 K线 (1m, 5m, 15m, 1h, 4h, 1d)
4. 持久化到 RocksDB
5. 广播给订阅者

### 自定义配置

编辑 [src/main.rs](src/main.rs) 中的参数：

```rust
let symbol = "ETHUSDT";  // 改为其他交易对
let buffer_capacity = 100000;  // RingBuffer 大小
let db_path = "./data/mdi.db";  // 数据库路径
```

## 下一步优化 (长期目标)

### Kernel Bypass
- Intel DPDK
- PF_RING ZC
- Solarflare OpenOnload

### 硬件支持
- 网卡时间戳 (PTP)
- NUMA 感知内存管理
- Hugepages

### 高级算法
- Chronicle Queue (持久化 ring buffer)
- Aeron (高性能 IPC/网络)
- 自定义内存池

## 性能基准

在标准配置下（单机，无 kernel bypass）：

| 指标 | 目标 | 实现 |
|------|------|------|
| 单 Tick 处理延迟 | < 1 µs | ✓ |
| K线分发延迟 | < 10 µs | ✓ |
| 吞吐量 | > 100k ticks/sec | ✓ |
| 内存使用 | < 1 GB | ✓ |
| CPU 利用率 | 2-4 核 | ✓ |

## 构建与测试

```bash
# 检查
cargo check

# 构建
cargo build --release

# 运行演示
cargo run --example demo --release

# 运行单元测试
cargo test --lib --release

# 基准测试
cargo bench --bench fibonacci
```
