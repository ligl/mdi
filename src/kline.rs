use crate::{Tick, KLine};
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// K 线构建器 - 支持多个时间周期
pub struct KLineBuilder {
    /// 支持的周期（秒）
    intervals: Vec<u64>,
    /// K 线缓存：symbol -> interval -> timestamp -> KLine
    klines: Arc<RwLock<HashMap<String, HashMap<u64, HashMap<u64, KLine>>>>>,
}

impl KLineBuilder {
    /// 创建新的 KLineBuilder
    /// # Arguments
    /// * `intervals` - K 线周期列表（秒） 例如: vec![60, 300, 900, 3600]
    pub fn new(intervals: Vec<u64>) -> Self {
        KLineBuilder {
            intervals,
            klines: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 标准周期: 1m, 5m, 15m, 1h, 4h, 1d
    pub fn standard() -> Self {
        // 60s, 5m, 15m, 1h, 4h, 1d
        KLineBuilder::new(vec![60, 300, 900, 3600, 14400, 86400])
    }

    /// 处理 Tick，更新相应周期的 K 线
    pub fn process_tick(&self, tick: &Tick) -> Vec<KLine> {
        let timestamp_sec = tick.timestamp / 1000; // 转换为秒
        let mut updated_klines = Vec::new();

        let mut klines = self.klines.write();
        
        for &interval in &self.intervals {
            let kline_ts = (timestamp_sec / interval) * interval;
            
            let symbol_klines = klines
                .entry(tick.symbol.clone())
                .or_insert_with(HashMap::new);
            
            let interval_klines = symbol_klines
                .entry(interval)
                .or_insert_with(HashMap::new);

            let kline = interval_klines
                .entry(kline_ts)
                .or_insert_with(|| {
                    KLine::new(
                        tick.symbol.clone(),
                        kline_ts,
                        interval,
                        tick.price,
                    )
                });

            kline.update(tick);
            updated_klines.push(kline.clone());
        }

        updated_klines
    }

    /// 获取指定品种和周期的最新 K 线
    pub fn get_latest_kline(&self, symbol: &str, interval: u64) -> Option<KLine> {
        let klines = self.klines.read();
        klines
            .get(symbol)
            .and_then(|symbol_klines| symbol_klines.get(&interval))
            .and_then(|interval_klines| {
                // 返回最新的 K 线（最大时间戳）
                interval_klines.iter().last().map(|(_, kline)| kline.clone())
            })
    }

    /// 获取指定品种、周期的所有 K 线
    pub fn get_klines(&self, symbol: &str, interval: u64) -> Vec<KLine> {
        let klines = self.klines.read();
        klines
            .get(symbol)
            .and_then(|symbol_klines| symbol_klines.get(&interval))
            .map(|interval_klines| {
                let mut bars: Vec<_> = interval_klines.values().cloned().collect();
                bars.sort_by_key(|k| k.timestamp);
                bars
            })
            .unwrap_or_default()
    }

    /// 获取所有品种和周期的 K 线统计
    pub fn get_stats(&self) -> KLineStats {
        let klines = self.klines.read();
        let mut total_klines = 0;
        let mut symbols = std::collections::HashSet::new();
        
        for (symbol, symbol_klines) in klines.iter() {
            symbols.insert(symbol.clone());
            for (_, interval_klines) in symbol_klines.iter() {
                total_klines += interval_klines.len();
            }
        }

        KLineStats {
            total_symbols: symbols.len(),
            total_klines,
            intervals: self.intervals.clone(),
        }
    }

    /// 清空所有 K 线数据
    pub fn clear(&self) {
        self.klines.write().clear();
    }

    /// 获取数据的写入权限（用于外部修改）
    pub fn get_klines_lock(&self) -> Arc<RwLock<HashMap<String, HashMap<u64, HashMap<u64, KLine>>>>> {
        Arc::clone(&self.klines)
    }
}

impl Clone for KLineBuilder {
    fn clone(&self) -> Self {
        KLineBuilder {
            intervals: self.intervals.clone(),
            klines: Arc::clone(&self.klines),
        }
    }
}

/// K 线统计信息
#[derive(Debug, Clone)]
pub struct KLineStats {
    pub total_symbols: usize,
    pub total_klines: usize,
    pub intervals: Vec<u64>,
}
