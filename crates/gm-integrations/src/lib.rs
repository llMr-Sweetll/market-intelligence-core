use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use gm_domain::{Action, NormalizedEvent, PriceBar};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderKind {
    MarketData,
    EventFeed,
    Filing,
    EntityMapping,
    Payment,
    BrokerExecution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderMode {
    Mock,
    ReadOnly,
    Paper,
    TestMode,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderHealth {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CircuitBreaker {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitState {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryState {
    pub max_attempts: u8,
    pub attempts_used: u8,
    pub backoff_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub state: CircuitBreaker,
    pub failure_count: u32,
    pub opened_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub name: String,
    pub kind: ProviderKind,
    pub mode: ProviderMode,
    pub health: ProviderHealth,
    pub rate_limit: RateLimitState,
    pub retry: RetryState,
    pub circuit_breaker: CircuitBreakerState,
    pub last_error: Option<String>,
}

impl ProviderStatus {
    pub fn healthy(name: impl Into<String>, kind: ProviderKind, mode: ProviderMode) -> Self {
        Self {
            name: name.into(),
            kind,
            mode,
            health: ProviderHealth::Healthy,
            rate_limit: RateLimitState {
                limit: 1_000,
                remaining: 1_000,
                reset_at: None,
            },
            retry: RetryState {
                max_attempts: 3,
                attempts_used: 0,
                backoff_ms: 250,
            },
            circuit_breaker: CircuitBreakerState {
                state: CircuitBreaker::Closed,
                failure_count: 0,
                opened_at: None,
            },
            last_error: None,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProviderError {
    #[error("provider unavailable: {0}")]
    Unavailable(String),
    #[error("symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("unsupported mode: {0}")]
    UnsupportedMode(String),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketQuote {
    pub symbol: String,
    pub exchange: String,
    pub as_of: DateTime<Utc>,
    pub price: f64,
    pub currency: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilingRecord {
    pub filing_id: String,
    pub symbol: String,
    pub title: String,
    pub filing_type: String,
    pub filed_at: DateTime<Utc>,
    pub source: String,
    pub url: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityQuery {
    pub query: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityMatch {
    pub query: String,
    pub symbol: String,
    pub company_name: String,
    pub sector: String,
    pub exchange: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub account_id: String,
    pub amount_paise: u64,
    pub currency: String,
    pub description: String,
    pub success_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckoutSession {
    pub provider: String,
    pub checkout_id: String,
    pub amount_paise: u64,
    pub currency: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub payment_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookVerification {
    pub provider: String,
    pub payment_id: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl TryFrom<Action> for OrderSide {
    type Error = ProviderError;

    fn try_from(action: Action) -> Result<Self, Self::Error> {
        match action {
            Action::Buy => Ok(Self::Buy),
            Action::Sell => Ok(Self::Sell),
            Action::Hold => Err(ProviderError::InvalidRequest(
                "HOLD does not map to an executable order side".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionMode {
    ReadOnly,
    Paper,
    Live,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderIntent {
    pub client_order_id: String,
    pub symbol: String,
    pub exchange: String,
    pub side: OrderSide,
    pub quantity: u64,
    pub order_type: OrderType,
    pub limit_price: Option<f64>,
    pub mode: ExecutionMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderReceipt {
    pub provider: String,
    pub provider_order_id: String,
    pub client_order_id: String,
    pub accepted: bool,
    pub mode: ExecutionMode,
    pub message: String,
}

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn status(&self) -> ProviderStatus;
    async fn latest_quote(&self, symbol: &str) -> Result<MarketQuote, ProviderError>;
    async fn price_bars(
        &self,
        symbol: &str,
        as_of: NaiveDate,
        lookback: usize,
    ) -> Result<Vec<PriceBar>, ProviderError>;
}

#[async_trait]
pub trait EventProvider: Send + Sync {
    async fn status(&self) -> ProviderStatus;
    async fn normalized_events(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<NormalizedEvent>, ProviderError>;
}

#[async_trait]
pub trait FilingProvider: Send + Sync {
    async fn status(&self) -> ProviderStatus;
    async fn filings(&self, symbol: &str) -> Result<Vec<FilingRecord>, ProviderError>;
}

#[async_trait]
pub trait EntityMappingProvider: Send + Sync {
    async fn status(&self) -> ProviderStatus;
    async fn resolve(&self, query: EntityQuery) -> Result<EntityMatch, ProviderError>;
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn status(&self) -> ProviderStatus;
    async fn create_checkout(
        &self,
        request: CheckoutRequest,
    ) -> Result<CheckoutSession, ProviderError>;
    async fn verify_webhook(
        &self,
        payload: WebhookPayload,
    ) -> Result<WebhookVerification, ProviderError>;
}

#[async_trait]
pub trait ExecutionProvider: Send + Sync {
    async fn status(&self) -> ProviderStatus;
    async fn submit_order(&self, intent: OrderIntent) -> Result<OrderReceipt, ProviderError>;
}

#[derive(Debug, Clone)]
pub struct MockMarketDataProvider {
    status: ProviderStatus,
    quotes: BTreeMap<String, MarketQuote>,
    bars: BTreeMap<String, Vec<PriceBar>>,
}

impl MockMarketDataProvider {
    pub fn fixture() -> Self {
        let symbol = "RELIANCE".to_string();
        let as_of = fixed_time();
        let quote = MarketQuote {
            symbol: symbol.clone(),
            exchange: "NSE".to_string(),
            as_of,
            price: 1000.0,
            currency: "INR".to_string(),
            source: "mock-market".to_string(),
        };

        let bars = vec![
            bar("RELIANCE", 2026, 7, 1, 980.0),
            bar("RELIANCE", 2026, 7, 2, 990.0),
            bar("RELIANCE", 2026, 7, 3, 995.0),
            bar("RELIANCE", 2026, 7, 6, 1000.0),
        ];

        Self {
            status: ProviderStatus::healthy(
                "mock-market",
                ProviderKind::MarketData,
                ProviderMode::Mock,
            ),
            quotes: BTreeMap::from([(symbol.clone(), quote)]),
            bars: BTreeMap::from([(symbol, bars)]),
        }
    }
}

#[async_trait]
impl MarketDataProvider for MockMarketDataProvider {
    async fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    async fn latest_quote(&self, symbol: &str) -> Result<MarketQuote, ProviderError> {
        self.quotes
            .get(symbol)
            .cloned()
            .ok_or_else(|| ProviderError::SymbolNotFound(symbol.to_string()))
    }

    async fn price_bars(
        &self,
        symbol: &str,
        as_of: NaiveDate,
        lookback: usize,
    ) -> Result<Vec<PriceBar>, ProviderError> {
        let bars = self
            .bars
            .get(symbol)
            .ok_or_else(|| ProviderError::SymbolNotFound(symbol.to_string()))?;
        let mut filtered = bars
            .iter()
            .filter(|bar| bar.date <= as_of)
            .cloned()
            .collect::<Vec<_>>();
        let start = filtered.len().saturating_sub(lookback);
        Ok(filtered.split_off(start))
    }
}

#[derive(Debug, Clone)]
pub struct MockEventProvider {
    status: ProviderStatus,
    events: Vec<NormalizedEvent>,
}

impl MockEventProvider {
    pub fn fixture() -> Self {
        Self {
            status: ProviderStatus::healthy(
                "mock-events",
                ProviderKind::EventFeed,
                ProviderMode::Mock,
            ),
            events: vec![NormalizedEvent {
                event_id: "norm-mock-earnings".to_string(),
                version: 1,
                causal_parent_id: Some("raw-mock-earnings".to_string()),
                event_type: Some("EARNINGS".to_string()),
                headline: "Quarterly earnings beat estimates".to_string(),
                body: "Profit rose and revenue grew higher than expected.".to_string(),
                occurred_at: fixed_time(),
                symbol: Some("RELIANCE".to_string()),
                sector: Some("Oil & Gas".to_string()),
                source: Some("mock-events".to_string()),
                region: Some("IN".to_string()),
                impact_level: None,
                impact_category: None,
            }],
        }
    }
}

#[async_trait]
impl EventProvider for MockEventProvider {
    async fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    async fn normalized_events(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<NormalizedEvent>, ProviderError> {
        Ok(self
            .events
            .iter()
            .filter(|event| since.is_none_or(|cutoff| event.occurred_at >= cutoff))
            .cloned()
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct MockFilingProvider {
    status: ProviderStatus,
    filings: BTreeMap<String, Vec<FilingRecord>>,
}

impl MockFilingProvider {
    pub fn fixture() -> Self {
        let filing = FilingRecord {
            filing_id: "filing-reliance-board-1".to_string(),
            symbol: "RELIANCE".to_string(),
            title: "Board approves reporting structure update".to_string(),
            filing_type: "BOARD_OUTCOME".to_string(),
            filed_at: fixed_time(),
            source: "mock-filings".to_string(),
            url: None,
            payload: serde_json::json!({
                "category": "company_structure",
                "requires_review": true
            }),
        };
        Self {
            status: ProviderStatus::healthy(
                "mock-filings",
                ProviderKind::Filing,
                ProviderMode::Mock,
            ),
            filings: BTreeMap::from([("RELIANCE".to_string(), vec![filing])]),
        }
    }
}

#[async_trait]
impl FilingProvider for MockFilingProvider {
    async fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    async fn filings(&self, symbol: &str) -> Result<Vec<FilingRecord>, ProviderError> {
        self.filings
            .get(symbol)
            .cloned()
            .ok_or_else(|| ProviderError::SymbolNotFound(symbol.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct MockEntityMappingProvider {
    status: ProviderStatus,
    matches: BTreeMap<String, EntityMatch>,
}

impl MockEntityMappingProvider {
    pub fn fixture() -> Self {
        let entity = EntityMatch {
            query: "Reliance Industries".to_string(),
            symbol: "RELIANCE".to_string(),
            company_name: "Reliance Industries Limited".to_string(),
            sector: "Oil & Gas".to_string(),
            exchange: "NSE".to_string(),
            confidence: 0.98,
        };
        Self {
            status: ProviderStatus::healthy(
                "mock-entities",
                ProviderKind::EntityMapping,
                ProviderMode::Mock,
            ),
            matches: BTreeMap::from([
                ("reliance industries".to_string(), entity.clone()),
                ("reliance".to_string(), entity),
            ]),
        }
    }
}

#[async_trait]
impl EntityMappingProvider for MockEntityMappingProvider {
    async fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    async fn resolve(&self, query: EntityQuery) -> Result<EntityMatch, ProviderError> {
        self.matches
            .get(&query.query.to_lowercase())
            .cloned()
            .ok_or_else(|| ProviderError::SymbolNotFound(query.query))
    }
}

#[derive(Debug, Clone)]
pub struct MockPaymentProvider {
    status: ProviderStatus,
}

impl MockPaymentProvider {
    pub fn fixture() -> Self {
        Self {
            status: ProviderStatus::healthy(
                "mock-razorpay",
                ProviderKind::Payment,
                ProviderMode::TestMode,
            ),
        }
    }
}

#[async_trait]
impl PaymentProvider for MockPaymentProvider {
    async fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    async fn create_checkout(
        &self,
        request: CheckoutRequest,
    ) -> Result<CheckoutSession, ProviderError> {
        if request.amount_paise == 0 {
            return Err(ProviderError::InvalidRequest(
                "amount must be greater than zero".to_string(),
            ));
        }

        Ok(CheckoutSession {
            provider: self.status.name.clone(),
            checkout_id: format!("checkout_test_{}", request.account_id),
            amount_paise: request.amount_paise,
            currency: request.currency,
            status: "created".to_string(),
        })
    }

    async fn verify_webhook(
        &self,
        payload: WebhookPayload,
    ) -> Result<WebhookVerification, ProviderError> {
        let verified =
            payload.signature == "test_signature" && payload.payment_id.starts_with("pay_test_");
        if !verified {
            return Err(ProviderError::VerificationFailed(
                "test signature mismatch".to_string(),
            ));
        }

        Ok(WebhookVerification {
            provider: self.status.name.clone(),
            payment_id: payload.payment_id,
            verified,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PaperExecutionProvider {
    status: ProviderStatus,
}

impl PaperExecutionProvider {
    pub fn fixture() -> Self {
        Self {
            status: ProviderStatus::healthy(
                "paper-broker",
                ProviderKind::BrokerExecution,
                ProviderMode::Paper,
            ),
        }
    }
}

#[async_trait]
impl ExecutionProvider for PaperExecutionProvider {
    async fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    async fn submit_order(&self, intent: OrderIntent) -> Result<OrderReceipt, ProviderError> {
        if intent.mode != ExecutionMode::Paper {
            return Err(ProviderError::UnsupportedMode(
                "paper provider accepts only PAPER mode".to_string(),
            ));
        }
        if intent.quantity == 0 {
            return Err(ProviderError::InvalidRequest(
                "quantity must be greater than zero".to_string(),
            ));
        }

        Ok(OrderReceipt {
            provider: self.status.name.clone(),
            provider_order_id: format!("paper_{}", intent.client_order_id),
            client_order_id: intent.client_order_id,
            accepted: true,
            mode: intent.mode,
            message: "paper order accepted".to_string(),
        })
    }
}

fn fixed_time() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-07-06T09:15:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

fn bar(symbol: &str, year: i32, month: u32, day: u32, close: f64) -> PriceBar {
    PriceBar {
        symbol: symbol.to_string(),
        date: NaiveDate::from_ymd_opt(year, month, day).unwrap(),
        open: close - 5.0,
        high: close + 10.0,
        low: close - 10.0,
        close,
        adj_close: Some(close),
        volume: 1_000_000,
        source: "mock-market".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn market_provider_returns_deterministic_quote_and_bars() {
        let provider = MockMarketDataProvider::fixture();
        let status = provider.status().await;
        let quote = provider.latest_quote("RELIANCE").await.unwrap();
        let bars = provider
            .price_bars("RELIANCE", NaiveDate::from_ymd_opt(2026, 7, 6).unwrap(), 2)
            .await
            .unwrap();

        assert_eq!(status.health, ProviderHealth::Healthy);
        assert_eq!(status.circuit_breaker.state, CircuitBreaker::Closed);
        assert_eq!(quote.price, 1000.0);
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[1].close, 1000.0);
    }

    #[tokio::test]
    async fn event_provider_returns_normalized_events() {
        let provider = MockEventProvider::fixture();
        let events = provider.normalized_events(None).await.unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_id, "norm-mock-earnings");
        assert_eq!(events[0].symbol.as_deref(), Some("RELIANCE"));
        assert_eq!(events[0].event_type.as_deref(), Some("EARNINGS"));
    }

    #[tokio::test]
    async fn entity_provider_resolves_known_company() {
        let provider = MockEntityMappingProvider::fixture();
        let entity = provider
            .resolve(EntityQuery {
                query: "Reliance".to_string(),
                region: Some("IN".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(entity.symbol, "RELIANCE");
        assert!(entity.confidence > 0.95);
    }

    #[tokio::test]
    async fn filing_provider_normalizes_company_structure_record() {
        let provider = MockFilingProvider::fixture();
        let filings = provider.filings("RELIANCE").await.unwrap();

        assert_eq!(filings.len(), 1);
        assert_eq!(filings[0].filing_type, "BOARD_OUTCOME");
        assert_eq!(filings[0].payload["category"], "company_structure");
    }

    #[tokio::test]
    async fn payment_provider_uses_test_mode_verification() {
        let provider = MockPaymentProvider::fixture();
        let checkout = provider
            .create_checkout(CheckoutRequest {
                account_id: "acct_1".to_string(),
                amount_paise: 49900,
                currency: "INR".to_string(),
                description: "MV access".to_string(),
                success_url: "https://example.test/success".to_string(),
            })
            .await
            .unwrap();
        let verification = provider
            .verify_webhook(WebhookPayload {
                event: "payment.captured".to_string(),
                payment_id: "pay_test_1".to_string(),
                signature: "test_signature".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(checkout.status, "created");
        assert!(verification.verified);
    }

    #[tokio::test]
    async fn paper_execution_rejects_live_orders() {
        let provider = PaperExecutionProvider::fixture();
        let intent = OrderIntent {
            client_order_id: "decision-1".to_string(),
            symbol: "RELIANCE".to_string(),
            exchange: "NSE".to_string(),
            side: OrderSide::Buy,
            quantity: 20,
            order_type: OrderType::Market,
            limit_price: None,
            mode: ExecutionMode::Live,
        };

        let error = provider.submit_order(intent).await.unwrap_err();
        assert!(matches!(error, ProviderError::UnsupportedMode(_)));
    }

    #[tokio::test]
    async fn paper_execution_accepts_paper_orders() {
        let provider = PaperExecutionProvider::fixture();
        let receipt = provider
            .submit_order(OrderIntent {
                client_order_id: "decision-1".to_string(),
                symbol: "RELIANCE".to_string(),
                exchange: "NSE".to_string(),
                side: OrderSide::Buy,
                quantity: 20,
                order_type: OrderType::Market,
                limit_price: None,
                mode: ExecutionMode::Paper,
            })
            .await
            .unwrap();

        assert!(receipt.accepted);
        assert_eq!(receipt.mode, ExecutionMode::Paper);
        assert_eq!(receipt.provider_order_id, "paper_decision-1");
    }
}
