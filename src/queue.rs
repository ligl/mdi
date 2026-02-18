use crate::{Tick, MdiError, Result};
use crossbeam::queue::SegQueue;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 高性能无锁环形缓冲区
/// 使用 crossbeam 的 SegQueue 作为底层实现
pub struct RingBuffer {
    queue: Arc<SegQueue<Tick>>,
    capacity: usize,
    size: Arc<AtomicUsize>,
}

impl RingBuffer {
    /// 创建新的 RingBuffer
    pub fn new(capacity: usize) -> Self {
        RingBuffer {
            queue: Arc::new(SegQueue::new()),
            capacity,
            size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 推送 Tick 数据到缓冲区（非阻塞）
    pub fn push(&self, tick: Tick) -> Result<()> {
        let current_size = self.size.load(Ordering::Relaxed);
        
        // 如果超过容量，返回错误（丢弃数据 - HFT 优先保证延迟）
        if current_size >= self.capacity {
            return Err(MdiError::QueueError(
                format!("RingBuffer full: {}/{}", current_size, self.capacity)
            ));
        }

        self.queue.push(tick);
        self.size.fetch_add(1, Ordering::Release);
        Ok(())
    }

    /// 弹出数据（非阻塞）
    pub fn pop(&self) -> Option<Tick> {
        self.queue.pop().and_then(|tick| {
            self.size.fetch_sub(1, Ordering::Release);
            Some(tick)
        })
    }

    /// 批量弹出数据
    pub fn pop_batch(&self, max_count: usize) -> Vec<Tick> {
        let mut batch = Vec::with_capacity(max_count);
        for _ in 0..max_count {
            if let Some(tick) = self.pop() {
                batch.push(tick);
            } else {
                break;
            }
        }
        batch
    }

    /// 获取当前队列大小
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }

    /// 队列是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 清空队列
    pub fn clear(&self) {
        while let Some(_) = self.pop() {}
    }

    /// 获取队列容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 获取当前使用率（0-100）
    pub fn usage_percent(&self) -> f64 {
        (self.len() as f64 / self.capacity as f64) * 100.0
    }

    /// 克隆引用（用于多个消费者）
    pub fn clone_ref(&self) -> Self {
        RingBuffer {
            queue: Arc::clone(&self.queue),
            capacity: self.capacity,
            size: Arc::clone(&self.size),
        }
    }
}

impl Clone for RingBuffer {
    fn clone(&self) -> Self {
        self.clone_ref()
    }
}
