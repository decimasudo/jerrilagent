use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct Position {
    pub symbol: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub current_price: f64,
}

impl Position {
    pub fn pnl_unrealized(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    pub fn return_percentage(&self) -> f64 {
        if self.entry_price == 0.0 {
             return 0.0;
        }
        (self.current_price - self.entry_price) / self.entry_price
    }
}

pub struct PortfolioVault {
    pub positions: HashMap<String, Position>,
    pub cash: f64,
    pub risk_free_rate: f64,
}

impl PortfolioVault {
    pub fn new(cash: f64, rfr: f64) -> Self {
        PortfolioVault {
            positions: HashMap::new(),
            cash,
            risk_free_rate: rfr,
        }
    }

    pub fn add_position(&mut self, symbol: &str, price: f64, qty: f64) -> bool {
        let cost = price * qty;
        if self.cash >= cost {
            self.cash -= cost;
            self.positions.insert(symbol.to_string(), Position {
                symbol: symbol.to_string(),
                entry_price: price,
                quantity: qty,
                current_price: price,
            });
            true
        } else {
            false
        }
    }

    pub fn update_market_price(&mut self, symbol: &str, new_price: f64) {
        if let Some(pos) = self.positions.get_mut(symbol) {
            pos.current_price = new_price;
        }
    }

    pub fn total_value(&self) -> f64 {
        let market_val: f64 = self.positions.values()
            .map(|p| p.current_price * p.quantity)
            .sum();
        self.cash + market_val
    }

    pub fn calc_weighted_returns(&self, market_hist: &HashMap<String, Vec<f64>>) -> Vec<f64> {
        let total_val = self.total_value();
        let mut daily_returns = Vec::new();
        let min_len = market_hist.values().map(|v| v.len()).min().unwrap_or(0);
        for i in 1..min_len {
            let mut day_ret = 0.0;
            for (symbol, prices) in market_hist {
                if let Some(pos) = self.positions.get(symbol) {
                    let weight = (pos.current_price * pos.quantity) / total_val;
                    let ret = (prices[i] - prices[i-1]) / prices[i-1];
                    day_ret += weight * ret;
                }
            }
            daily_returns.push(day_ret);
        }
        daily_returns
    }

    pub fn sharpe_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
             return 0.0;
        }
        let daily_rfr = (1.0 + self.risk_free_rate).powf(1.0 / 252.0) - 1.0;
        let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let excess_return = avg_return - daily_rfr;
        let variance = returns.iter()
            .map(|&r| (r - avg_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 {
             return 0.0;
        }
        (excess_return / std_dev) * (252.0f64).sqrt()
    }

    pub fn rebalance_signals(&self, target_weights: &HashMap<String, f64>) -> Vec<(String, f64)> {
        let mut signals = Vec::new();
        let total_val = self.total_value();
        for (symbol, &target) in target_weights {
            let current_pos = self.positions.get(symbol).map(|p| p.current_price * p.quantity).unwrap_or(0.0);
            let current_weight = current_pos / total_val;
            let diff = target - current_weight;
            if diff.abs() > 0.01 {
                signals.push((symbol.clone(), diff * total_val / self.positions.get(symbol).map(|p| p.current_price).unwrap_or(1.0)));
            }
        }
        signals
    }
}
