import numpy as np
import pandas as pd
from typing import List, Tuple, Union, Optional
from datetime import datetime
from dataclasses import dataclass

@dataclass
class MarketSignal:
    ticker: str
    price: float
    volume: int
    timestamp: datetime
    sentiment_score: float

class IndicatorEngine:
    def __init__(self, lookback: int = 14):
        self.lookback = lookback
        self.smoothing = 2 / (lookback + 1)

    def compute_rsi(self, prices: List[float]) -> float:
        if len(prices) < self.lookback:
            return 50.0
        deltas = np.diff(prices)
        seed = deltas[:self.lookback+1]
        up = seed[seed >= 0].sum() / self.lookback
        down = -seed[seed < 0].sum() / self.lookback
        rs = up / down if down != 0 else 100
        rsi = np.zeros_like(prices)
        rsi[:self.lookback] = 100. - 100. / (1. + rs)
        for i in range(self.lookback, len(prices)):
            delta = deltas[i - 1]
            if delta > 0:
                upval = delta
                downval = 0.
            else:
                upval = 0.
                downval = -delta
            up = (upval - up) * self.smoothing + up
            down = (downval - down) * self.smoothing + down
            rs = up / down if down != 0 else 100
            rsi[i] = 100. - 100. / (1. + rs)
        return rsi[-1]

    def compute_ema(self, prices: List[float], span: int) -> List[float]:
        alpha = 2 / (span + 1)
        ema = [prices[0]]
        for i in range(1, len(prices)):
            ema.append(prices[i] * alpha + ema[-1] * (1 - alpha))
        return ema

class SignalProcessor:
    def __init__(self, engine: IndicatorEngine):
        self.engine = engine
        self._history: List[MarketSignal] = []

    def ingest(self, signals: List[MarketSignal]):
        self._history.extend(signals)
        if len(self._history) > 1000:
            self._history = self._history[-1000:]

    def generate_alpha(self, ticker: str) -> dict:
        ticker_data = [s for s in self._history if s.ticker == ticker]
        if len(ticker_data) < 20:
            return {"status": "WAITING_DATA", "score": 0.0}
        prices = [s.price for s in ticker_data]
        rsi = self.engine.compute_rsi(prices)
        ema_short = self.engine.compute_ema(prices, 9)
        ema_long = self.engine.compute_ema(prices, 21)
        momentum = 1 if ema_short[-1] > ema_long[-1] else -1
        vol_avg = sum(s.volume for s in ticker_data) / len(ticker_data)
        vol_ratio = ticker_data[-1].volume / vol_avg if vol_avg > 0 else 1.0
        final_score = (rsi * 0.4) + (momentum * 30) + (vol_ratio * 10)
        return {
            "ticker": ticker,
            "rsi": round(rsi, 2),
            "momentum_bias": momentum,
            "volume_spread": round(vol_ratio, 3),
            "lumo_score": round(final_score, 4),
            "timestamp": datetime.now().isoformat()
        }

    def aggregate_market_health(self) -> float:
        unique_tickers = list(set(s.ticker for s in self._history))
        scores = []
        for t in unique_tickers:
            res = self.generate_alpha(t)
            if res["status"] != "WAITING_DATA":
                scores.append(res["lumo_score"])
        return sum(scores) / len(scores) if scores else 0.0

if __name__ == "__main__":
    engine = IndicatorEngine()
    processor = SignalProcessor(engine)
    test_data = [
        MarketSignal("BTC", 50000 + i*10, 100, datetime.now(), 0.5) 
        for i in range(50)
    ]
    processor.ingest(test_data)
    print(processor.generate_alpha("BTC"))
