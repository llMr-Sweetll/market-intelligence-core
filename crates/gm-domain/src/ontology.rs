use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::taxonomy::EventClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Company,
    Instrument,
    Country,
    Regulator,
    DiseaseClassification,
    Commodity,
    Sector,
    Index,
    Broker,
    Source,
    Exchange,
    Region,
    PolicyBody,
    ConflictActor,
    PoliticalOrganization,
    Currency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    IssuesInstrument,
    BelongsToSector,
    ListedOnIndex,
    GovernedByRegulator,
    PolicyAffectsSector,
    ConflictAffectsCommodity,
    CommodityAffectsCompanyMargin,
    MedicalClassificationAffectsSector,
    SourceReportsEvent,
    BrokerProvidesPaperExecution,
    CountryHostsCompany,
    EventCorroboratesEvent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,
    pub source_id: Option<String>,
    pub url: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeEntity {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub label: String,
    pub aliases: Vec<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeRelationship {
    pub relationship_id: String,
    pub subject_id: String,
    pub relationship_type: RelationshipType,
    pub object_id: String,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_to: Option<DateTime<Utc>>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OntologyFixture {
    pub event_classes: Vec<EventClass>,
    pub entities: Vec<KnowledgeEntity>,
    pub relationships: Vec<KnowledgeRelationship>,
}

pub fn mv_event_classes() -> &'static [EventClass] {
    &[
        EventClass::Earnings,
        EventClass::PromoterAction,
        EventClass::CorporateDisclosure,
        EventClass::RegulatoryAction,
        EventClass::Governance,
        EventClass::Risk,
        EventClass::CorporateAction,
        EventClass::MergerAcquisition,
        EventClass::GlobalMovement,
        EventClass::PoliticalMeeting,
        EventClass::CompanyStructure,
        EventClass::Regulation,
        EventClass::PolicyChange,
        EventClass::Conflict,
        EventClass::MedicalClassification,
        EventClass::Filing,
        EventClass::MacroEvent,
        EventClass::MarketEvent,
    ]
}

pub fn mv_entity_types() -> &'static [EntityType] {
    &[
        EntityType::Company,
        EntityType::Instrument,
        EntityType::Country,
        EntityType::Regulator,
        EntityType::DiseaseClassification,
        EntityType::Commodity,
        EntityType::Sector,
        EntityType::Index,
        EntityType::Broker,
        EntityType::Source,
        EntityType::Exchange,
        EntityType::Region,
        EntityType::PolicyBody,
        EntityType::ConflictActor,
        EntityType::PoliticalOrganization,
        EntityType::Currency,
    ]
}

pub fn mv_relationship_types() -> &'static [RelationshipType] {
    &[
        RelationshipType::IssuesInstrument,
        RelationshipType::BelongsToSector,
        RelationshipType::ListedOnIndex,
        RelationshipType::GovernedByRegulator,
        RelationshipType::PolicyAffectsSector,
        RelationshipType::ConflictAffectsCommodity,
        RelationshipType::CommodityAffectsCompanyMargin,
        RelationshipType::MedicalClassificationAffectsSector,
        RelationshipType::SourceReportsEvent,
        RelationshipType::BrokerProvidesPaperExecution,
        RelationshipType::CountryHostsCompany,
        RelationshipType::EventCorroboratesEvent,
    ]
}

