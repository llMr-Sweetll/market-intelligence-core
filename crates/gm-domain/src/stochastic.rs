use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, Normal};

use crate::{context::clamp, indicators, types::PriceBar};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GbmForecast {
    pub mean_log_return: f64,
    pub std_log_return: f64,
    pub expected_return: f64,
    pub quantiles: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionRecord {
    pub symbol: String,
    pub as_of: NaiveDate,
    pub horizon: u32,
    pub model_version: String,
    pub seed: u64,
    pub n_bars: usize,
    pub mu: f64,
    pub sigma: f64,
    pub flow_adjustment: f64,
    pub expected_return: f64,
    pub forecast_std: f64,
    pub quantiles: serde_json::Value,
}

pub fn calibrate_gbm(prices: &[f64]) -> (f64, f64) {
    let returns = indicators::log_returns(prices);
    if returns.len() < 2 {
        return (0.0, 0.0);
    }
    let mu = returns.iter().sum::<f64>() / returns.len() as f64;
    let sigma = indicators::sample_stddev(&returns).unwrap_or(0.0);
    (mu, sigma)
}

pub fn gbm_forecast(mu: f64, sigma: f64, horizon: u32) -> GbmForecast {
    let normal = Normal::new(0.0, 1.0).expect("valid standard normal");
    let horizon = horizon as f64;
    let mean_log_return = mu * horizon;
    let std_log_return = sigma * horizon.sqrt();
    let expected_return = (mean_log_return + 0.5 * std_log_return.powi(2)).exp() - 1.0;
    let quantiles = [
        ("5", 0.05),
        ("25", 0.25),
        ("50", 0.50),
        ("75", 0.75),
        ("95", 0.95),
    ]
    .into_iter()
    .map(|(label, q)| {
        let z = normal.inverse_cdf(q);
        (
            label.to_string(),
            serde_json::json!((mean_log_return + std_log_return * z).exp() - 1.0),
        )
    })
    .collect::<serde_json::Map<_, _>>();

    GbmForecast {
        mean_log_return,
        std_log_return,
        expected_return,
        quantiles: serde_json::Value::Object(quantiles),
    }
}

pub fn flow_drift_adjustment(fii_flow_norm: f64, scale: f64, cap: f64) -> f64 {
    clamp(scale * fii_flow_norm, -cap, cap)
}

pub fn gbm_flow_prediction(
    symbol: &str,
    as_of: NaiveDate,
    bars: &[PriceBar],
    horizon: u32,
    fii_flow_norm: f64,
) -> PredictionRecord {
    let mut asof_bars = bars
        .iter()
        .filter(|bar| bar.date <= as_of)
        .cloned()
        .collect::<Vec<_>>();
    asof_bars.sort_by_key(|bar| bar.date);
    let closes = asof_bars.iter().map(|bar| bar.close).collect::<Vec<_>>();
    let (mu, sigma) = calibrate_gbm(&closes);
    let flow_adjustment = flow_drift_adjustment(fii_flow_norm, 0.0005, 0.002);
    let forecast = gbm_forecast(mu + flow_adjustment, sigma, horizon);

    PredictionRecord {
        symbol: symbol.to_string(),
        as_of,
        horizon,
        model_version: "gbm-flow-v1".to_string(),
        seed: 0,
        n_bars: asof_bars.len(),
        mu,
        sigma,
        flow_adjustment,
        expected_return: forecast.expected_return,
        forecast_std: forecast.std_log_return,
        quantiles: forecast.quantiles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gbm_forecast_is_deterministic() {
        let one = gbm_forecast(0.001, 0.02, 5);
        let two = gbm_forecast(0.001, 0.02, 5);
        assert_eq!(
            serde_json::to_value(one).unwrap(),
            serde_json::to_value(two).unwrap()
        );
    }

    #[test]
    fn flow_adjustment_is_capped() {
        assert_eq!(flow_drift_adjustment(10.0, 0.0005, 0.002), 0.002);
        assert_eq!(flow_drift_adjustment(-10.0, 0.0005, 0.002), -0.002);
    }
}
