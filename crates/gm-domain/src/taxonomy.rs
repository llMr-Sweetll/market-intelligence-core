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
    GlobalMovement,
    PoliticalMeeting,
    CompanyStructure,
    Regulation,
    PolicyChange,
    Conflict,
    MedicalClassification,
    Filing,
    MacroEvent,
    MarketEvent,
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
        (
            EventClass::GlobalMovement,
            &[
                "protest",
                "protests",
                "strike",
                "strikes",
                "migration",
                "global movement",
                "supply-chain disruption",
                "supply chain disruption",
                "shipping disruption",
                "port disruption",
                "civil unrest",
            ],
        ),
        (
            EventClass::PoliticalMeeting,
            &[
                "summit",
                "bilateral meeting",
                "political reunion",
                "political meeting",
                "diplomatic meeting",
                "alliance meeting",
                "leaders met",
                "trade talks",
                "peace talks",
            ],
        ),
        (
            EventClass::CompanyStructure,
            &[
                "restructuring",
                "subsidiary",
                "holding company",
                "ownership structure",
                "shareholding structure",
                "business transfer",
                "spin-off",
                "spinoff",
                "entity structure",
            ],
        ),
        (
            EventClass::Regulation,
            &[
                "new regulation",
                "regulatory framework",
                "market regulation",
                "sector rule",
                "exchange circular",
                "compliance rule",
                "licensing requirement",
                "prudential norm",
            ],
        ),
        (
            EventClass::PolicyChange,
            &[
                "policy change",
                "policy statement",
                "rate decision",
                "tax policy",
                "trade restriction",
                "tariff",
                "sanctions",
                "liquidity stance",
                "public-health policy",
                "environmental policy",
            ],
        ),
        (
            EventClass::Conflict,
            &[
                "conflict",
                "military escalation",
                "border clash",
                "ceasefire",
                "war",
                "sanctioned actor",
                "shipping lane risk",
                "geopolitical tension",
                "armed group",
            ],
        ),
        (
            EventClass::MedicalClassification,
            &[
                "icd-11",
                "icd 11",
                "medical classification",
                "disease classification",
                "public-health alert",
                "drug approval",
                "therapy classification",
                "reimbursement code",
                "health classification",
            ],
        ),
        (
            EventClass::Filing,
            &[
                "filing",
                "annual report",
                "quarterly filing",
                "regulatory filing",
                "exchange filing",
                "sec filing",
                "edgar",
                "prospectus",
                "disclosure document",
            ],
        ),
        (
            EventClass::MacroEvent,
            &[
                "inflation",
                "gdp",
                "employment data",
                "central bank",
                "monetary policy",
                "fiscal deficit",
                "current account",
                "fii flow",
                "currency reserves",
            ],
        ),
        (
            EventClass::MarketEvent,
            &[
                "price shock",
                "volume shock",
                "volatility",
                "liquidity drop",
                "currency movement",
                "commodity movement",
                "index rebalancing",
                "credit spread",
                "market halt",
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
        let event = event("SEBI show cause notice", "Penalty proceedings initiated");

        assert_eq!(classify(&event), EventClass::RegulatoryAction);
    }

    #[test]
    fn classifies_release_ontology_events() {
        let cases = [
            (
                "Port strikes disrupt global supply chain",
                "Civil unrest and shipping disruption affect commodity flows.",
                EventClass::GlobalMovement,
            ),
            (
                "Regional alliance summit begins trade talks",
                "Leaders met for a diplomatic meeting on tariff policy.",
                EventClass::PoliticalMeeting,
            ),
            (
                "Company announces subsidiary restructuring",
                "Ownership structure and holding company reporting will change.",
                EventClass::CompanyStructure,
            ),
            (
                "Exchange publishes new market regulation",
                "The exchange circular changes a sector rule for brokers.",
                EventClass::Regulation,
            ),
            (
                "Central bank policy statement updates liquidity stance",
                "Rate decision and tax policy expectations moved markets.",
                EventClass::PolicyChange,
            ),
            (
                "Border conflict escalates near shipping lane",
                "Military escalation raised commodity movement risk.",
                EventClass::Conflict,
            ),
            (
                "ICD-11 medical classification update",
                "Therapy classification changes reimbursement code exposure.",
                EventClass::MedicalClassification,
            ),
            (
                "Company files annual report",
                "The exchange filing updates risk factors.",
                EventClass::Filing,
            ),
            (
                "Inflation and GDP data surprise markets",
                "Central bank expectations changed after employment data.",
                EventClass::MacroEvent,
            ),
            (
                "Index rebalancing causes volume shock",
                "Volatility rose after a liquidity drop.",
                EventClass::MarketEvent,
            ),
        ];

        for (headline, body, expected) in cases {
            assert_eq!(classify(&event(headline, body)), expected, "{headline}");
        }
    }

    fn event(headline: &str, body: &str) -> NormalizedEvent {
        NormalizedEvent {
            event_id: "e".to_string(),
            version: 1,
            causal_parent_id: None,
            event_type: None,
            headline: headline.to_string(),
            body: body.to_string(),
            occurred_at: Utc::now(),
            symbol: None,
            sector: None,
            source: None,
            region: None,
            impact_level: None,
            impact_category: None,
        }
    }
}
