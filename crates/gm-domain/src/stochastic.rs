use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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
        let z = inverse_standard_normal_cdf(q);
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

fn inverse_standard_normal_cdf(p: f64) -> f64 {
    assert!((0.0..=1.0).contains(&p), "probability must be in [0, 1]");
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }

    // Peter J. Acklam's rational approximation. Accurate enough for forecast
    // quantiles, dependency-free, and deterministic across platforms.
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];

    let plow = 0.02425;
    let phigh = 1.0 - plow;

    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        return (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0);
    }

    if p > phigh {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        return -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0);
    }

    let q = p - 0.5;
    let r = q * q;
    (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
        / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
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

    #[test]
    fn inverse_normal_quantiles_match_expected_values() {
        assert!(inverse_standard_normal_cdf(0.50).abs() < 1e-12);
        assert!((inverse_standard_normal_cdf(0.05) - -1.644_853_625).abs() < 1e-6);
        assert!((inverse_standard_normal_cdf(0.95) - 1.644_853_625).abs() < 1e-6);
    }
}
