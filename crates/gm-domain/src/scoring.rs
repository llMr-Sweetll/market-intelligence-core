use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    context::{MacroContext, clamp, round2, round4},
    features::FeatureVector,
    rules::{RuleRegistry, RuleResult},
    stochastic::PredictionRecord,
    taxonomy::{EventClass, classify},
    types::{Action, Decision, NormalizedEvent},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreOutput {
    pub score: f64,
    pub matched_rules: Vec<String>,
    pub event_class: EventClass,
    pub rule_results: Vec<RuleResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DecisionThresholds {
    pub buy_threshold: f64,
    pub sell_threshold: f64,
    pub max_position_size: f64,
    pub min_position_size: f64,
    pub portfolio_capital: f64,
}

impl Default for DecisionThresholds {
    fn default() -> Self {
        Self {
            buy_threshold: 0.70,
            sell_threshold: -0.70,
            max_position_size: 0.05,
            min_position_size: 0.02,
            portfolio_capital: 1_000_000.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AsOfFacts {
    pub macro_context: MacroContext,
    pub entry_price: Option<f64>,
    pub exchange: Option<String>,
    pub features: Option<FeatureVector>,
    pub prediction: Option<PredictionRecord>,
    pub kg_modifier: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionInput {
    pub event: NormalizedEvent,
    pub score: ScoreOutput,
    pub facts: AsOfFacts,
    pub thresholds: DecisionThresholds,
}

pub fn score_event(event: &NormalizedEvent, registry: &RuleRegistry) -> ScoreOutput {
    let rule_results = registry.evaluate_matched(event);
    let score = round4(clamp(
        rule_results.iter().map(|r| r.contribution).sum::<f64>(),
        -1.0,
        1.0,
    ));
    let matched_rules = rule_results
        .iter()
        .map(|result| result.rule_id.clone())
        .collect::<Vec<_>>();

    ScoreOutput {
        score,
        matched_rules,
        event_class: classify(event),
        rule_results,
    }
}

pub fn decide(input: DecisionInput) -> Decision {
    let feature_component = input
        .facts
        .features
        .as_ref()
        .map(feature_signal)
        .unwrap_or(0.0);
    let prediction_component = input
        .facts
        .prediction
        .as_ref()
        .map(prediction_signal)
        .unwrap_or(0.0);
    let kg_component = clamp(input.facts.kg_modifier, -0.15, 0.15);
    let combined_score = round4(clamp(
        input.score.score
            + 0.30 * input.facts.macro_context.total_macro_score
            + feature_component
            + prediction_component
            + kg_component,
        -1.0,
        1.0,
    ));

    let intended_action = if combined_score >= input.thresholds.buy_threshold {
        Action::Buy
    } else if combined_score <= input.thresholds.sell_threshold {
        Action::Sell
    } else {
        Action::Hold
    };
    let mut action = intended_action;

    let mut confidence = if action == Action::Hold {
        0.0
    } else {
        combined_score.abs()
    };
    let mut position_size = if action == Action::Hold {
        0.0
    } else if confidence >= 0.85 {
        input.thresholds.max_position_size
    } else {
        input.thresholds.min_position_size
    };

    let mut quantity = None;
    let mut target_price = None;
    let mut stop_loss = None;
    let mut timing = None;
    let mut execution_ready = false;

    let mut blocked_by_missing_price = false;

    if matches!(action, Action::Buy | Action::Sell) {
        if let Some(entry_price) = input.facts.entry_price {
            let quantity_value =
                ((input.thresholds.portfolio_capital * position_size) / entry_price).floor();
            quantity = (quantity_value > 0.0).then_some(quantity_value as u64);
            let (target, stop) = target_stop(action, entry_price, input.facts.features.as_ref());
            target_price = Some(target);
            stop_loss = Some(stop);
            timing = Some("Immediate market order".to_string());
            execution_ready = quantity.is_some();
        } else {
            action = Action::Hold;
            confidence = 0.0;
            position_size = 0.0;
            blocked_by_missing_price = true;
        }
    }

    let reasons = serde_json::to_value(&input.score.rule_results).unwrap_or(serde_json::json!([]));
    let thesis = thesis(ThesisInput {
        action,
        score: combined_score,
        symbol: input.event.symbol.as_deref(),
        quantity,
        entry_price: input.facts.entry_price,
        target_price,
        stop_loss,
        matched_rules: &input.score.matched_rules,
        blocked_by_missing_price,
    });
    let decision_id = deterministic_decision_id(
        &input.event,
        action,
        combined_score,
        input.facts.entry_price,
        &input.score.matched_rules,
    );

    Decision {
        decision_id,
        parent_event_id: input.event.event_id,
        parent_event_version: input.event.version,
        action,
        total_score: combined_score,
        confidence: round2(confidence),
        position_size,
        quantity,
        entry_price: input.facts.entry_price.map(round2),
        target_price,
        stop_loss,
        timing,
        exchange: input.facts.exchange,
        symbol: input.event.symbol,
        sector: input.event.sector,
        thesis,
        reasons,
        execution_ready,
    }
}

fn feature_signal(features: &FeatureVector) -> f64 {
    let mut signal = 0.0;
    let momentum_values = [features.momentum_1m, features.momentum_3m]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if !momentum_values.is_empty() {
        signal += momentum_values.iter().sum::<f64>() / momentum_values.len() as f64 * 2.0;
    }

    if let Some(rsi) = features.rsi_14 {
        if rsi < 30.0 {
            signal += 0.05;
        } else if rsi > 70.0 {
            signal -= 0.05;
        }
    }

    if let Some(zscore) = features.zscore_20 {
        if zscore < -2.0 {
            signal += 0.05;
        } else if zscore > 2.0 {
            signal -= 0.05;
        }
    }

    clamp(signal, -0.20, 0.20)
}

fn prediction_signal(prediction: &PredictionRecord) -> f64 {
    clamp(prediction.expected_return * 3.0, -0.20, 0.20)
}

fn target_stop(action: Action, entry_price: f64, features: Option<&FeatureVector>) -> (f64, f64) {
    let atr = features.and_then(|f| f.atr_14).unwrap_or(0.0);
    let target_distance = (2.0 * atr).max(entry_price * 0.03);
    let stop_distance = (1.2 * atr).max(entry_price * 0.02);

    match action {
        Action::Buy => (
            round2(entry_price + target_distance),
            round2(entry_price - stop_distance),
        ),
        Action::Sell => (
            round2(entry_price - target_distance),
            round2(entry_price + stop_distance),
        ),
        Action::Hold => (entry_price, entry_price),
    }
}

struct ThesisInput<'a> {
    action: Action,
    score: f64,
    symbol: Option<&'a str>,
    quantity: Option<u64>,
    entry_price: Option<f64>,
    target_price: Option<f64>,
    stop_loss: Option<f64>,
    matched_rules: &'a [String],
    blocked_by_missing_price: bool,
}

fn thesis(input: ThesisInput<'_>) -> String {
    let symbol_text = input.symbol.unwrap_or("instrument");
    if input.blocked_by_missing_price {
        return format!(
            "Signal crossed the action threshold for {symbol_text}, but no as-of price was supplied; holding until an executable price fact is available."
        );
    }

    let rules_text = if input.matched_rules.is_empty() {
        "no matched rules".to_string()
    } else {
        input.matched_rules.join(", ")
    };

    if let (Some(quantity), Some(entry), Some(target), Some(stop)) = (
        input.quantity,
        input.entry_price,
        input.target_price,
        input.stop_loss,
    ) {
        return format!(
            "{} {symbol_text}: {quantity} shares @ {:.2}. Target {:.2}, stop {:.2}. Triggered by {rules_text}. Total score {:.2}.",
            input.action.as_str(),
            entry,
            target,
            stop,
            input.score
        );
    }

    format!(
        "{} {symbol_text}. Triggered by {rules_text}. Total score {:.2}.",
        input.action.as_str(),
        input.score
    )
}

fn deterministic_decision_id(
    event: &NormalizedEvent,
    action: Action,
    score: f64,
    entry_price: Option<f64>,
    matched_rules: &[String],
) -> String {
    let seed = format!(
        "{}:{}:{}:{:.4}:{:?}:{}",
        event.event_id,
        event.version,
        action.as_str(),
        score,
        entry_price.map(round2),
        matched_rules.join(",")
    );
    Uuid::new_v5(&Uuid::NAMESPACE_URL, seed.as_bytes()).to_string()
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};

    use super::*;
    use crate::{features::FeatureVector, rules::RuleRegistry};

    fn event(headline: &str) -> NormalizedEvent {
        NormalizedEvent {
            event_id: "norm-1".to_string(),
            version: 1,
            causal_parent_id: Some("raw-1".to_string()),
            event_type: None,
            headline: headline.to_string(),
            body: "Profit rose and revenue grew higher than expected.".to_string(),
            occurred_at: Utc::now(),
            symbol: Some("RELIANCE".to_string()),
            sector: Some("Oil & Gas".to_string()),
            source: Some("NSE".to_string()),
            region: Some("IN".to_string()),
            impact_level: None,
            impact_category: None,
        }
    }

    #[test]
    fn score_event_is_deterministic() {
        let registry = RuleRegistry::builtin();
        let event = event("Quarterly earnings beat estimates");
        let one = score_event(&event, &registry);
        let two = score_event(&event, &registry);
        assert_eq!(one, two);
        assert!(one.score > 0.7);
    }

    #[test]
    fn decide_does_not_trade_without_price_fact() {
        let registry = RuleRegistry::builtin();
        let event = event("Quarterly earnings beat estimates");
        let score = score_event(&event, &registry);
        let decision = decide(DecisionInput {
            event,
            score,
            facts: AsOfFacts::default(),
            thresholds: DecisionThresholds::default(),
        });

        assert_eq!(decision.action, Action::Hold);
        assert!(!decision.execution_ready);
    }

    #[test]
    fn earnings_beat_with_price_is_actionable() {
        let registry = RuleRegistry::builtin();
        let event = event("Quarterly earnings beat estimates");
        let score = score_event(&event, &registry);

        let decision = decide(DecisionInput {
            event,
            score,
            facts: AsOfFacts {
                entry_price: Some(1000.0),
                exchange: Some("NSE".to_string()),
                ..AsOfFacts::default()
            },
            thresholds: DecisionThresholds::default(),
        });

        assert_eq!(decision.action, Action::Buy);
        assert_eq!(decision.quantity, Some(20));
        assert!(decision.execution_ready);
    }

    #[test]
    fn decide_uses_injected_price_and_atr() {
        let registry = RuleRegistry::builtin();
        let event = event("Quarterly earnings beat estimates");
        let score = score_event(&event, &registry);
        let features = FeatureVector {
            symbol: "RELIANCE".to_string(),
            as_of: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            n_bars: 30,
            close: Some(1000.0),
            momentum_1m: Some(0.05),
            momentum_3m: None,
            momentum_6m: None,
            rsi_14: Some(55.0),
            atr_14: Some(20.0),
            sma_20: None,
            sma_50: None,
            ema_20: None,
            annualized_vol: None,
            max_drawdown: None,
            zscore_20: None,
            adv_20: None,
        };

        let decision = decide(DecisionInput {
            event,
            score,
            facts: AsOfFacts {
                entry_price: Some(1000.0),
                exchange: Some("NSE".to_string()),
                features: Some(features),
                ..AsOfFacts::default()
            },
            thresholds: DecisionThresholds::default(),
        });

        assert_eq!(decision.action, Action::Buy);
        assert_eq!(decision.quantity, Some(20));
        assert_eq!(decision.target_price, Some(1040.0));
        assert_eq!(decision.stop_loss, Some(976.0));
        assert!(decision.execution_ready);
    }

    #[test]
    fn decide_is_deterministic_including_id() {
        let registry = RuleRegistry::builtin();
        let event = event("Quarterly earnings beat estimates");
        let score = score_event(&event, &registry);
        let input = DecisionInput {
            event,
            score,
            facts: AsOfFacts {
                entry_price: Some(1000.0),
                exchange: Some("NSE".to_string()),
                ..AsOfFacts::default()
            },
            thresholds: DecisionThresholds::default(),
        };

        let one = decide(input.clone());
        let two = decide(input);

        assert_eq!(one, two);
        assert_eq!(one.action, Action::Buy);
    }

    #[test]
    fn severe_negative_event_with_price_is_actionable_sell() {
        let registry = RuleRegistry::builtin();
        let mut event = event("Insider trading investigation");
        event.body = "Front running and price manipulation investigation opened.".to_string();
        let score = score_event(&event, &registry);

        let decision = decide(DecisionInput {
            event,
            score,
            facts: AsOfFacts {
                entry_price: Some(1000.0),
                exchange: Some("NSE".to_string()),
                ..AsOfFacts::default()
            },
            thresholds: DecisionThresholds::default(),
        });

        assert_eq!(decision.action, Action::Sell);
        assert_eq!(decision.quantity, Some(20));
        assert!(decision.execution_ready);
    }
}
