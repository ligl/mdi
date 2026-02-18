use serde::{Deserialize, Serialize};

/// 行情 Tick 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    /// 交易对
    pub symbol: String,
    /// 时间戳（毫秒）
    pub timestamp: u64,
    /// 记录时间
    pub event_time: u64,
    /// 价格
    pub price: f64,
    /// 数量
    pub quantity: f64,
    /// 买卖方向
    pub is_buyer_maker: bool,
    /// 交易 ID
    pub trade_id: u64,
}

impl Tick {
    pub fn new(
        symbol: String,
        timestamp: u64,
        event_time: u64,
        price: f64,
        quantity: f64,
        is_buyer_maker: bool,
        trade_id: u64,
    ) -> Self {
        Tick {
            symbol,
            timestamp,
            event_time,
            price,
            quantity,
            is_buyer_maker,
            trade_id,
        }
    }

    /// 按秒获取分组键
    pub fn kline_key_for_period(&self, period_seconds: u64) -> u64 {
        (self.timestamp / 1000) / period_seconds * period_seconds
    }
}

/// K 线数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLine {
    /// 交易对
    pub symbol: String,
    /// 时间戳（秒）
    pub timestamp: u64,
    /// K 线间隔（秒）
    pub interval: u64,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量
    pub volume: f64,
    /// 成交金额
    pub quote_asset_volume: f64,
    /// 成交笔数
    pub number_of_trades: u64,
    /// 时间范围
    pub open_time: u64,
    pub close_time: u64,
}

impl KLine {
    pub fn new(
        symbol: String,
        timestamp: u64,
        interval: u64,
        open: f64,
    ) -> Self {
        KLine {
            symbol,
            timestamp,
            interval,
            open,
            high: open,
            low: open,
            close: open,
            volume: 0.0,
            quote_asset_volume: 0.0,
            number_of_trades: 0,
            open_time: timestamp,
            close_time: timestamp + interval,
        }
    }

    /// 更新 K 线（增量更新）
    pub fn update(&mut self, tick: &Tick) {
        self.high = self.high.max(tick.price);
        self.low = self.low.min(tick.price);
        self.close = tick.price;
        self.volume += tick.quantity;
        self.quote_asset_volume += tick.price * tick.quantity;
        self.number_of_trades += 1;
    }

    /// 获取 K 线关键指标
    pub fn vwap(&self) -> f64 {
        if self.volume == 0.0 {
            0.0
        } else {
            self.quote_asset_volume / self.volume
        }
    }

    /// K 线涨跌幅
    pub fn change_percent(&self) -> f64 {
        if self.open == 0.0 {
            0.0
        } else {
            ((self.close - self.open) / self.open) * 100.0
        }
    }
}

/// 聚合品种数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolStats {
    pub symbol: String,
    pub tick_count: u64,
    pub volume: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub last_price: f64,
    pub last_update: u64,
}

impl SymbolStats {
    pub fn new(symbol: String) -> Self {
        SymbolStats {
            symbol,
            tick_count: 0,
            volume: 0.0,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            last_price: 0.0,
            last_update: 0,
        }
    }

    pub fn update(&mut self, tick: &Tick) {
        if self.tick_count == 0 {
            self.open = tick.price;
            self.high = tick.price;
            self.low = tick.price;
        }

        self.tick_count += 1;
        self.volume += tick.quantity;
        self.high = self.high.max(tick.price);
        self.low = self.low.min(tick.price);
        self.last_price = tick.price;
        self.last_update = tick.timestamp;
    }
}
