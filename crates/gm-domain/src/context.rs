use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MacroInputs {
    pub sp500_change_pct: f64,
    pub nasdaq_change_pct: f64,
    pub brent_change_pct: f64,
    pub usd_inr_change_pct: f64,
    pub fii_net_cr: f64,
    pub gold_change_pct: f64,
}

impl Default for MacroInputs {
    fn default() -> Self {
        Self {
            sp500_change_pct: 0.0,
            nasdaq_change_pct: 0.0,
            brent_change_pct: 0.0,
            usd_inr_change_pct: 0.0,
            fii_net_cr: 0.0,
            gold_change_pct: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MacroContext {
    pub sp500_futures_change: f64,
    pub nasdaq_futures_change: f64,
    pub brent_crude_change: f64,
    pub usd_inr_change: f64,
    pub fii_net_flow: f64,
    pub gold_change: f64,
    pub total_macro_score: f64,
}

impl Default for MacroContext {
    fn default() -> Self {
        Self {
            sp500_futures_change: 0.0,
            nasdaq_futures_change: 0.0,
            brent_crude_change: 0.0,
            usd_inr_change: 0.0,
            fii_net_flow: 0.0,
            gold_change: 0.0,
            total_macro_score: 0.0,
        }
    }
}

pub fn build_macro_context(sector: &str, inputs: MacroInputs) -> MacroContext {
    let mut context = MacroContext {
        sp500_futures_change: normalize_pct(inputs.sp500_change_pct),
        nasdaq_futures_change: normalize_pct(inputs.nasdaq_change_pct),
        brent_crude_change: normalize_pct(inputs.brent_change_pct),
        usd_inr_change: normalize_pct(inputs.usd_inr_change_pct),
        fii_net_flow: normalize_fii_flow(inputs.fii_net_cr),
        gold_change: normalize_pct(inputs.gold_change_pct),
        total_macro_score: 0.0,
    };

    context.total_macro_score = round4(
        sector_weights(sector)
            .iter()
            .map(|(factor, weight)| signal_value(context, factor) * weight)
            .sum::<f64>(),
    );
    context
}

pub fn normalize_pct(value: f64) -> f64 {
    clamp(value / 5.0, -1.0, 1.0)
}

pub fn normalize_fii_flow(value: f64) -> f64 {
    clamp(value / 2000.0, -1.0, 1.0)
}

fn signal_value(context: MacroContext, factor: &str) -> f64 {
    match factor {
        "sp500_futures_change" => context.sp500_futures_change,
        "nasdaq_futures_change" => context.nasdaq_futures_change,
        "brent_crude_change" => context.brent_crude_change,
        "usd_inr_change" => context.usd_inr_change,
        "fii_net_flow" => context.fii_net_flow,
        "gold_change" => context.gold_change,
        _ => 0.0,
    }
}

fn sector_weights(sector: &str) -> &'static [(&'static str, f64)] {
    match sector {
        "Oil & Gas" => &[
            ("brent_crude_change", 0.6),
            ("sp500_futures_change", 0.15),
            ("fii_net_flow", 0.15),
            ("usd_inr_change", 0.1),
        ],
        "IT" => &[
            ("usd_inr_change", 0.5),
            ("nasdaq_futures_change", 0.25),
            ("sp500_futures_change", 0.15),
            ("fii_net_flow", 0.1),
        ],
        "Banking" => &[
            ("sp500_futures_change", 0.2),
            ("fii_net_flow", 0.3),
            ("usd_inr_change", 0.2),
            ("brent_crude_change", 0.1),
            ("gold_change", 0.2),
        ],
        "Auto" => &[
            ("usd_inr_change", 0.25),
            ("brent_crude_change", 0.25),
            ("sp500_futures_change", 0.2),
            ("fii_net_flow", 0.15),
            ("nasdaq_futures_change", 0.15),
        ],
        "Pharma" => &[
            ("usd_inr_change", 0.4),
            ("sp500_futures_change", 0.2),
            ("nasdaq_futures_change", 0.2),
            ("fii_net_flow", 0.2),
        ],
        "Metals" | "FMCG" | "Infrastructure" => &[
            ("brent_crude_change", 0.2),
            ("usd_inr_change", 0.2),
            ("sp500_futures_change", 0.2),
            ("fii_net_flow", 0.2),
            ("gold_change", 0.2),
        ],
        "Realty" => &[
            ("sp500_futures_change", 0.2),
            ("fii_net_flow", 0.3),
            ("usd_inr_change", 0.25),
            ("brent_crude_change", 0.15),
            ("gold_change", 0.1),
        ],
        "Telecom" => &[
            ("sp500_futures_change", 0.25),
            ("fii_net_flow", 0.25),
            ("usd_inr_change", 0.25),
            ("nasdaq_futures_change", 0.25),
        ],
        _ => &[
            ("sp500_futures_change", 0.3),
            ("nasdaq_futures_change", 0.2),
            ("fii_net_flow", 0.2),
            ("usd_inr_change", 0.15),
            ("brent_crude_change", 0.15),
        ],
    }
}

pub(crate) fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

pub(crate) fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub(crate) fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_fii_flow() {
        assert_eq!(normalize_fii_flow(2500.0), 1.0);
        assert_eq!(normalize_fii_flow(-1000.0), -0.5);
    }

    #[test]
    fn computes_sector_weighted_macro_score() {
        let context = build_macro_context(
            "IT",
            MacroInputs {
                usd_inr_change_pct: 2.5,
                nasdaq_change_pct: 1.0,
                ..MacroInputs::default()
            },
        );

        assert_eq!(context.usd_inr_change, 0.5);
        assert_eq!(context.total_macro_score, 0.3);
    }
}
