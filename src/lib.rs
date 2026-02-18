pub mod models;
pub mod affinity;
pub mod queue;
pub mod receiver;
pub mod kline;
pub mod storage;
pub mod distributor;

pub use models::{Tick, KLine};
pub use queue::RingBuffer;
pub use receiver::TickReceiver;
pub use kline::KLineBuilder;
pub use storage::TickStorage;
pub use distributor::Distributor;
pub use affinity::{CpuAffinity, ThreadBuilder};

/// 错误类型定义
#[derive(Debug)]
pub enum MdiError {
    ReceiverError(String),
    QueueError(String),
    StorageError(String),
    SerializationError(serde_json::Error),
    Other(String),
}

impl From<serde_json::Error> for MdiError {
    fn from(e: serde_json::Error) -> Self {
        MdiError::SerializationError(e)
    }
}

impl std::fmt::Display for MdiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MdiError::ReceiverError(e) => write!(f, "Receiver error: {}", e),
            MdiError::QueueError(e) => write!(f, "Queue error: {}", e),
            MdiError::StorageError(e) => write!(f, "Storage error: {}", e),
            MdiError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            MdiError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for MdiError {}

pub type Result<T, MdiE = MdiError> = std::result::Result<T, MdiE>;