use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::context::clamp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarStats {
    pub n: usize,
    pub mean_abnormal_return: Option<f64>,
    pub std: Option<f64>,
    pub hit_rate: Option<f64>,
    pub t_stat: Option<f64>,
}

pub fn forward_return(
    closes: &BTreeMap<NaiveDate, f64>,
    event_date: NaiveDate,
    window: usize,
) -> Option<f64> {
    let dates = closes.keys().copied().collect::<Vec<_>>();
    let i = dates.iter().position(|date| *date == event_date)?;
    let j = i + window;
    if j >= dates.len() {
        return None;
    }
    let base = closes.get(&dates[i])?;
    let fwd = closes.get(&dates[j])?;
    if *base == 0.0 {
        return None;
    }
    Some(fwd / base - 1.0)
}

pub fn abnormal_return(
    stock_closes: &BTreeMap<NaiveDate, f64>,
    index_closes: &BTreeMap<NaiveDate, f64>,
    event_date: NaiveDate,
    window: usize,
) -> Option<f64> {
    let stock = forward_return(stock_closes, event_date, window)?;
    let index = forward_return(index_closes, event_date, window).unwrap_or(0.0);
    Some(stock - index)
}

pub fn cumulative_abnormal_return(samples: &[f64]) -> f64 {
    samples.iter().sum()
}

pub fn aggregate_car(samples: &[f64]) -> CarStats {
    if samples.is_empty() {
        return CarStats {
            n: 0,
            mean_abnormal_return: None,
            std: None,
            hit_rate: None,
            t_stat: None,
        };
    }

    let n = samples.len();
    let mean = samples.iter().sum::<f64>() / n as f64;
    let hit_rate = samples.iter().filter(|sample| **sample > 0.0).count() as f64 / n as f64;
    if n < 2 {
        return CarStats {
            n,
            mean_abnormal_return: Some(mean),
            std: None,
            hit_rate: Some(hit_rate),
            t_stat: None,
        };
    }

    let std = {
        let sum_sq = samples
            .iter()
            .map(|sample| (sample - mean).powi(2))
            .sum::<f64>();
        (sum_sq / (n as f64 - 1.0)).sqrt()
    };
    let se = std / (n as f64).sqrt();
    let t_stat = if se > 0.0 { Some(mean / se) } else { None };

    CarStats {
        n,
        mean_abnormal_return: Some(mean),
        std: Some(std),
        hit_rate: Some(hit_rate),
        t_stat,
    }
}

pub fn calibrated_weight(mean_abnormal_return: f64, n: usize, t_stat: Option<f64>) -> (f64, f64) {
    let weight = clamp(mean_abnormal_return * 10.0, -1.0, 1.0);
    let Some(t_stat) = t_stat else {
        return (weight, 0.0);
    };
    let significance = (t_stat.abs() / 3.0).min(1.0);
    let sample_factor = (n as f64 / 20.0).min(1.0);
    (weight, significance * sample_factor)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn aggregates_car_samples() {
        let stats = aggregate_car(&[0.02, 0.04, -0.01]);
        assert_eq!(stats.n, 3);
        assert_eq!(stats.hit_rate, Some(2.0 / 3.0));
    }

    #[test]
    fn computes_cumulative_abnormal_return() {
        let car = cumulative_abnormal_return(&[0.02, -0.01, 0.03]);
        assert!((car - 0.04).abs() < 1e-12);
    }

    #[test]
    fn computes_forward_return() {
        let closes = [
            (NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), 100.0),
            (NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(), 110.0),
            (NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(), 121.0),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        let result = forward_return(&closes, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), 2);
        assert!((result.unwrap() - 0.21).abs() < 1e-12);
    }
}
