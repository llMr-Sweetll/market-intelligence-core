use serde::{Deserialize, Serialize};

use crate::types::NormalizedEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventClass {
    Earnings,
    PromoterAction,
    CorporateDisclosure,
    RegulatoryAction,
    Governance,
    Risk,
    CorporateAction,
    MergerAcquisition,
    General,
}

pub fn classify(event: &NormalizedEvent) -> EventClass {
    let text = event.searchable_text();
    let mut best = (EventClass::General, 0usize);

    for (class, keywords) in classification_map() {
        let score = keywords
            .iter()
            .filter(|keyword| text.contains(**keyword))
            .count();
        if score > best.1 {
            best = (*class, score);
        }
    }

    best.0
}

fn classification_map() -> &'static [(EventClass, &'static [&'static str])] {
    &[
        (
            EventClass::Earnings,
            &[
                "earnings",
                "profit",
                "revenue",
                "loss",
                "quarterly results",
                "annual results",
                "financial results",
                "q1",
                "q2",
                "q3",
                "q4",
                "fy",
                "beat estimates",
                "earnings surprise",
            ],
        ),
        (
            EventClass::PromoterAction,
            &[
                "promoter",
                "promoter stake",
                "promoter buying",
                "increased stake",
                "promoter holding",
                "acquisition of shares by promoter",
            ],
        ),
        (
            EventClass::CorporateDisclosure,
            &[
                "dividend",
                "buyback",
                "bonus",
                "split",
                "rights issue",
                "board meeting",
                "outcome of board meeting",
            ],
        ),
        (
            EventClass::RegulatoryAction,
            &[
                "compliance",
                "regulatory",
                "sebi",
                "show cause",
                "penalty",
                "fine",
                "violation",
                "insider trading",
                "price manipulation",
            ],
        ),
        (
            EventClass::Governance,
            &[
                "board of directors",
                "appointment",
                "resignation",
                "auditor",
                "independent director",
                "csr",
                "esg",
            ],
        ),
        (
            EventClass::Risk,
            &[
                "credit rating",
                "downgrade",
                "default",
                "bankruptcy",
                "insolvency",
                "rating watch negative",
                "liquidity risk",
            ],
        ),
        (
            EventClass::CorporateAction,
            &["corporate action", "record date", "ex-date", "agm", "egm"],
        ),
        (
            EventClass::MergerAcquisition,
            &[
                "merger",
                "acquisition",
                "amalgamation",
                "demerger",
                "takeover",
                "scheme of arrangement",
                "slump sale",
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn classifies_regulatory_action() {
        let event = NormalizedEvent {
            event_id: "e".to_string(),
            version: 1,
            causal_parent_id: None,
            event_type: None,
            headline: "SEBI show cause notice".to_string(),
            body: "Penalty proceedings initiated".to_string(),
            occurred_at: Utc::now(),
            symbol: None,
            sector: None,
            source: None,
            region: None,
            impact_level: None,
            impact_category: None,
        };

        assert_eq!(classify(&event), EventClass::RegulatoryAction);
    }
}
