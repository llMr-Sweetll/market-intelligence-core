use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    Buy,
    Sell,
    Hold,
}

impl Action {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
            Self::Hold => "HOLD",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedEvent {
    pub event_id: String,
    pub version: i32,
    pub causal_parent_id: Option<String>,
    pub event_type: Option<String>,
    pub headline: String,
    pub body: String,
    pub occurred_at: DateTime<Utc>,
    pub symbol: Option<String>,
    pub sector: Option<String>,
    pub source: Option<String>,
    pub region: Option<String>,
    pub impact_level: Option<String>,
    pub impact_category: Option<String>,
}

impl NormalizedEvent {
    pub fn searchable_text(&self) -> String {
        format!("{} {}", self.headline, self.body).to_lowercase()
    }

    pub fn sector_or_general(&self) -> &str {
        self.sector.as_deref().unwrap_or("General")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceBar {
    pub symbol: String,
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: Option<f64>,
    pub volume: i64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub decision_id: String,
    pub parent_event_id: String,
    pub parent_event_version: i32,
    pub action: Action,
    pub total_score: f64,
    pub confidence: f64,
    pub position_size: f64,
    pub quantity: Option<u64>,
    pub entry_price: Option<f64>,
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub timing: Option<String>,
    pub exchange: Option<String>,
    pub symbol: Option<String>,
    pub sector: Option<String>,
    pub thesis: String,
    pub reasons: serde_json::Value,
    pub model_version: String,
    pub input_hash: String,
    pub expected_return: Option<f64>,
    pub downside: Option<f64>,
    pub explanation: serde_json::Value,
    pub execution_ready: bool,
}
