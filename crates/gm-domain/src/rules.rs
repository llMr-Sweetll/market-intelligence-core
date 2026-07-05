use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::types::NormalizedEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_id: String,
    pub matched: bool,
    pub weight: f64,
    pub confidence: f64,
    pub contribution: f64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub rule_id: String,
    pub weight: f64,
    pub confidence: f64,
    pub keywords: Vec<String>,
}

impl RuleDefinition {
    pub fn keyword(
        rule_id: impl Into<String>,
        weight: f64,
        confidence: f64,
        keywords: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        assert!((-1.0..=1.0).contains(&weight), "rule weight out of range");
        assert!(
            (0.0..=1.0).contains(&confidence),
            "rule confidence out of range"
        );
        Self {
            rule_id: rule_id.into(),
            weight,
            confidence,
            keywords: keywords
                .into_iter()
                .map(|k| k.into().to_lowercase())
                .collect(),
        }
    }

    pub fn evaluate(&self, event: &NormalizedEvent) -> RuleResult {
        let text = event.searchable_text();
        let matched_keywords = self
            .keywords
            .iter()
            .filter(|keyword| keyword_matches(&text, keyword))
            .cloned()
            .collect::<Vec<_>>();
        let matched = !matched_keywords.is_empty();
        let contribution = if matched {
            self.weight * self.confidence
        } else {
            0.0
        };

        RuleResult {
            rule_id: self.rule_id.clone(),
            matched,
            weight: self.weight,
            confidence: self.confidence,
            contribution,
            reason: if matched {
                format!("Matched keyword(s): {}", matched_keywords.join(", "))
            } else {
                "No keywords matched".to_string()
            },
        }
    }
}

fn keyword_matches(text: &str, keyword: &str) -> bool {
    if keyword.contains(' ') || keyword.contains('-') {
        return text.contains(keyword);
    }

    let pattern = format!(r"\b{}\b", regex::escape(keyword));
    Regex::new(&pattern)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

#[derive(Debug, Clone)]
pub struct RuleRegistry {
    rules: Vec<RuleDefinition>,
}

impl RuleRegistry {
    pub fn new(rules: Vec<RuleDefinition>) -> Self {
        Self { rules }
    }

    pub fn builtin() -> Self {
        Self::new(builtin_rules())
    }

    pub fn rules(&self) -> &[RuleDefinition] {
        &self.rules
    }

    pub fn evaluate(&self, event: &NormalizedEvent) -> Vec<RuleResult> {
        self.rules.iter().map(|rule| rule.evaluate(event)).collect()
    }

    pub fn evaluate_matched(&self, event: &NormalizedEvent) -> Vec<RuleResult> {
        self.evaluate(event)
            .into_iter()
            .filter(|result| result.matched)
            .collect()
    }
}

pub fn builtin_rules() -> Vec<RuleDefinition> {
    vec![
        RuleDefinition::keyword(
            "EARNINGS_BEAT",
            0.8,
            0.9,
            [
                "earnings beat",
                "beat estimates",
                "earnings surprise",
                "profit rose",
                "revenue grew",
                "higher than expected",
                "better than expected",
                "outperformed",
                "exceeded expectations",
            ],
        ),
        RuleDefinition::keyword(
            "PROMOTER_STAKE_INCREASE",
            0.9,
            0.95,
            [
                "promoter stake",
                "promoter buying",
                "promoter acquired",
                "increased stake",
                "promoter holding",
                "promoter purchase",
            ],
        ),
        RuleDefinition::keyword(
            "BOARD_MEETING",
            0.2,
            0.7,
            [
                "board meeting",
                "board of directors",
                "outcome of board meeting",
                "consider buyback",
                "consider dividend",
                "consider bonus",
            ],
        ),
        RuleDefinition::keyword(
            "COMPLIANCE_NOTICE",
            -0.7,
            0.85,
            [
                "compliance notice",
                "regulatory warning",
                "show cause notice",
                "sebi penalty",
                "sebi fine",
                "violation",
                "non-compliance",
                "regulatory action",
                "enforcement action",
            ],
        ),
        RuleDefinition::keyword(
            "DIVIDEND_DECLARATION",
            0.4,
            0.85,
            [
                "dividend declared",
                "interim dividend",
                "final dividend",
                "special dividend",
                "dividend per share",
                "dividend announcement",
            ],
        ),
        RuleDefinition::keyword(
            "SHARE_BUYBACK",
            0.6,
            0.9,
            [
                "buyback",
                "share buyback",
                "buy-back",
                "repurchase of shares",
                "consider buyback",
                "approved buyback",
            ],
        ),
        RuleDefinition::keyword(
            "MERGER_ACQUISITION",
            0.5,
            0.75,
            [
                "merger",
                "acquisition",
                "acquire",
                "amalgamation",
                "scheme of arrangement",
                "demerger",
                "takeover",
            ],
        ),
        RuleDefinition::keyword(
            "INSIDER_TRADING",
            -0.8,
            0.9,
            [
                "insider trading",
                "unpublished price sensitive information",
                "upsi",
                "price manipulation",
                "front running",
            ],
        ),
        RuleDefinition::keyword(
            "CREDIT_RATING_UPGRADE",
            0.5,
            0.85,
            [
                "credit rating upgraded",
                "rating upgraded",
                "outlook revised to positive",
                "credit enhancement",
                "rating improved",
            ],
        ),
        RuleDefinition::keyword(
            "CREDIT_RATING_DOWNGRADE",
            -0.6,
            0.85,
            [
                "credit rating downgraded",
                "rating downgraded",
                "outlook revised to negative",
                "credit watch negative",
                "rating withdrawn",
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn event(headline: &str, body: &str) -> NormalizedEvent {
        NormalizedEvent {
            event_id: "norm-1".to_string(),
            version: 1,
            causal_parent_id: Some("raw-1".to_string()),
            event_type: None,
            headline: headline.to_string(),
            body: body.to_string(),
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
    fn keyword_rule_uses_word_boundaries() {
        let rule = RuleDefinition::keyword("WAR", -0.5, 1.0, ["war"]);
        assert!(!rule.evaluate(&event("Company warns investors", "")).matched);
        assert!(rule.evaluate(&event("War risk affects oil", "")).matched);
    }

    #[test]
    fn builtin_buyback_matches() {
        let registry = RuleRegistry::builtin();
        let results = registry.evaluate_matched(&event(
            "Board approves buyback",
            "The company approved buyback of equity shares.",
        ));
        assert!(results.iter().any(|r| r.rule_id == "SHARE_BUYBACK"));
    }
}
