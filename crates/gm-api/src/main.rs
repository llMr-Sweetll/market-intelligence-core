use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use clap::Parser;
use gm_domain::{
    AsOfFacts, DecisionInput, DecisionThresholds, MacroContext, NormalizedEvent, PriceBar,
    RuleRegistry, build_macro_context, compute_features, decide, gbm_flow_prediction, score_event,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "GM_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "GM_PORT", default_value_t = 8000)]
    port: u16,
}

#[derive(Clone)]
struct AppState {
    registry: Arc<RuleRegistry>,
    thresholds: DecisionThresholds,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Debug, Deserialize)]
struct DecideRequest {
    event: NormalizedEvent,
    facts: Option<AsOfFacts>,
}

#[derive(Debug, Deserialize)]
struct FeatureRequest {
    symbol: String,
    as_of: chrono::NaiveDate,
    bars: Vec<PriceBar>,
}

#[derive(Debug, Deserialize)]
struct PredictionRequest {
    symbol: String,
    as_of: chrono::NaiveDate,
    horizon: u32,
    fii_flow_norm: f64,
    bars: Vec<PriceBar>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let args = Args::parse();
    let state = AppState {
        registry: Arc::new(RuleRegistry::builtin()),
        thresholds: DecisionThresholds::default(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/rules", get(rules))
        .route("/score", post(score))
        .route("/decide", post(decide_route))
        .route("/quant/features", post(features))
        .route("/predict/gbm", post(predict_gbm))
        .route("/macro/context", post(macro_context))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    tracing::info!(%addr, "starting gm-api");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("gm_api=info,tower_http=info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "gm-api",
    })
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
) -> Json<gm_domain::Decision> {
    let score = score_event(&request.event, &state.registry);
    let decision = decide(DecisionInput {
        event: request.event,
        score,
        facts: request.facts.unwrap_or_default(),
        thresholds: state.thresholds,
    });
    Json(decision)
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

#[derive(Debug, Deserialize)]
struct MacroContextRequest {
    sector: String,
    inputs: gm_domain::MacroInputs,
}
