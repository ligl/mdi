use crate::{Tick, KLine, MdiError, Result};
use rocksdb::{DB, Options, IteratorMode};
use serde_json;
use std::path::Path;
use std::sync::Arc;

/// RocksDB 存储层
pub struct TickStorage {
    db: Arc<DB>,
}

impl TickStorage {
    /// 创建或打开数据库
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let db = DB::open(&opts, path).map_err(|e| {
            MdiError::StorageError(format!("Failed to open RocksDB: {}", e))
        })?;

        Ok(TickStorage {
            db: Arc::new(db),
        })
    }

    /// 存储单个 Tick
    pub fn write_tick(&self, tick: &Tick) -> Result<()> {
        let key = format!("tick:{}:{}", tick.symbol, tick.trade_id);
        let value = serde_json::to_vec(tick).map_err(|e| {
            MdiError::StorageError(format!("Serialization error: {}", e))
        })?;

        self.db.put(&key, &value).map_err(|e| {
            MdiError::StorageError(format!("Put error: {}", e))
        })?;

        Ok(())
    }

    /// 批量存储 Tick
    pub fn write_ticks(&self, ticks: &[Tick]) -> Result<()> {
        let mut batch = rocksdb::WriteBatch::default();

        for tick in ticks {
            let key = format!("tick:{}:{}", tick.symbol, tick.trade_id);
            let value = serde_json::to_vec(tick).map_err(|e| {
                MdiError::StorageError(format!("Serialization error: {}", e))
            })?;
            batch.put(&key, &value);
        }

        self.db.write(batch).map_err(|e| {
            MdiError::StorageError(format!("Batch write error: {}", e))
        })?;

        Ok(())
    }

    /// 读取特定 Tick
    pub fn read_tick(&self, symbol: &str, trade_id: u64) -> Result<Option<Tick>> {
        let key = format!("tick:{}:{}", symbol, trade_id);
        
        match self.db.get(&key).map_err(|e| {
            MdiError::StorageError(format!("Get error: {}", e))
        })? {
            Some(value) => {
                let tick = serde_json::from_slice(&value).map_err(|e| {
                    MdiError::StorageError(format!("Deserialization error: {}", e))
                })?;
                Ok(Some(tick))
            }
            None => Ok(None),
        }
    }

    /// 存储 K 线
    pub fn write_kline(&self, kline: &KLine) -> Result<()> {
        let key = format!("kline:{}:{}:{}", kline.symbol, kline.interval, kline.timestamp);
        let value = serde_json::to_vec(kline).map_err(|e| {
            MdiError::StorageError(format!("Serialization error: {}", e))
        })?;

        self.db.put(&key, &value).map_err(|e| {
            MdiError::StorageError(format!("Put error: {}", e))
        })?;

        Ok(())
    }

    /// 批量存储 K 线
    pub fn write_klines(&self, klines: &[KLine]) -> Result<()> {
        let mut batch = rocksdb::WriteBatch::default();

        for kline in klines {
            let key = format!("kline:{}:{}:{}", kline.symbol, kline.interval, kline.timestamp);
            let value = serde_json::to_vec(kline).map_err(|e| {
                MdiError::StorageError(format!("Serialization error: {}", e))
            })?;
            batch.put(&key, &value);
        }

        self.db.write(batch).map_err(|e| {
            MdiError::StorageError(format!("Batch write error: {}", e))
        })?;

        Ok(())
    }

    /// 读取特定 K 线
    pub fn read_kline(&self, symbol: &str, interval: u64, timestamp: u64) -> Result<Option<KLine>> {
        let key = format!("kline:{}:{}:{}", symbol, interval, timestamp);
        
        match self.db.get(&key).map_err(|e| {
            MdiError::StorageError(format!("Get error: {}", e))
        })? {
            Some(value) => {
                let kline = serde_json::from_slice(&value).map_err(|e| {
                    MdiError::StorageError(format!("Deserialization error: {}", e))
                })?;
                Ok(Some(kline))
            }
            None => Ok(None),
        }
    }

    /// 读取指定品种的所有 Tick
    pub fn read_ticks_by_symbol(&self, symbol: &str, limit: usize) -> Result<Vec<Tick>> {
        let prefix = format!("tick:{}:", symbol);
        let iter = self.db.iterator(IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));
        
        let mut ticks = Vec::new();
        for result in iter.take(limit) {
            match result {
                Ok((key, value)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    if !key_str.starts_with(&prefix) {
                        break;
                    }

                    if let Ok(tick) = serde_json::from_slice::<Tick>(&value) {
                        ticks.push(tick);
                    }
                }
                Err(_) => break,
            }
        }

        Ok(ticks)
    }

    /// 读取指定品种的所有 K 线
    pub fn read_klines_by_symbol(&self, symbol: &str, interval: u64) -> Result<Vec<KLine>> {
        let prefix = format!("kline:{}:{}:", symbol, interval);
        let iter = self.db.iterator(IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));
        
        let mut klines = Vec::new();
        for result in iter {
            match result {
                Ok((key, value)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    if !key_str.starts_with(&prefix) {
                        break;
                    }

                    if let Ok(kline) = serde_json::from_slice::<KLine>(&value) {
                        klines.push(kline);
                    }
                }
                Err(_) => break,
            }
        }

        Ok(klines)
    }

    /// 获取数据库统计信息
    pub fn get_stats(&self) -> Result<StorageStats> {
        let property = self.db.property_value("rocksdb.stats")
            .map_err(|e| MdiError::StorageError(format!("Failed to get stats: {}", e)))?
            .unwrap_or_default();

        Ok(StorageStats {
            info: property,
        })
    }

    /// 清空数据库
    pub fn clear(&self) -> Result<()> {
        // RocksDB 没有直接的 clear 操作，需要删除所有键
        let iter = self.db.iterator(IteratorMode::Start);
        let mut batch = rocksdb::WriteBatch::default();
        let mut count = 0;

        for result in iter {
            if let Ok((key, _)) = result {
                batch.delete(&key);
                count += 1;
                
                if count >= 10000 {
                    self.db.write(batch).map_err(|e| {
                        MdiError::StorageError(format!("Batch delete error: {}", e))
                    })?;
                    batch = rocksdb::WriteBatch::default();
                    count = 0;
                }
            }
        }

        if count > 0 {
            self.db.write(batch).map_err(|e| {
                MdiError::StorageError(format!("Batch delete error: {}", e))
            })?;
        }

        Ok(())
    }
}

impl Clone for TickStorage {
    fn clone(&self) -> Self {
        TickStorage {
            db: Arc::clone(&self.db),
        }
    }
}

/// 存储统计信息
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub info: String,
}