pub fn mv_fixture_graph() -> OntologyFixture {
    let observed_at = fixture_time();
    let provenance = |source: &str, source_id: &str, url: &str, confidence: f64| Provenance {
        source: source.to_string(),
        source_id: Some(source_id.to_string()),
        url: Some(url.to_string()),
        observed_at,
        confidence,
    };

    OntologyFixture {
        event_classes: mv_event_classes().to_vec(),
        entities: vec![
            entity(
                "company:reliance",
                EntityType::Company,
                "Reliance Industries",
                ["RELIANCE", "RIL"],
                Some("IN"),
            ),
            entity(
                "instrument:reliance-nse",
                EntityType::Instrument,
                "Reliance NSE equity",
                ["NSE:RELIANCE"],
                Some("IN"),
            ),
            entity(
                "country:in",
                EntityType::Country,
                "India",
                ["IN"],
                Some("IN"),
            ),
            entity(
                "regulator:sebi",
                EntityType::Regulator,
                "Securities and Exchange Board of India",
                ["SEBI"],
                Some("IN"),
            ),
            entity(
                "policy:rbi",
                EntityType::PolicyBody,
                "Reserve Bank of India",
                ["RBI"],
                Some("IN"),
            ),
            entity(
                "classification:icd11-respiratory",
                EntityType::DiseaseClassification,
                "ICD-11 respiratory classification",
                ["ICD-11", "respiratory disease"],
                None,
            ),
            entity(
                "commodity:brent",
                EntityType::Commodity,
                "Brent crude oil",
                ["Brent", "crude oil"],
                None,
            ),
            entity(
                "sector:energy",
                EntityType::Sector,
                "Energy",
                ["Oil & Gas"],
                Some("IN"),
            ),
            entity(
                "sector:healthcare",
                EntityType::Sector,
                "Healthcare",
                ["Pharma", "Hospitals"],
                Some("IN"),
            ),
            entity(
                "index:nifty50",
                EntityType::Index,
                "NIFTY 50",
                ["NIFTY50"],
                Some("IN"),
            ),
            entity(
                "broker:zerodha",
                EntityType::Broker,
                "Zerodha",
                ["Kite Connect"],
                Some("IN"),
            ),
            entity(
                "source:gdelt",
                EntityType::Source,
                "GDELT",
                ["Global Database of Events"],
                None,
            ),
            entity(
                "source:acled",
                EntityType::Source,
                "ACLED",
                ["Armed Conflict Location & Event Data"],
                None,
            ),
            entity(
                "actor:red-sea-shipping-risk",
                EntityType::ConflictActor,
                "Red Sea shipping risk",
                ["shipping lane risk"],
                None,
            ),
        ],
        relationships: vec![
            relationship(
                "rel:reliance-issues-nse",
                "company:reliance",
                RelationshipType::IssuesInstrument,
                "instrument:reliance-nse",
                &provenance("NSE", "NSE:RELIANCE", "https://www.nseindia.com/", 0.90),
            ),
            relationship(
                "rel:reliance-sector-energy",
                "company:reliance",
                RelationshipType::BelongsToSector,
                "sector:energy",
                &provenance(
                    "NSE",
                    "NSE_SECTOR_RELIANCE",
                    "https://www.nseindia.com/",
                    0.85,
                ),
            ),
            relationship(
                "rel:reliance-nifty50",
                "instrument:reliance-nse",
                RelationshipType::ListedOnIndex,
                "index:nifty50",
                &provenance(
                    "NSE",
                    "NIFTY50_CONSTITUENTS",
                    "https://www.nseindia.com/",
                    0.80,
                ),
            ),
            relationship(
                "rel:sebi-governs-nifty",
                "regulator:sebi",
                RelationshipType::GovernedByRegulator,
                "index:nifty50",
                &provenance("SEBI", "SEBI_MARKET_REG", "https://www.sebi.gov.in/", 0.85),
            ),
            relationship(
                "rel:rbi-policy-banking",
                "policy:rbi",
                RelationshipType::PolicyAffectsSector,
                "sector:energy",
                &provenance(
                    "RBI",
                    "RBI_POLICY_STATEMENT",
                    "https://www.rbi.org.in/",
                    0.70,
                ),
            ),
            relationship(
                "rel:shipping-risk-brent",
                "actor:red-sea-shipping-risk",
                RelationshipType::ConflictAffectsCommodity,
                "commodity:brent",
                &provenance(
                    "ACLED",
                    "CONFLICT_SHIPPING_FIXTURE",
                    "https://acleddata.com/",
                    0.70,
                ),
            ),
            relationship(
                "rel:brent-energy-margin",
                "commodity:brent",
                RelationshipType::CommodityAffectsCompanyMargin,
                "sector:energy",
                &provenance(
                    "World Bank",
                    "BRENT_PRICE_FIXTURE",
                    "https://www.worldbank.org/",
                    0.75,
                ),
            ),
            relationship(
                "rel:icd-healthcare-sector",
                "classification:icd11-respiratory",
                RelationshipType::MedicalClassificationAffectsSector,
                "sector:healthcare",
                &provenance("WHO", "ICD11_FIXTURE", "https://icd.who.int/", 0.80),
            ),
            relationship(
                "rel:gdelt-reports-global-event",
                "source:gdelt",
                RelationshipType::SourceReportsEvent,
                "event:global-supply-chain-disruption",
                &provenance(
                    "GDELT",
                    "GDELT_EVENT_FIXTURE",
                    "https://www.gdeltproject.org/",
                    0.65,
                ),
            ),
            relationship(
                "rel:zerodha-paper-execution",
                "broker:zerodha",
                RelationshipType::BrokerProvidesPaperExecution,
                "instrument:reliance-nse",
                &provenance(
                    "Zerodha",
                    "PAPER_EXECUTION_FIXTURE",
                    "https://kite.trade/",
                    0.60,
                ),
            ),
            relationship(
                "rel:india-hosts-reliance",
                "country:in",
                RelationshipType::CountryHostsCompany,
                "company:reliance",
                &provenance(
                    "Company filing",
                    "REGISTERED_OFFICE_FIXTURE",
                    "https://www.bseindia.com/",
                    0.80,
                ),
            ),
            relationship(
                "rel:acled-corroborates-gdelt",
                "source:acled",
                RelationshipType::EventCorroboratesEvent,
                "event:global-supply-chain-disruption",
                &provenance(
                    "ACLED",
                    "ACLED_EVENT_FIXTURE",
                    "https://acleddata.com/",
                    0.60,
                ),
            ),
        ],
    }
}

