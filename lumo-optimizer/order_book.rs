use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub price: u64,
    pub quantity: u64,
    pub side: Side,
    pub timestamp: u128,
}

#[derive(Debug)]
pub struct OrderBook {
    pub bids: BTreeMap<u64, Vec<Order>>,
    pub asks: BTreeMap<u64, Vec<Order>>,
    pub last_update: u128,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update: 0,
        }
    }

    pub fn insert_order(&mut self, order: Order) {
        let entry = match order.side {
            Side::Buy => self.bids.entry(order.price).or_insert(Vec::new()),
            Side::Sell => self.asks.entry(order.price).or_insert(Vec::new()),
        };
        entry.push(order);
        self.last_update = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    }

    pub fn match_orders(&mut self) -> Vec<(u64, u64, u64)> {
        let mut matches = Vec::new();
        while let (Some(mut best_bid_entry), Some(mut best_ask_entry)) = (self.bids.last_entry(), self.asks.first_entry()) {
            let bid_price = *best_bid_entry.key();
            let ask_price = *best_ask_entry.key();
            if bid_price >= ask_price {
                let bids = best_bid_entry.get_mut();
                let asks = best_ask_entry.get_mut();
                while !bids.is_empty() && !asks.is_empty() {
                    let mut bid = bids.remove(0);
                    let mut ask = asks.remove(0);
                    let matched_qty = std::cmp::min(bid.quantity, ask.quantity);
                    matches.push((bid.id, ask.id, matched_qty));
                    bid.quantity -= matched_qty;
                    ask.quantity -= matched_qty;
                    if bid.quantity > 0 {
                        bids.insert(0, bid);
                    }
                    if ask.quantity > 0 {
                        asks.insert(0, ask);
                    }
                }
                if bids.is_empty() {
                    best_bid_entry.remove();
                }
                if asks.is_empty() {
                    best_ask_entry.remove();
                }
            } else {
                break;
            }
        }
        matches
    }

    pub fn get_spread(&self) -> Option<u64> {
        if let (Some(bid), Some(ask)) = (self.bids.keys().next_back(), self.asks.keys().next()) {
            Some(ask - bid)
        } else {
            None
        }
    }

    pub fn volume_at_price(&self, price: u64, side: Side) -> u64 {
        match side {
            Side::Buy => self.bids.get(&price).map(|v| v.iter().map(|o| o.quantity).sum()).unwrap_or(0),
            Side::Sell => self.asks.get(&price).map(|v| v.iter().map(|o| o.quantity).sum()).unwrap_or(0),
        }
    }
}

pub struct Engine {
    pub book: Arc<RwLock<OrderBook>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            book: Arc::new(RwLock::new(OrderBook::new())),
        }
    }

    pub fn process_order(&self, order: Order) {
        let mut book = self.book.write().unwrap();
        book.insert_order(order);
        let matches = book.match_orders();
        for m in matches {
            println!("Execution: Order {} matched with Order {} for {} units", m.0, m.1, m.2);
        }
    }
}
