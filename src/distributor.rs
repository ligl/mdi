use crate::KLine;
use tokio::sync::broadcast;
use std::sync::Arc;

/// K 线广播事件
#[derive(Debug, Clone)]
pub struct KLineEvent {
    pub kline: KLine,
    pub is_closed: bool, // K 线是否已完成
}

/// 分发器 - 管理多个订阅通道
pub struct Distributor {
    /// symbol -> （interval -> broadcast channel）
    channels: Arc<parking_lot::RwLock<std::collections::HashMap<String, Arc<broadcast::Sender<KLineEvent>>>>>,
    channel_capacity: usize,
}

impl Distributor {
    /// 创建新的分发器
    pub fn new(channel_capacity: usize) -> Self {
        Distributor {
            channels: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            channel_capacity,
        }
    }

    /// 发送 K 线事件
    pub fn broadcast_kline(&self, kline: KLine, is_closed: bool) -> usize {
        let key = format!("{}:{}", kline.symbol, kline.interval);
        let channels = self.channels.read();

        if let Some(sender) = channels.get(&key) {
            let event = KLineEvent { kline, is_closed };
            // 记录失败的订阅者数量，但不中断广播
            let _ = sender.send(event);
            sender.receiver_count()
        } else {
            0
        }
    }

    /// 订阅指定品种和周期的 K 线
    pub fn subscribe(&self, symbol: &str, interval: u64) -> broadcast::Receiver<KLineEvent> {
        let key = format!("{}:{}", symbol, interval);
        let mut channels = self.channels.write();

        let sender = channels
            .entry(key)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(self.channel_capacity);
                Arc::new(tx)
            })
            .clone();

        sender.subscribe()
    }

    /// 获取订阅者数量
    pub fn subscriber_count(&self, symbol: &str, interval: u64) -> usize {
        let key = format!("{}:{}", symbol, interval);
        let channels = self.channels.read();
        channels
            .get(&key)
            .map(|sender| sender.receiver_count())
            .unwrap_or(0)
    }

    /// 获取所有活跃频道
    pub fn active_channels(&self) -> Vec<String> {
        let channels = self.channels.read();
        channels
            .iter()
            .filter(|(_, sender)| sender.receiver_count() > 0)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// 清空所有频道
    pub fn clear(&self) {
        self.channels.write().clear();
    }
}

impl Clone for Distributor {
    fn clone(&self) -> Self {
        Distributor {
            channels: Arc::clone(&self.channels),
            channel_capacity: self.channel_capacity,
        }
    }
}