fn entity<const N: usize>(
    entity_id: &str,
    entity_type: EntityType,
    label: &str,
    aliases: [&str; N],
    region: Option<&str>,
) -> KnowledgeEntity {
    KnowledgeEntity {
        entity_id: entity_id.to_string(),
        entity_type,
        label: label.to_string(),
        aliases: aliases.into_iter().map(str::to_string).collect(),
        region: region.map(str::to_string),
    }
}

fn relationship(
    relationship_id: &str,
    subject_id: &str,
    relationship_type: RelationshipType,
    object_id: &str,
    provenance: &Provenance,
) -> KnowledgeRelationship {
    KnowledgeRelationship {
        relationship_id: relationship_id.to_string(),
        subject_id: subject_id.to_string(),
        relationship_type,
        object_id: object_id.to_string(),
        effective_from: Some(fixture_time()),
        effective_to: None,
        provenance: provenance.clone(),
    }
}

fn fixture_time() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-07-01T00:00:00Z")
        .expect("valid fixture timestamp")
        .with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ontology_includes_required_release_event_classes() {
        let classes = mv_event_classes();

        for class in [
            EventClass::GlobalMovement,
            EventClass::PoliticalMeeting,
            EventClass::CompanyStructure,
            EventClass::Regulation,
            EventClass::PolicyChange,
            EventClass::Conflict,
            EventClass::MedicalClassification,
            EventClass::Filing,
            EventClass::MacroEvent,
            EventClass::MarketEvent,
        ] {
            assert!(classes.contains(&class), "missing {class:?}");
        }
    }

    #[test]
    fn ontology_includes_required_entity_types() {
        let entity_types = mv_entity_types();

        for entity_type in [
            EntityType::Company,
            EntityType::Instrument,
            EntityType::Country,
            EntityType::Regulator,
            EntityType::DiseaseClassification,
            EntityType::Commodity,
            EntityType::Sector,
            EntityType::Index,
            EntityType::Broker,
            EntityType::Source,
        ] {
            assert!(
                entity_types.contains(&entity_type),
                "missing {entity_type:?}"
            );
        }
    }

    #[test]
    fn fixture_relationships_have_provenance() {
        let graph = mv_fixture_graph();

        assert!(!graph.relationships.is_empty());
        for relationship in graph.relationships {
            assert!(!relationship.relationship_id.is_empty());
            assert!(!relationship.subject_id.is_empty());
            assert!(!relationship.object_id.is_empty());
            assert!(relationship.provenance.confidence > 0.0);
            assert!(relationship.provenance.confidence <= 1.0);
            assert!(relationship.provenance.url.is_some());
            assert!(relationship.effective_from.is_some());
        }
    }

    #[test]
    fn fixture_connects_policy_conflict_medical_and_market_paths() {
        let graph = mv_fixture_graph();
        let relationship_types = graph
            .relationships
            .iter()
            .map(|relationship| relationship.relationship_type)
            .collect::<Vec<_>>();

        for relationship_type in [
            RelationshipType::PolicyAffectsSector,
            RelationshipType::ConflictAffectsCommodity,
            RelationshipType::CommodityAffectsCompanyMargin,
            RelationshipType::MedicalClassificationAffectsSector,
            RelationshipType::BrokerProvidesPaperExecution,
        ] {
            assert!(
                relationship_types.contains(&relationship_type),
                "missing {relationship_type:?}"
            );
        }
    }
}
