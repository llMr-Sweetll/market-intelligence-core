use std::{path::PathBuf, sync::Arc};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use gm_domain::{
    AsOfFacts, DECISION_MODEL_VERSION, DecisionInput, DecisionThresholds, MacroContext,
    NormalizedEvent, PriceBar, RuleRegistry, build_macro_context, compute_features, decide,
    gbm_flow_prediction, score_event,
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
