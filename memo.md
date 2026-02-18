# mdi

## 常用库

- tokio 异步
- rayon 并发
- chrono 日期时间
- tracing 日志与跟踪调试
- log
- serde 序列化与反序列化
- pyo3
- polars
- rand 随机数
- base64
- reqwest http client
- libc  raw FFI like libc
- toml encoder and decoder fro TOML file
- ndarray
- anyhow 方便使用Error
- thiserror 自定义错误
- tempfile 管理临时文件
- socket2
- io_uring

## dev

- maturin

## 练习示例

### 1. Binance 行情接收与分发

```scss
Binance WebSocket (tick)
        │
        ▼
Tick 数据接收器 (async tokio task)
        │
        ▼
Tick 存储器 (内存/数据库)
        │
        ▼
K线合并器 (按周期: 1m, 5m, 15m 等)
        │
        ▼
广播服务 (订阅者通过 channel 接收)
```


### 核心模块 ###

1. **Tick 数据接收**

- 使用`tokio`异步任务连接Binance WebSocket
- 解析JSON tick数据`serde_json`
- 发到内存队列 `tokio::mpsc` 供K线合并使用

2. **K线合并**

- 按时间段（1m,5m,15m,1h,1d）合并tick → K线
- 支持增量更新（Tick来了就更新当前K线）
- 用`tokio::mpsc::broadcast`或`tokio::sync::watch`广播K线更新给订阅者

3. **订阅者/消费者**

- 订阅合并后的K线数据
- 支持多消费者同时订阅

4. **落地保存**

- memmap
- rocksdb

## **高频交易(HFT)**场景

**行情系统的三个核心要求：**

- **极低延迟**
- **高吞吐与零丢包**
- **可扩展的分发架构**

### 1. 行情接入

- **UDP Multicast + Kernel Bypass**
  - Intel DPDK / PF_RING ZC / Solarflare OpenOnload
  - 直接从网卡 DMA 到用户空间，绕过内核网络栈
  - 延迟：常见 500ns ~ 3µs
  - 丢包率极低（配合大 buf + 多核处理）
  - 推荐网卡：Solarflare / Broadcom / Mellanox 支持 kernel bypass
  
- **实时时间戳**
  - 必须从硬件 PTP / 10GbE/25GbE 网卡获得行级时间戳
  - 软件时间戳精度无法满足 ~100ns 要求

### 2. 报文解析与分发

**总体方案目标：零锁竞争 + 低复制 + 高并发**

**设计模式**

- A) Zero-Copy + Lock-Free Queues
  - 所有解析线程解析后写入 lock-free ring buffer
  - 消费者直接从 ring 读取，无内存复制
  - 推荐结构：
    - LMAX Disruptor 模式
    - Hazelcast’s Ringbuffer
    - Boost::lockfree::queue（简单场景）
- B) 高性能 IPC / Network Stack
  - Aeron | Aeron Cluste
  - UDP、IPC 多消费者支持
  - 低延迟、可靠传输
  - Nanomsg / NNG
  - 简洁但性能次于 Aeron
  - Redis Streams（**仅适合中延迟分析系统，不适合 HFT 核心）

| 方案              | 延迟 | 适用         |
| --------------- | -- | ---------- |
| Aeron           | 低  | 多进程分发，可跨机器 |
| Disruptor Ring  | 极低 | 单进程多线程     |
| Chronicle Queue | 极低 | 持久化 + 分发   |

### 3. 内存与缓存策略

**内存布局**

- Hugepages / NUMA 绑定
- 避免 TLB miss
- 线程与内存局部性优化
- Pre-allocated object pool
- 解析对象复用，无 GC
- CPU Affinity
- RX IRQ 绑定行情接收线程
- 处理线程固定核心（避免迁移抖动）

**缓存优化**

- 所有热点数据使用 cache-aligned struct
- 避免 false sharing（每个 consumer 对应独立缓存线）
- 使用 prefetch 指令

### 4. 落地存储

HFT 核心 tick 数据写入需要**高吞吐、低延迟、可检索**

**高性能选择**
1. 专用TickDB

### 5. 架构参考

```pgsql

Exchange → NIC (25/40/100Gbps)
           ↓ (kernel bypass)
Market Data Receiver Threads
           ↓ zero-copy
Ring Buffer (Disruptor)
           ↓
/--------------\
| Processor(s) |
| – Normalizer |
| – Filter     |
| – TimeStamp  |
\--------------/
           ↓ Aeron IPC/IPC
+-------------------------------+
| Strategy Engines (multi proc) |
+-------------------------------+
           ↓ Write Behind
Chronicle Queue / kdb+ Tick
           ↓ Async Sync
Analytics / Backtester

```

### 6. 技术栈

**Rust**

- zero-cost abstraction
- 安全无 GC
- ecosystem 支持：
  - tokio-uring / io_uring
  - Zero-copy crates
  - DPDK bindings

**其他**

- Linux RT Kernel
- NUMA / CPU affinity
- PTP/Precision Time Sync

### 7. 典型性能指标参考

| 指标       | 目标                   |
| -------- | -------------------- |
| 行情接收延迟   | < 1–3 µs             |
| 解析到内存分发  | < 5–10 µs end-to-end |
| 写盘延迟（同步） | < few 100 µs         |
| 分发吞吐     | > 数百万报文/s            |

### 8. 结论

| 组件   | 最优方案                                         |
| ---- | -------------------------------------------- |
| 接入   | UDP Multicast + Kernel Bypass (DPDK/PF_RING) |
| 分发   | Lock-Free Ring + Aeron / Disruptor           |
| 落地   | kdb+/Chronicle Queue + Async IO              |
| 系统优化 | NUMA/Hugepages/PTP/CPU Affinity              |
