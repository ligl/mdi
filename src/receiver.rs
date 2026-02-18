use crate::{MdiError, Result, RingBuffer, Tick};
use futures::stream::StreamExt;
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/ws";

/// Binance WebSocket 行情接收器
pub struct TickReceiver {
    ring_buffer: RingBuffer,
    symbol: String,
}

impl TickReceiver {
    /// 创建新的接收器
    pub fn new(symbol: String, buffer_capacity: usize) -> Self {
        TickReceiver {
            ring_buffer: RingBuffer::new(buffer_capacity),
            symbol,
        }
    }

    /// 获取 RingBuffer 引用
    pub fn buffer(&self) -> RingBuffer {
        self.ring_buffer.clone_ref()
    }

    /// 开始接收行情数据（阻塞）
    pub async fn start(&self) -> Result<()> {
        let stream_name = format!("{}@trade", self.symbol.to_lowercase());
        let url = format!("{}/{}", BINANCE_WS_URL, stream_name);

        tracing::info!("Connecting to Binance WebSocket: {}", url);

        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                tracing::info!("Connected to Binance WebSocket");
                self.process_stream(ws_stream).await
            }
            Err(e) => {
                tracing::error!("Failed to connect to Binance: {}", e);
                Err(MdiError::ReceiverError(format!(
                    "WebSocket connection failed: {}",
                    e
                )))
            }
        }
    }

    /// 处理 WebSocket 流
    async fn process_stream<S>(&self, stream: S) -> Result<()>
    where
        S: futures::stream::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Unpin,
    {
        let mut stream = stream;
        let mut tick_count = 0u64;
        let start_time = std::time::Instant::now();

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Text(text)) => match self.parse_tick(&text) {
                    Ok(tick) => match self.ring_buffer.push(tick) {
                        Ok(_) => {
                            tick_count += 1;
                            if tick_count % 10000 == 0 {
                                let elapsed = start_time.elapsed().as_secs_f64();
                                let tps = tick_count as f64 / elapsed;
                                tracing::info!(
                                    "Received {} ticks, TPS: {:.2}, Buffer usage: {:.2}%",
                                    tick_count,
                                    tps,
                                    self.ring_buffer.usage_percent()
                                );
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to push tick to buffer: {}", e);
                        }
                    },
                    Err(e) => {
                        tracing::debug!("Failed to parse tick: {}", e);
                    }
                },
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    return Err(MdiError::ReceiverError(format!("WebSocket error: {}", e)));
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 解析 Binance JSON 消息为 Tick
    pub fn parse_tick(&self, json_str: &str) -> Result<Tick> {
        let value: Value = serde_json::from_str(json_str)?;

        let timestamp = value
            .get("E")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| MdiError::Other("Missing 'E' field".to_string()))?;

        let event_time = value.get("T").and_then(|v| v.as_u64()).unwrap_or(timestamp);

        let price: f64 = value
            .get("p")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| MdiError::Other("Missing or invalid 'p' field".to_string()))?;

        let quantity: f64 = value
            .get("q")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| MdiError::Other("Missing or invalid 'q' field".to_string()))?;

        let is_buyer_maker = value.get("m").and_then(|v| v.as_bool()).unwrap_or(false);

        let trade_id = value.get("t").and_then(|v| v.as_u64()).unwrap_or(0);

        Ok(Tick::new(
            self.symbol.clone(),
            timestamp,
            event_time,
            price,
            quantity,
            is_buyer_maker,
            trade_id,
        ))
    }
}
