use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{indicators, types::PriceBar};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureVector {
    pub symbol: String,
    pub as_of: NaiveDate,
    pub n_bars: usize,
    pub close: Option<f64>,
    pub momentum_1m: Option<f64>,
    pub momentum_3m: Option<f64>,
    pub momentum_6m: Option<f64>,
    pub rsi_14: Option<f64>,
    pub atr_14: Option<f64>,
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub ema_20: Option<f64>,
    pub annualized_vol: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub zscore_20: Option<f64>,
    pub adv_20: Option<f64>,
}

pub fn compute_features(symbol: &str, as_of: NaiveDate, bars: &[PriceBar]) -> FeatureVector {
    let mut asof_bars = bars
        .iter()
        .filter(|bar| bar.date <= as_of)
        .cloned()
        .collect::<Vec<_>>();
    asof_bars.sort_by_key(|bar| bar.date);

    let closes = asof_bars.iter().map(|bar| bar.close).collect::<Vec<_>>();
    let highs = asof_bars.iter().map(|bar| bar.high).collect::<Vec<_>>();
    let lows = asof_bars.iter().map(|bar| bar.low).collect::<Vec<_>>();
    let volumes = asof_bars
        .iter()
        .map(|bar| bar.volume as f64)
        .collect::<Vec<_>>();

    FeatureVector {
        symbol: symbol.to_string(),
        as_of,
        n_bars: asof_bars.len(),
        close: closes.last().copied(),
        momentum_1m: indicators::momentum(&closes, 21),
        momentum_3m: indicators::momentum(&closes, 63),
        momentum_6m: indicators::momentum(&closes, 126),
        rsi_14: indicators::rsi(&closes, 14),
        atr_14: indicators::atr(&highs, &lows, &closes, 14),
        sma_20: indicators::sma(&closes, 20),
        sma_50: indicators::sma(&closes, 50),
        ema_20: indicators::ema(&closes, 20),
        annualized_vol: indicators::annualized_volatility(&closes, 252.0),
        max_drawdown: indicators::max_drawdown(&closes),
        zscore_20: indicators::zscore(&closes, 20),
        adv_20: indicators::sma(&volumes, 20),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn bar(day: u32, close: f64) -> PriceBar {
        PriceBar {
            symbol: "RELIANCE".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            open: close,
            high: close + 1.0,
            low: close - 1.0,
            close,
            adj_close: None,
            volume: 1000,
            source: "fixture".to_string(),
        }
    }

    #[test]
    fn compute_features_enforces_as_of_cutoff() {
        let bars = vec![bar(1, 100.0), bar(2, 101.0), bar(3, 200.0)];
        let features = compute_features(
            "RELIANCE",
            NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            &bars,
        );

        assert_eq!(features.n_bars, 2);
        assert_eq!(features.close, Some(101.0));
    }
}
