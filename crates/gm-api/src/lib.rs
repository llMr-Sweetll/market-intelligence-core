use std::{path::PathBuf, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use gm_domain::{
    AsOfFacts, DECISION_MODEL_VERSION, DecisionInput, DecisionThresholds, EntityType, EventClass,
    MacroContext, NormalizedEvent, PriceBar, RuleRegistry, build_macro_context, classify,
    compute_features, decide, gbm_flow_prediction, score_event,
};
use gm_persistence::PgStore;
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

const SERVICE_NAME: &str = "gm-api";

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub database_url: Option<String>,
    pub migrations: PathBuf,
    pub run_migrations: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            database_url: None,
            migrations: PathBuf::from("migrations"),
            run_migrations: true,
        }
    }
}

#[derive(Clone)]
struct AppState {
    registry: Arc<RuleRegistry>,
    thresholds: DecisionThresholds,
    store: Option<PgStore>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Debug, Serialize)]
struct VersionResponse {
    service: &'static str,
    version: &'static str,
    model_version: &'static str,
}

#[derive(Debug, Serialize)]
struct ReadyResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    persistence: PersistenceStatus,
}

#[derive(Debug, Serialize)]
struct PersistenceStatus {
    configured: bool,
    connected: bool,
    migrations: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct EventReviewSummary {
    event_id: String,
    version: i32,
    headline: String,
    occurred_at: chrono::DateTime<chrono::Utc>,
    source: Option<String>,
    region: Option<String>,
    sector: Option<String>,
    symbol: Option<String>,
    event_class: EventClass,
    confidence: f64,
    severity: String,
    entity_mapping_status: String,
    source_reliability: SourceReliability,
}

#[derive(Debug, Clone, Serialize)]
struct EventReviewDetail {
    summary: EventReviewSummary,
    event: NormalizedEvent,
    raw_source: RawSourceMetadata,
    normalized_facts: NormalizedFacts,
    entity_mappings: Vec<EntityMapping>,
    source_reliability: SourceReliability,
}

#[derive(Debug, Clone, Serialize)]
struct RawSourceMetadata {
    provider: String,
    source_id: String,
    url: Option<String>,
    received_at: chrono::DateTime<chrono::Utc>,
    language: String,
    raw_headline: String,
}

#[derive(Debug, Clone, Serialize)]
struct NormalizedFacts {
    event_type: Option<String>,
    symbol: Option<String>,
    sector: Option<String>,
    region: Option<String>,
    impact_level: Option<String>,
    impact_category: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct EntityMapping {
    entity_id: String,
    entity_type: EntityType,
    label: String,
    confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
struct SourceReliability {
    tier: String,
    score: f64,
    rationale: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DecideRequest {
    event: NormalizedEvent,
    facts: Option<AsOfFacts>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FeatureRequest {
    symbol: String,
    as_of: chrono::NaiveDate,
    bars: Vec<PriceBar>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PredictionRequest {
    symbol: String,
    as_of: chrono::NaiveDate,
    horizon: u32,
    fii_flow_norm: f64,
    bars: Vec<PriceBar>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MacroContextRequest {
    sector: String,
    inputs: gm_domain::MacroInputs,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn persistence(error: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("persistence failed: {error}"),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}

pub async fn build_app(config: ApiConfig) -> anyhow::Result<Router> {
    let store = match config.database_url.as_deref() {
        Some(database_url) if !database_url.trim().is_empty() => {
            let store = PgStore::connect(database_url).await?;
            if config.run_migrations {
                store.run_migrations(&config.migrations).await?;
            }
            Some(store)
        }
        _ => None,
    };

    let state = AppState {
        registry: Arc::new(RuleRegistry::builtin()),
        thresholds: DecisionThresholds::default(),
        store,
    };

    Ok(Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/version", get(version))
        .route("/openapi.json", get(openapi))
        .route("/events", get(events))
        .route("/events/{event_id}", get(event_detail))
        .route("/rules", get(rules))
        .route("/score", post(score))
        .route("/decide", post(decide_route))
        .route("/quant/features", post(features))
        .route("/predict/gbm", post(predict_gbm))
        .route("/macro/context", post(macro_context))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: SERVICE_NAME,
    })
}

async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        service: SERVICE_NAME,
        version: env!("CARGO_PKG_VERSION"),
        model_version: DECISION_MODEL_VERSION,
    })
}

async fn ready(State(state): State<AppState>) -> (StatusCode, Json<ReadyResponse>) {
    match state.store.as_ref() {
        Some(store) => match store.ping().await {
            Ok(()) => (
                StatusCode::OK,
                Json(ReadyResponse {
                    status: "ready",
                    service: SERVICE_NAME,
                    version: env!("CARGO_PKG_VERSION"),
                    persistence: PersistenceStatus {
                        configured: true,
                        connected: true,
                        migrations: "applied",
                    },
                }),
            ),
            Err(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ReadyResponse {
                    status: "not_ready",
                    service: SERVICE_NAME,
                    version: env!("CARGO_PKG_VERSION"),
                    persistence: PersistenceStatus {
                        configured: true,
                        connected: false,
                        migrations: "unknown",
                    },
                }),
            ),
        },
        None => (
            StatusCode::OK,
            Json(ReadyResponse {
                status: "ready",
                service: SERVICE_NAME,
                version: env!("CARGO_PKG_VERSION"),
                persistence: PersistenceStatus {
                    configured: false,
                    connected: false,
                    migrations: "not_configured",
                },
            }),
        ),
    }
}

async fn events() -> Json<Vec<EventReviewSummary>> {
    Json(
        event_review_fixtures()
            .into_iter()
            .map(|review| review.summary)
            .collect(),
    )
}

async fn event_detail(Path(event_id): Path<String>) -> Result<Json<EventReviewDetail>, ApiError> {
    event_review_fixtures()
        .into_iter()
        .find(|review| review.summary.event_id == event_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("event not found: {event_id}")))
}

async fn rules(State(state): State<AppState>) -> Json<Vec<gm_domain::RuleDefinition>> {
    Json(state.registry.rules().to_vec())
}

async fn score(
    State(state): State<AppState>,
    Json(event): Json<NormalizedEvent>,
) -> Json<gm_domain::scoring::ScoreOutput> {
    Json(score_event(&event, &state.registry))
}

async fn decide_route(
    State(state): State<AppState>,
    Json(request): Json<DecideRequest>,
) -> Result<Json<gm_domain::Decision>, ApiError> {
    let score = score_event(&request.event, &state.registry);
    let input = DecisionInput {
        event: request.event,
        score,
        facts: request.facts.unwrap_or_default(),
        thresholds: state.thresholds,
    };
    let decision = decide(input.clone());

    if let Some(store) = state.store.as_ref() {
        store
            .save_decision_audit(&input, &decision, DECISION_MODEL_VERSION)
            .await
            .map_err(ApiError::persistence)?;
    }

    Ok(Json(decision))
}

async fn features(Json(request): Json<FeatureRequest>) -> Json<gm_domain::FeatureVector> {
    Json(compute_features(
        &request.symbol,
        request.as_of,
        &request.bars,
    ))
}

async fn predict_gbm(Json(request): Json<PredictionRequest>) -> Json<gm_domain::PredictionRecord> {
    Json(gbm_flow_prediction(
        &request.symbol,
        request.as_of,
        &request.bars,
        request.horizon,
        request.fii_flow_norm,
    ))
}

async fn macro_context(Json(request): Json<MacroContextRequest>) -> Json<MacroContext> {
    Json(build_macro_context(&request.sector, request.inputs))
}

fn event_review_fixtures() -> Vec<EventReviewDetail> {
    vec![
        build_event_review(EventReviewSeed {
            event_id: "norm-smoke-earnings",
            version: 1,
            causal_parent_id: Some("raw-smoke-earnings"),
            event_type: Some("EARNINGS"),
            headline: "Quarterly earnings beat estimates",
            body: "Profit rose and revenue grew higher than expected.",
            occurred_at: "2026-07-06T09:15:00Z",
            symbol: Some("RELIANCE"),
            sector: Some("Oil & Gas"),
            source: Some("NSE"),
            region: Some("IN"),
            impact_level: Some("HIGH"),
            impact_category: Some("EARNINGS"),
            provider: "NSE",
            source_id: "raw-smoke-earnings",
            url: Some("https://www.nseindia.com/"),
            severity: "High",
            confidence: 0.91,
            reliability_tier: "primary",
            reliability_score: 0.90,
            reliability_rationale: "Exchange filing fixture with direct company symbol mapping.",
            mappings: vec![
                entity_mapping(
                    "company:reliance",
                    EntityType::Company,
                    "Reliance Industries",
                    0.95,
                ),
                entity_mapping(
                    "instrument:reliance-nse",
                    EntityType::Instrument,
                    "Reliance NSE equity",
                    0.93,
                ),
                entity_mapping("sector:energy", EntityType::Sector, "Energy", 0.86),
            ],
        }),
        build_event_review(EventReviewSeed {
            event_id: "norm-policy-liquidity",
            version: 1,
            causal_parent_id: Some("raw-rbi-liquidity"),
            event_type: Some("POLICY_CHANGE"),
            headline: "Central bank policy statement updates liquidity stance",
            body: "Rate decision commentary changes liquidity stance and bank funding expectations.",
            occurred_at: "2026-07-06T10:00:00Z",
            symbol: Some("BANKNIFTY"),
            sector: Some("Banking"),
            source: Some("RBI"),
            region: Some("IN"),
            impact_level: Some("REVIEW"),
            impact_category: Some("MACRO_POLICY"),
            provider: "RBI",
            source_id: "raw-rbi-liquidity",
            url: Some("https://www.rbi.org.in/"),
            severity: "Review",
            confidence: 0.82,
            reliability_tier: "primary",
            reliability_score: 0.88,
            reliability_rationale: "Policy-body fixture with direct macro-policy source.",
            mappings: vec![
                entity_mapping(
                    "policy:rbi",
                    EntityType::PolicyBody,
                    "Reserve Bank of India",
                    0.92,
                ),
                entity_mapping("country:in", EntityType::Country, "India", 0.88),
                entity_mapping("index:nifty50", EntityType::Index, "NIFTY 50", 0.65),
            ],
        }),
        build_event_review(EventReviewSeed {
            event_id: "norm-medical-classification",
            version: 1,
            causal_parent_id: Some("raw-who-icd11"),
            event_type: Some("MEDICAL_CLASSIFICATION"),
            headline: "Therapy classification update affects reimbursement basket",
            body: "ICD-11 medical classification and reimbursement code update affects healthcare exposure.",
            occurred_at: "2026-07-06T11:30:00Z",
            symbol: Some("PHARMA"),
            sector: Some("Healthcare"),
            source: Some("WHO"),
            region: Some("GLOBAL"),
            impact_level: Some("WATCH"),
            impact_category: Some("HEALTH_CLASSIFICATION"),
            provider: "WHO",
            source_id: "raw-who-icd11",
            url: Some("https://icd.who.int/"),
            severity: "Watch",
            confidence: 0.78,
            reliability_tier: "reference",
            reliability_score: 0.80,
            reliability_rationale: "Reference taxonomy fixture used for market categorization only.",
            mappings: vec![
                entity_mapping(
                    "classification:icd11-respiratory",
                    EntityType::DiseaseClassification,
                    "ICD-11 respiratory classification",
                    0.84,
                ),
                entity_mapping("sector:healthcare", EntityType::Sector, "Healthcare", 0.76),
            ],
        }),
        build_event_review(EventReviewSeed {
            event_id: "norm-company-structure",
            version: 1,
            causal_parent_id: Some("raw-company-restructure"),
            event_type: Some("COMPANY_STRUCTURE"),
            headline: "Company board approves structure and reporting change",
            body: "Subsidiary restructuring updates ownership structure and segment reporting.",
            occurred_at: "2026-07-06T12:45:00Z",
            symbol: Some("RELIANCE"),
            sector: Some("Oil & Gas"),
            source: Some("BSE"),
            region: Some("IN"),
            impact_level: Some("REVIEW"),
            impact_category: Some("COMPANY_STRUCTURE"),
            provider: "BSE",
            source_id: "raw-company-restructure",
            url: Some("https://www.bseindia.com/"),
            severity: "Review",
            confidence: 0.74,
            reliability_tier: "primary",
            reliability_score: 0.84,
            reliability_rationale: "Company disclosure fixture with known issuer and sector path.",
            mappings: vec![
                entity_mapping(
                    "company:reliance",
                    EntityType::Company,
                    "Reliance Industries",
                    0.91,
                ),
                entity_mapping("sector:energy", EntityType::Sector, "Energy", 0.80),
            ],
        }),
        build_event_review(EventReviewSeed {
            event_id: "norm-conflict-shipping",
            version: 1,
            causal_parent_id: Some("raw-acled-shipping"),
            event_type: Some("CONFLICT"),
            headline: "Shipping lane conflict raises crude supply risk",
            body: "Geopolitical tension and shipping lane risk affect Brent crude movement.",
            occurred_at: "2026-07-06T13:20:00Z",
            symbol: Some("BRENT"),
            sector: Some("Energy"),
            source: Some("ACLED"),
            region: Some("GLOBAL"),
            impact_level: Some("HIGH"),
            impact_category: Some("CONFLICT_MARKET"),
            provider: "ACLED",
            source_id: "raw-acled-shipping",
            url: Some("https://acleddata.com/"),
            severity: "High",
            confidence: 0.72,
            reliability_tier: "corroborated",
            reliability_score: 0.72,
            reliability_rationale: "Conflict fixture linked to commodity and sector exposure.",
            mappings: vec![
                entity_mapping(
                    "actor:red-sea-shipping-risk",
                    EntityType::ConflictActor,
                    "Red Sea shipping risk",
                    0.78,
                ),
                entity_mapping(
                    "commodity:brent",
                    EntityType::Commodity,
                    "Brent crude oil",
                    0.86,
                ),
                entity_mapping("sector:energy", EntityType::Sector, "Energy", 0.74),
            ],
        }),
    ]
}

struct EventReviewSeed {
    event_id: &'static str,
    version: i32,
    causal_parent_id: Option<&'static str>,
    event_type: Option<&'static str>,
    headline: &'static str,
    body: &'static str,
    occurred_at: &'static str,
    symbol: Option<&'static str>,
    sector: Option<&'static str>,
    source: Option<&'static str>,
    region: Option<&'static str>,
    impact_level: Option<&'static str>,
    impact_category: Option<&'static str>,
    provider: &'static str,
    source_id: &'static str,
    url: Option<&'static str>,
    severity: &'static str,
    confidence: f64,
    reliability_tier: &'static str,
    reliability_score: f64,
    reliability_rationale: &'static str,
    mappings: Vec<EntityMapping>,
}

fn build_event_review(seed: EventReviewSeed) -> EventReviewDetail {
    let event = NormalizedEvent {
        event_id: seed.event_id.to_string(),
        version: seed.version,
        causal_parent_id: seed.causal_parent_id.map(str::to_string),
        event_type: seed.event_type.map(str::to_string),
        headline: seed.headline.to_string(),
        body: seed.body.to_string(),
        occurred_at: parse_timestamp(seed.occurred_at),
        symbol: seed.symbol.map(str::to_string),
        sector: seed.sector.map(str::to_string),
        source: seed.source.map(str::to_string),
        region: seed.region.map(str::to_string),
        impact_level: seed.impact_level.map(str::to_string),
        impact_category: seed.impact_category.map(str::to_string),
    };
    let source_reliability = SourceReliability {
        tier: seed.reliability_tier.to_string(),
        score: seed.reliability_score,
        rationale: seed.reliability_rationale.to_string(),
    };
    let entity_mapping_status = if seed
        .mappings
        .iter()
        .all(|mapping| mapping.confidence >= 0.75)
    {
        "resolved"
    } else {
        "review"
    };
    let summary = EventReviewSummary {
        event_id: event.event_id.clone(),
        version: event.version,
        headline: event.headline.clone(),
        occurred_at: event.occurred_at,
        source: event.source.clone(),
        region: event.region.clone(),
        sector: event.sector.clone(),
        symbol: event.symbol.clone(),
        event_class: classify(&event),
        confidence: seed.confidence,
        severity: seed.severity.to_string(),
        entity_mapping_status: entity_mapping_status.to_string(),
        source_reliability: source_reliability.clone(),
    };
    let normalized_facts = NormalizedFacts {
        event_type: event.event_type.clone(),
        symbol: event.symbol.clone(),
        sector: event.sector.clone(),
        region: event.region.clone(),
        impact_level: event.impact_level.clone(),
        impact_category: event.impact_category.clone(),
    };
    let raw_source = RawSourceMetadata {
        provider: seed.provider.to_string(),
        source_id: seed.source_id.to_string(),
        url: seed.url.map(str::to_string),
        received_at: event.occurred_at,
        language: "en".to_string(),
        raw_headline: seed.headline.to_string(),
    };

    EventReviewDetail {
        summary,
        event,
        raw_source,
        normalized_facts,
        entity_mappings: seed.mappings,
        source_reliability,
    }
}

fn entity_mapping(
    entity_id: &str,
    entity_type: EntityType,
    label: &str,
    confidence: f64,
) -> EntityMapping {
    EntityMapping {
        entity_id: entity_id.to_string(),
        entity_type,
        label: label.to_string(),
        confidence,
    }
}

fn parse_timestamp(value: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(value)
        .expect("valid fixture timestamp")
        .with_timezone(&chrono::Utc)
}

async fn openapi() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Market Intelligence Core API",
            "version": env!("CARGO_PKG_VERSION"),
            "summary": "Deterministic market event scoring and decision service"
        },
        "paths": {
            "/health": {
                "get": {
                    "summary": "Liveness probe",
                    "responses": {
                        "200": { "description": "API process is alive" }
                    }
                }
            },
            "/ready": {
                "get": {
                    "summary": "Readiness probe including optional persistence",
                    "responses": {
                        "200": { "description": "API is ready" },
                        "503": { "description": "Configured persistence is unavailable" }
                    }
                }
            },
            "/version": {
                "get": {
                    "summary": "Service and model version",
                    "responses": {
                        "200": { "description": "Version metadata" }
                    }
                }
            },
            "/rules": {
                "get": {
                    "summary": "Built-in scoring rules",
                    "responses": {
                        "200": { "description": "Rule definitions" }
                    }
                }
            },
            "/events": {
                "get": {
                    "summary": "Fixture-backed normalized event review list",
                    "responses": {
                        "200": {
                            "description": "Event review summaries",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": { "$ref": "#/components/schemas/EventReviewSummary" }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/events/{event_id}": {
                "get": {
                    "summary": "Fixture-backed normalized event review detail",
                    "parameters": [
                        {
                            "name": "event_id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": { "description": "Event review detail" },
                        "404": { "description": "Event not found" }
                    }
                }
            },
            "/score": {
                "post": {
                    "summary": "Score a normalized event",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/NormalizedEvent" }
                            }
                        }
                    },
                    "responses": {
                        "200": { "description": "Score output" }
                    }
                }
            },
            "/decide": {
                "post": {
                    "summary": "Create a deterministic decision from an event and as-of facts",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/DecideRequest" }
                            }
                        }
                    },
                    "responses": {
                        "200": { "description": "Decision" },
                        "500": { "description": "Persistence failed when configured" }
                    }
                }
            },
            "/quant/features": {
                "post": {
                    "summary": "Compute technical features from supplied price bars",
                    "responses": {
                        "200": { "description": "Feature vector" }
                    }
                }
            },
            "/predict/gbm": {
                "post": {
                    "summary": "Compute deterministic GBM flow-adjusted prediction",
                    "responses": {
                        "200": { "description": "Prediction record" }
                    }
                }
            },
            "/macro/context": {
                "post": {
                    "summary": "Build sector-weighted macro context",
                    "responses": {
                        "200": { "description": "Macro context" }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "NormalizedEvent": {
                    "type": "object",
                    "required": ["event_id", "version", "headline", "body", "occurred_at"],
                    "properties": {
                        "event_id": { "type": "string" },
                        "version": { "type": "integer" },
                        "causal_parent_id": { "type": ["string", "null"] },
                        "event_type": { "type": ["string", "null"] },
                        "headline": { "type": "string" },
                        "body": { "type": "string" },
                        "occurred_at": { "type": "string", "format": "date-time" },
                        "symbol": { "type": ["string", "null"] },
                        "sector": { "type": ["string", "null"] },
                        "source": { "type": ["string", "null"] },
                        "region": { "type": ["string", "null"] },
                        "impact_level": { "type": ["string", "null"] },
                        "impact_category": { "type": ["string", "null"] }
                    }
                },
                "DecideRequest": {
                    "type": "object",
                    "required": ["event"],
                    "properties": {
                        "event": { "$ref": "#/components/schemas/NormalizedEvent" },
                        "facts": { "type": ["object", "null"] }
                    }
                },
                "Decision": {
                    "type": "object",
                    "required": [
                        "decision_id",
                        "model_version",
                        "input_hash",
                        "action",
                        "total_score",
                        "confidence",
                        "expected_return",
                        "downside",
                        "explanation",
                        "execution_ready"
                    ],
                    "properties": {
                        "decision_id": { "type": "string" },
                        "model_version": { "type": "string" },
                        "input_hash": { "type": "string" },
                        "action": { "type": "string", "enum": ["BUY", "SELL", "HOLD"] },
                        "total_score": { "type": "number" },
                        "confidence": { "type": "number" },
                        "expected_return": { "type": ["number", "null"] },
                        "downside": { "type": ["number", "null"] },
                        "explanation": { "type": "object" },
                        "execution_ready": { "type": "boolean" }
                    }
                },
                "EventReviewSummary": {
                    "type": "object",
                    "required": [
                        "event_id",
                        "version",
                        "headline",
                        "occurred_at",
                        "event_class",
                        "confidence",
                        "severity",
                        "entity_mapping_status",
                        "source_reliability"
                    ],
                    "properties": {
                        "event_id": { "type": "string" },
                        "version": { "type": "integer" },
                        "headline": { "type": "string" },
                        "occurred_at": { "type": "string", "format": "date-time" },
                        "source": { "type": ["string", "null"] },
                        "region": { "type": ["string", "null"] },
                        "sector": { "type": ["string", "null"] },
                        "symbol": { "type": ["string", "null"] },
                        "event_class": { "type": "string" },
                        "confidence": { "type": "number" },
                        "severity": { "type": "string" },
                        "entity_mapping_status": { "type": "string" },
                        "source_reliability": { "type": "object" }
                    }
                },
                "EventReviewDetail": {
                    "type": "object",
                    "required": [
                        "summary",
                        "event",
                        "raw_source",
                        "normalized_facts",
                        "entity_mappings",
                        "source_reliability"
                    ],
                    "properties": {
                        "summary": { "$ref": "#/components/schemas/EventReviewSummary" },
                        "event": { "$ref": "#/components/schemas/NormalizedEvent" },
                        "raw_source": { "type": "object" },
                        "normalized_facts": { "type": "object" },
                        "entity_mappings": {
                            "type": "array",
                            "items": { "type": "object" }
                        },
                        "source_reliability": { "type": "object" }
                    }
                }
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;

    async fn app() -> Router {
        build_app(ApiConfig::default()).await.unwrap()
    }

    async fn json_request(
        app: Router,
        method: &str,
        uri: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload = serde_json::from_slice(&bytes).unwrap();
        (status, payload)
    }

    async fn get_json(app: Router, uri: &str) -> (StatusCode, Value) {
        let request = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload = serde_json::from_slice(&bytes).unwrap();
        (status, payload)
    }

    fn decision_request() -> Value {
        json!({
          "event": {
            "event_id": "norm-smoke-earnings",
            "version": 1,
            "causal_parent_id": "raw-smoke-earnings",
            "event_type": "EARNINGS",
            "headline": "Quarterly earnings beat estimates",
            "body": "Profit rose and revenue grew higher than expected.",
            "occurred_at": "2026-07-06T09:15:00Z",
            "symbol": "RELIANCE",
            "sector": "Oil & Gas",
            "source": "NSE",
            "region": "IN",
            "impact_level": null,
            "impact_category": null
          },
          "facts": {
            "macro_context": {
              "sp500_futures_change": 0,
              "nasdaq_futures_change": 0,
              "brent_crude_change": 0,
              "usd_inr_change": 0,
              "fii_net_flow": 0,
              "gold_change": 0,
              "total_macro_score": 0
            },
            "entry_price": 1000,
            "exchange": "NSE",
            "features": null,
            "prediction": null,
            "kg_modifier": 0
          }
        })
    }

    #[tokio::test]
    async fn readiness_reports_optional_persistence() {
        let (status, payload) = get_json(app().await, "/ready").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["status"], "ready");
        assert_eq!(payload["persistence"]["configured"], false);
        assert_eq!(payload["persistence"]["migrations"], "not_configured");
    }

    #[tokio::test]
    async fn version_reports_service_and_model() {
        let (status, payload) = get_json(app().await, "/version").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["service"], SERVICE_NAME);
        assert_eq!(payload["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(payload["model_version"], DECISION_MODEL_VERSION);
    }

    #[tokio::test]
    async fn openapi_lists_release_paths() {
        let (status, payload) = get_json(app().await, "/openapi.json").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["openapi"], "3.1.0");
        assert!(payload["paths"].get("/ready").is_some());
        assert!(payload["paths"].get("/version").is_some());
        assert!(payload["paths"].get("/decide").is_some());
        assert!(payload["paths"].get("/events").is_some());
        assert!(payload["paths"].get("/events/{event_id}").is_some());
    }

    #[tokio::test]
    async fn events_contract_returns_fixture_summaries() {
        let (status, payload) = get_json(app().await, "/events").await;

        assert_eq!(status, StatusCode::OK);
        assert!(payload.as_array().unwrap().len() >= 5);
        assert_eq!(payload[0]["event_id"], "norm-smoke-earnings");
        assert_eq!(payload[0]["event_class"], "EARNINGS");
        assert_eq!(payload[0]["entity_mapping_status"], "resolved");
        assert!(payload[0]["source_reliability"]["score"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn event_detail_contract_returns_review_context() {
        let (status, payload) = get_json(app().await, "/events/norm-medical-classification").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["summary"]["event_class"], "MEDICAL_CLASSIFICATION");
        assert_eq!(payload["raw_source"]["provider"], "WHO");
        assert_eq!(
            payload["normalized_facts"]["impact_category"],
            "HEALTH_CLASSIFICATION"
        );
        assert!(payload["entity_mappings"].as_array().unwrap().len() >= 2);
        assert_eq!(
            payload["entity_mappings"][0]["entity_type"],
            "DISEASE_CLASSIFICATION"
        );
    }

    #[tokio::test]
    async fn missing_event_returns_not_found() {
        let (status, payload) = get_json(app().await, "/events/does-not-exist").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(payload["error"], "event not found: does-not-exist");
    }

    #[tokio::test]
    async fn decide_contract_returns_deterministic_buy() {
        let (status, payload) =
            json_request(app().await, "POST", "/decide", decision_request()).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["action"], "BUY");
        assert_eq!(payload["execution_ready"], true);
        assert_eq!(payload["quantity"], 20);
        assert_eq!(payload["target_price"], 1030.0);
        assert_eq!(payload["stop_loss"], 980.0);
        assert_eq!(payload["total_score"], 0.72);
        assert_eq!(payload["model_version"], DECISION_MODEL_VERSION);
        assert!(payload["input_hash"].as_str().unwrap().len() > 20);
        assert!(payload["explanation"]["utilities"].is_array());
    }
}
