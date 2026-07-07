use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    context::{clamp, round2, round4},
    event_study::{aggregate_car, calibrated_weight, cumulative_abnormal_return},
    scoring::DecisionInput,
    types::Action,
};

pub const DECISION_MODEL_VERSION: &str = "rules-impact-v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EventStudyEvidence {
    pub abnormal_returns: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateAction {
    Buy,
    Sell,
    Hold,
    Paper,
}

impl CandidateAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
            Self::Hold => "HOLD",
            Self::Paper => "PAPER",
        }
    }
}

impl From<Action> for CandidateAction {
    fn from(action: Action) -> Self {
        match action {
            Action::Buy => Self::Buy,
            Action::Sell => Self::Sell,
            Action::Hold => Self::Hold,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityResolution {
    pub symbol: Option<String>,
    pub sector: Option<String>,
    pub region: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_type: String,
    pub label: String,
    pub contribution: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventStudySummary {
    pub sample_count: usize,
    pub cumulative_abnormal_return: Option<f64>,
    pub mean_abnormal_return: Option<f64>,
    pub hit_rate: Option<f64>,
    pub t_stat: Option<f64>,
    pub calibrated_weight: f64,
    pub calibrated_confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactEstimate {
    pub combined_score: f64,
    pub expected_return: f64,
    pub downside: f64,
    pub event_study: Option<EventStudySummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateReport {
    pub name: String,
    pub passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtilityEstimate {
    pub action: CandidateAction,
    pub expected_utility: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelReport {
    pub model_version: String,
    pub input_hash: String,
    pub pipeline: Vec<String>,
    pub entity_resolution: EntityResolution,
    pub evidence: Vec<EvidenceItem>,
    pub impact: ImpactEstimate,
    pub gates: Vec<GateReport>,
    pub utilities: Vec<UtilityEstimate>,
    pub recommended_action: Action,
    pub confidence: f64,
    pub missing_facts: Vec<String>,
    pub summary: String,
}

pub fn input_hash(input: &DecisionInput) -> String {
    let input_bytes = serde_json::to_vec(input).unwrap_or_default();
    Uuid::new_v5(&Uuid::NAMESPACE_URL, &input_bytes).to_string()
}

pub fn build_model_report(input: &DecisionInput, combined_score: f64) -> ModelReport {
    let entity_resolution = resolve_entities(input);
    let event_study = event_study_summary(input);
    let evidence = collect_evidence(input, event_study.as_ref());
    let impact = estimate_impact(input, combined_score, event_study);
    let missing_facts = missing_facts(input);
    let evidence_passed = !evidence.is_empty();
    let price_passed = input.facts.entry_price.is_some();
    let confidence = confidence(
        combined_score,
        entity_resolution.confidence,
        evidence.len(),
        impact.event_study.as_ref(),
    );
    let confidence_passed = confidence >= 0.30;

    let utilities = utilities(&impact, confidence, price_passed, evidence_passed);
    let recommended_action = recommended_action(
        input,
        combined_score,
        price_passed,
        evidence_passed,
        confidence_passed,
    );
    let gates = vec![
        GateReport {
            name: "evidence".to_string(),
            passed: evidence_passed,
            reason: if evidence_passed {
                format!("{} evidence item(s) available", evidence.len())
            } else {
                "no matched rules, prediction, event-study, macro, or relationship evidence"
                    .to_string()
            },
        },
        GateReport {
            name: "price".to_string(),
            passed: price_passed,
            reason: if price_passed {
                "as-of entry price supplied".to_string()
            } else {
                "missing as-of entry price".to_string()
            },
        },
        GateReport {
            name: "confidence".to_string(),
            passed: confidence_passed,
            reason: format!("confidence {:.2}", confidence),
        },
    ];

    ModelReport {
        model_version: DECISION_MODEL_VERSION.to_string(),
        input_hash: input_hash(input),
        pipeline: vec![
            "classify_event".to_string(),
            "resolve_entities".to_string(),
            "collect_evidence".to_string(),
            "estimate_impact".to_string(),
            "apply_risk_liquidity_confidence_gates".to_string(),
            "decide".to_string(),
        ],
        entity_resolution,
        evidence,
        impact,
        gates,
        utilities,
        recommended_action,
        confidence,
        missing_facts,
        summary: summary(input, recommended_action, combined_score, confidence),
    }
}

fn resolve_entities(input: &DecisionInput) -> EntityResolution {
    let symbol = input.event.symbol.clone();
    let sector = input.event.sector.clone();
    let confidence = match (symbol.is_some(), sector.is_some()) {
        (true, true) => 0.95,
        (true, false) => 0.75,
        (false, true) => 0.50,
        (false, false) => 0.20,
    };

    EntityResolution {
        symbol,
        sector,
        region: input.event.region.clone(),
        confidence,
    }
}

fn collect_evidence(
    input: &DecisionInput,
    event_study: Option<&EventStudySummary>,
) -> Vec<EvidenceItem> {
    let mut evidence = input
        .score
        .rule_results
        .iter()
        .map(|result| EvidenceItem {
            evidence_type: "rule".to_string(),
            label: result.rule_id.clone(),
            contribution: round4(result.contribution),
            confidence: result.confidence,
        })
        .collect::<Vec<_>>();

    if input.facts.macro_context.total_macro_score != 0.0 {
        evidence.push(EvidenceItem {
            evidence_type: "macro".to_string(),
            label: "sector_weighted_macro".to_string(),
            contribution: round4(0.30 * input.facts.macro_context.total_macro_score),
            confidence: 0.70,
        });
    }

    if let Some(features) = input.facts.features.as_ref() {
        evidence.push(EvidenceItem {
            evidence_type: "technical".to_string(),
            label: format!("{} feature snapshot", features.symbol),
            contribution: 0.0,
            confidence: if features.n_bars >= 20 { 0.75 } else { 0.40 },
        });
    }

    if let Some(prediction) = input.facts.prediction.as_ref() {
        evidence.push(EvidenceItem {
            evidence_type: "prediction".to_string(),
            label: prediction.model_version.clone(),
            contribution: round4(prediction.expected_return),
            confidence: if prediction.n_bars >= 20 { 0.75 } else { 0.35 },
        });
    }

    if let Some(event_study) = event_study {
        evidence.push(EvidenceItem {
            evidence_type: "event_study".to_string(),
            label: "car_fixture".to_string(),
            contribution: event_study.calibrated_weight,
            confidence: event_study.calibrated_confidence,
        });
    }

    if input.facts.kg_modifier != 0.0 {
        evidence.push(EvidenceItem {
            evidence_type: "relationship".to_string(),
            label: "knowledge_graph_modifier".to_string(),
            contribution: round4(input.facts.kg_modifier),
            confidence: 0.60,
        });
    }

    evidence
}

fn event_study_summary(input: &DecisionInput) -> Option<EventStudySummary> {
    let evidence = input.facts.event_study.as_ref()?;
    if evidence.abnormal_returns.is_empty() {
        return None;
    }

    let stats = aggregate_car(&evidence.abnormal_returns);
    let car = cumulative_abnormal_return(&evidence.abnormal_returns);
    let mean = stats.mean_abnormal_return.unwrap_or(0.0);
    let (weight, confidence) = calibrated_weight(mean, stats.n, stats.t_stat);

    Some(EventStudySummary {
        sample_count: stats.n,
        cumulative_abnormal_return: Some(round4(car)),
        mean_abnormal_return: stats.mean_abnormal_return.map(round4),
        hit_rate: stats.hit_rate.map(round4),
        t_stat: stats.t_stat.map(round4),
        calibrated_weight: round4(weight),
        calibrated_confidence: round4(confidence),
    })
}

fn estimate_impact(
    input: &DecisionInput,
    combined_score: f64,
    event_study: Option<EventStudySummary>,
) -> ImpactEstimate {
    let event_study_mean = event_study
        .as_ref()
        .and_then(|summary| summary.mean_abnormal_return);
    let expected_return = input
        .facts
        .prediction
        .as_ref()
        .map(|prediction| prediction.expected_return)
        .or(event_study_mean)
        .unwrap_or(combined_score * 0.03);
    let downside = input
        .facts
        .prediction
        .as_ref()
        .and_then(|prediction| {
            prediction
                .quantiles
                .get("5")
                .and_then(serde_json::Value::as_f64)
        })
        .unwrap_or_else(|| -0.02 - expected_return.abs() * 0.50);

    ImpactEstimate {
        combined_score,
        expected_return: round4(clamp(expected_return, -0.50, 0.50)),
        downside: round4(clamp(downside, -0.50, 0.0)),
        event_study,
    }
}

fn confidence(
    combined_score: f64,
    entity_confidence: f64,
    evidence_count: usize,
    event_study: Option<&EventStudySummary>,
) -> f64 {
    if evidence_count == 0 {
        return 0.0;
    }
    let evidence_factor = (evidence_count as f64 / 4.0).min(1.0);
    let event_study_factor = event_study
        .map(|summary| summary.calibrated_confidence * 0.10)
        .unwrap_or(0.0);
    round2(clamp(
        combined_score.abs() * 0.75
            + entity_confidence * 0.10
            + evidence_factor * 0.15
            + event_study_factor,
        0.0,
        1.0,
    ))
}

fn utilities(
    impact: &ImpactEstimate,
    confidence: f64,
    price_passed: bool,
    evidence_passed: bool,
) -> Vec<UtilityEstimate> {
    let executable_multiplier = if price_passed && evidence_passed {
        1.0
    } else {
        0.0
    };
    let downside_penalty = impact.downside.abs() * 0.40;
    let buy = executable_multiplier * (impact.expected_return * confidence - downside_penalty);
    let sell = executable_multiplier * (-impact.expected_return * confidence - downside_penalty);
    let paper = if evidence_passed {
        buy.max(sell) * 0.75
    } else {
        -0.01
    };

    vec![
        UtilityEstimate {
            action: CandidateAction::Buy,
            expected_utility: round4(clamp(buy, -1.0, 1.0)),
        },
        UtilityEstimate {
            action: CandidateAction::Sell,
            expected_utility: round4(clamp(sell, -1.0, 1.0)),
        },
        UtilityEstimate {
            action: CandidateAction::Hold,
            expected_utility: 0.0,
        },
        UtilityEstimate {
            action: CandidateAction::Paper,
            expected_utility: round4(clamp(paper, -1.0, 1.0)),
        },
    ]
}

fn recommended_action(
    input: &DecisionInput,
    combined_score: f64,
    price_passed: bool,
    evidence_passed: bool,
    confidence_passed: bool,
) -> Action {
    if !evidence_passed || !confidence_passed || !price_passed {
        return Action::Hold;
    }

    if combined_score >= input.thresholds.buy_threshold {
        Action::Buy
    } else if combined_score <= input.thresholds.sell_threshold {
        Action::Sell
    } else {
        Action::Hold
    }
}

fn missing_facts(input: &DecisionInput) -> Vec<String> {
    let mut missing = Vec::new();
    if input.event.symbol.is_none() {
        missing.push("symbol".to_string());
    }
    if input.event.sector.is_none() {
        missing.push("sector".to_string());
    }
    if input.facts.entry_price.is_none() {
        missing.push("entry_price".to_string());
    }
    if input.score.rule_results.is_empty()
        && input.facts.prediction.is_none()
        && input.facts.event_study.is_none()
    {
        missing.push("evidence".to_string());
    }
    missing
}

fn summary(input: &DecisionInput, action: Action, combined_score: f64, confidence: f64) -> String {
    format!(
        "{} via {} for {}. Event class {:?}, score {:.2}, confidence {:.2}.",
        action.as_str(),
        DECISION_MODEL_VERSION,
        input
            .event
            .symbol
            .as_deref()
            .unwrap_or("unresolved instrument"),
        input.score.event_class,
        combined_score,
        confidence
    )
}
