pub fn simple_returns(prices: &[f64]) -> Vec<f64> {
    prices.windows(2).map(|w| w[1] / w[0] - 1.0).collect()
}

pub fn log_returns(prices: &[f64]) -> Vec<f64> {
    prices.windows(2).map(|w| (w[1] / w[0]).ln()).collect()
}

pub fn sma(values: &[f64], window: usize) -> Option<f64> {
    if window == 0 || values.len() < window {
        return None;
    }
    let slice = &values[values.len() - window..];
    Some(slice.iter().sum::<f64>() / window as f64)
}

pub fn ema(prices: &[f64], window: usize) -> Option<f64> {
    if window == 0 || prices.len() < window {
        return None;
    }
    let alpha = 2.0 / (window as f64 + 1.0);
    let mut value = prices[..window].iter().sum::<f64>() / window as f64;
    for price in &prices[window..] {
        value = alpha * price + (1.0 - alpha) * value;
    }
    Some(value)
}

pub fn momentum(prices: &[f64], window: usize) -> Option<f64> {
    if prices.len() < window + 1 {
        return None;
    }
    Some(prices[prices.len() - 1] / prices[prices.len() - window - 1] - 1.0)
}

pub fn rsi(prices: &[f64], period: usize) -> Option<f64> {
    if period == 0 || prices.len() < period + 1 {
        return None;
    }

    let deltas = prices.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
    let mut gains = Vec::with_capacity(deltas.len());
    let mut losses = Vec::with_capacity(deltas.len());
    for delta in deltas {
        gains.push(delta.max(0.0));
        losses.push((-delta).max(0.0));
    }

    let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;
    for i in period..gains.len() {
        avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i]) / period as f64;
    }

    if avg_loss == 0.0 {
        return Some(100.0);
    }
    if avg_gain == 0.0 {
        return Some(0.0);
    }

    let rs = avg_gain / avg_loss;
    Some(100.0 - 100.0 / (1.0 + rs))
}

pub fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Option<f64> {
    let n = closes.len();
    if period == 0 || n < period || highs.len() != n || lows.len() != n {
        return None;
    }

    let mut tr = Vec::with_capacity(n);
    tr.push(highs[0] - lows[0]);
    for i in 1..n {
        tr.push(
            (highs[i] - lows[i])
                .max((highs[i] - closes[i - 1]).abs())
                .max((lows[i] - closes[i - 1]).abs()),
        );
    }

    sma(&tr, period)
}

pub fn max_drawdown(prices: &[f64]) -> Option<f64> {
    if prices.len() < 2 {
        return None;
    }

    let mut running_max = prices[0];
    let mut max_drawdown = 0.0;
    for price in prices {
        running_max = running_max.max(*price);
        let drawdown = price / running_max - 1.0;
        if drawdown < max_drawdown {
            max_drawdown = drawdown;
        }
    }
    Some(max_drawdown)
}

pub fn annualized_volatility(prices: &[f64], periods_per_year: f64) -> Option<f64> {
    let returns = log_returns(prices);
    if returns.len() < 2 {
        return None;
    }
    Some(sample_stddev(&returns)? * periods_per_year.sqrt())
}

pub fn zscore(prices: &[f64], window: usize) -> Option<f64> {
    if window == 0 || prices.len() < window {
        return None;
    }
    let slice = &prices[prices.len() - window..];
    let mean = slice.iter().sum::<f64>() / window as f64;
    let std = sample_stddev(slice)?;
    if std == 0.0 {
        return Some(0.0);
    }
    Some((slice[window - 1] - mean) / std)
}

pub fn beta(asset_returns: &[f64], market_returns: &[f64]) -> Option<f64> {
    if asset_returns.len() != market_returns.len() || asset_returns.len() < 2 {
        return None;
    }
    let market_var = sample_variance(market_returns)?;
    if market_var == 0.0 {
        return None;
    }
    Some(sample_covariance(asset_returns, market_returns)? / market_var)
}

pub fn sample_stddev(values: &[f64]) -> Option<f64> {
    sample_variance(values).map(f64::sqrt)
}

pub fn sample_variance(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let sum_sq = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>();
    Some(sum_sq / (values.len() as f64 - 1.0))
}

fn sample_covariance(xs: &[f64], ys: &[f64]) -> Option<f64> {
    if xs.len() != ys.len() || xs.len() < 2 {
        return None;
    }
    let x_mean = xs.iter().sum::<f64>() / xs.len() as f64;
    let y_mean = ys.iter().sum::<f64>() / ys.len() as f64;
    let covariance = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (x - x_mean) * (y - y_mean))
        .sum::<f64>()
        / (xs.len() as f64 - 1.0);
    Some(covariance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_simple_returns() {
        let result = simple_returns(&[100.0, 110.0, 99.0]);
        assert!((result[0] - 0.1).abs() < 1e-12);
        assert!((result[1] - -0.1).abs() < 1e-12);
    }

    #[test]
    fn computes_rsi_bounds() {
        let prices = (1..=30).map(|v| v as f64).collect::<Vec<_>>();
        assert_eq!(rsi(&prices, 14), Some(100.0));
    }

    #[test]
    fn computes_max_drawdown() {
        let result = max_drawdown(&[100.0, 120.0, 90.0, 110.0]).unwrap();
        assert!((result - -0.25).abs() < 1e-12);
    }
}
