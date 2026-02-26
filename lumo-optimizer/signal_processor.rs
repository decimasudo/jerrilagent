use std::f64::consts::PI;
use std::collections::VecDeque;

pub trait QuantSignal {
    fn calculate(&self, data: &[f64]) -> f64;
    fn get_name(&self) -> String;
}

pub struct MovingAverageConvergenceDivergence {
    pub short_period: usize,
    pub long_period: usize,
    pub signal_period: usize,
}

impl QuantSignal for MovingAverageConvergenceDivergence {
    fn calculate(&self, data: &[f64]) -> f64 {
        if data.len() < self.long_period {
            return 0.0;
        }
        let ema_short = self.ema(data, self.short_period);
        let ema_long = self.ema(data, self.long_period);
        ema_short - ema_long
    }

    fn get_name(&self) -> String {
        "MACD".to_string()
    }
}

impl MovingAverageConvergenceDivergence {
    fn ema(&self, data: &[f64], period: usize) -> f64 {
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = data[0];
        for val in data.iter().skip(1) {
            ema = (val - ema) * alpha + ema;
        }
        ema
    }

    pub fn generate_histogram(&self, data: &[f64]) -> f64 {
        let macd_line = self.calculate(data);
        let mut macd_history = Vec::<f64>::new();
        for i in (self.signal_period..data.len()).rev().take(self.signal_period) {
            macd_history.push(self.calculate(&data[..i]));
        }
        let signal_line = self.ema(&macd_history, self.signal_period);
        macd_line - signal_line
    }
}

pub struct BollingerBands {
    pub period: usize,
    pub num_std_dev: f64,
}

impl QuantSignal for BollingerBands {
    fn calculate(&self, data: &[f64]) -> f64 {
        if data.len() < self.period {
            return 0.0;
        }
        let subset = &data[data.len() - self.period..];
        let mean = subset.iter().sum::<f64>() / self.period as f64;
        let variance = subset.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.period as f64;
        let std_dev = variance.sqrt();
        (data.last().unwrap() - (mean - self.num_std_dev * std_dev)) / (self.num_std_dev * 2.0 * std_dev)
    }

    fn get_name(&self) -> String {
        "BB_PERCENT".to_string()
    }
}

pub struct SignalAggregator {
    pub signals: Vec<Box<dyn QuantSignal>>,
    pub weights: Vec<f64>,
}

impl SignalAggregator {
    pub fn new() -> Self {
        SignalAggregator {
            signals: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add_signal(&mut self, signal: Box<dyn QuantSignal>, weight: f64) {
        self.signals.push(signal);
        self.weights.push(weight);
    }

    pub fn compute_aggregate_confidence(&self, data: &[f64]) -> f64 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        for (i, signal) in self.signals.iter().enumerate() {
            let score = signal.calculate(data);
            total_score += score * self.weights[i];
            total_weight += self.weights[i];
        }
        total_score / total_weight
    }

    pub fn fast_fourier_transform_filter(&self, data: &[f64]) -> Vec<f64> {
        let n = data.len();
        let mut out = Vec::with_capacity(n);
        for k in 0..n {
            let mut sum_re = 0.0;
            let mut sum_im = 0.0;
            for t in 0..n {
                let angle = 2.0 * PI * (k as f64) * (t as f64) / (n as f64);
                sum_re += data[t] * angle.cos();
                sum_im -= data[t] * angle.sin();
            }
            out.push((sum_re.powi(2) + sum_im.powi(2)).sqrt());
        }
        out
    }
}
