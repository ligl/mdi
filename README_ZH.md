# MDI - Market Data Infrastructure

一个用 **Rust** 构建的低延时、高并发市场行情数据接收与分发系统。

## 快速开始

### 编译

```bash
# 确保安装了编译工具
# Linux: sudo apt install build-essential clang

# 构建项目
cargo build --release

# 构建并运行演示
cargo run --example demo --release
```

### 演示输出示例

```
=== MDI Market Data System Demo ===

CPU Configuration:
  Total CPUs: 8
  Physical CPUs: 4

Initializing components...
Generating simulated tick data...

Processing ticks (building klines)...
✓ Processed 1000 ticks in 0.023s (43478 ticks/sec)

K-Line Statistics:
  Total Symbols: 1
  Total K-Lines: 6
  Intervals: [60, 300, 900, 3600, 14400, 86400] seconds

Latest K-Lines for BTCUSDT:
  60s: O=100.0000 H=100.4950 L=99.5050 C=100.2850 V=49.5454 T=10
  ...
```

## 系统架构

详见 [ARCHITECTURE.md](ARCHITECTURE.md)

核心模块：

| 模块 | 功能 | 特性 |
|------|------|------|
| `models.rs` | 数据结构 | Tick、KLine、SymbolStats |
| `queue.rs` | 无锁队列 | Lock-free RingBuffer |
| `receiver.rs` | 数据接收 | Binance WebSocket 异步接收 |
| `kline.rs` | K线合并 | 多周期支持、增量更新 |
| `storage.rs` | 持久化 | RocksDB LSM 树存储 |
| `distributor.rs` | 分发 | Tokio broadcast 多订阅者 |
| `affinity.rs` | 线程绑定 | CPU 亲和性优化 |

## 主要特性

### 🚀 性能优化

- **无锁并发** - crossbeam SegQueue 零竞争
- **缓存优化** - CPU affinity 线程绑定，减少上下文切换
- **批量操作** - 批量推送/存储，降低开销
- **写后台** - 异步存储不阻塞数据流

### 📊 完整数据处理

- **Tick 接收** - 异步 WebSocket 连接
- **K线编织** - 支持 1m, 5m, 15m, 1h, 4h, 1d
- **广播分发** - 多消费者订阅同一数据流
- **持久化** - RocksDB 高性能存储

### 📈 可观测性

- **实时监控** - TPS、缓冲区使用率
- **详细日志** - tracing + 结构化日志
- **性能统计** - K线数量、订阅者数、存储数据

## 使用场景

### 1. 开发与测试

```bash
# 运行演示（本地模拟数据，无需网络）
cargo run --example demo --release
```

### 2. 实时行情接收

```bash
# 连接 Binance WebSocket 实时接收
cargo run --release
```

需要修改 [src/main.rs](src/main.rs) 中的 `symbol` 配置。

### 3. 库使用

```rust
use mdi::{TickReceiver, KLineBuilder, Distributor};

let receiver = TickReceiver::new("BTCUSDT".to_string(), 100000);
let kline_builder = KLineBuilder::standard();
let distributor = Distributor::new(1000);

// 在自己的应用中使用
```

## 配置

### 环境变量

```bash
# 日志级别
RUST_LOG=mdi=info,debug

# Cargo 编译优化
RUSTFLAGS="-C target-cpu=native"
```

### 性能调优参数

在 [src/main.rs](src/main.rs) 中调整：

```rust
// RingBuffer 容量 (越大内存越多，但容错能力越强)
let buffer_capacity = 100000;

// 分发器通道容量
let distributor = Distributor::new(1000);

// K线存储写入间隔
let storage_interval = Duration::from_secs(60);

// CPU Affinity 线程配置
let affinity_config = CpuAffinity::get_thread_affinity_config(4);
```

## 数据库结构

RocksDB 中使用前缀编码的 Key：

### Tick 存储

```
Key: "tick:SYMBOL:TRADE_ID"
Value: JSON(Tick)

示例: "tick:BTCUSDT:12345" -> {"symbol":"BTCUSDT","timestamp":1000000,...}
```

### K线存储

```
Key: "kline:SYMBOL:INTERVAL:TIMESTAMP"
Value: JSON(KLine)

示例: "kline:BTCUSDT:60:1000000" -> {"symbol":"BTCUSDT","interval":60,...}
```

### 查询

```rust
// 读取最后 1000 条 BTCUSDT tick
let ticks = storage.read_ticks_by_symbol("BTCUSDT", 1000)?;

// 读取 BTCUSDT 1分钟 K线
let klines = storage.read_klines_by_symbol("BTCUSDT", 60)?;
```

## 测试

```bash
# 运行所有单元测试
cargo test --lib --release

# 运行特定模块测试
cargo test --lib queue --release
cargo test --lib kline --release

# 性能基准测试
cargo bench
```

## 故障排查

### 编译错误

1. **缺少 C 编译工具**
   ```bash
   # Linux
   sudo apt install build-essential clang
   ```

2. **缺少 RocksDB 依赖**
   ```bash
   # 自动构建，但需要 C/C++ 编译器
   ```

### 运行时问题

1. **缓冲区溢出**
   - 增大 `buffer_capacity`
   - 检查消费者是否跟上生产者速度

2. **高延迟**
   - 检查 CPU 核心数和绑定配置
   - 减少服务器其他负载
   - 开启 release 优化

3. **存储性能**
   - 批量写入而不是单条写入
   - 定期合并 RocksDB (compaction)

## 下一步优化

### 中期 ✓ (已实现)
- [x] Lock-free RingBuffer
- [x] CPU Affinity 线程管理
- [x] RocksDB 持久化
- [x] Tokio broadcast 分发

### 长期 (规划)
- [ ] Chronicle Queue (零复制持久化)
- [ ] Kernel bypass (DPDK / PF_RING)
- [ ] 硬件时间戳 (PTP)
- [ ] 分布式部署 (Aeron IPC/网络)

## 参考资源

- [Rust 异步编程](https://rust-lang.github.io/async-book/)
- [Crossbeam 无锁编程](https://docs.rs/crossbeam/)
- [RocksDB](https://rocksdb.org/)
- [Tokio](https://tokio.rs/)

## 许可证

MIT / Apache 2.0

## 贡献

欢迎提交 Issue 和 Pull Request！

---

**最后更新**: 2026-02-14
