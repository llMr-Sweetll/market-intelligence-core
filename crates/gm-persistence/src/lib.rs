use std::path::Path;

use anyhow::Context;
use chrono::{DateTime, Utc};
use gm_domain::{
    Decision, DecisionInput, NormalizedEvent, PriceBar, input_hash, scoring::ScoreOutput,
};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct PaymentOrderRecord {
    pub provider_order_id: String,
    pub provider: String,
    pub account_id: String,
    pub receipt: String,
    pub amount_paise: i64,
    pub currency: String,
    pub status: String,
    pub test_mode: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaymentEventRecord {
    pub event_id: String,
    pub provider: String,
    pub event_type: String,
    pub provider_order_id: Option<String>,
    pub provider_payment_id: Option<String>,
    pub verified: bool,
    pub payload_json: serde_json::Value,
    pub received_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .with_context(|| "failed to connect to Postgres")?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn ping(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn run_migrations(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let migrator = sqlx::migrate::Migrator::new(path.as_ref()).await?;
        migrator.run(&self.pool).await?;
        Ok(())
    }

    pub async fn save_normalized_event(&self, event: &NormalizedEvent) -> anyhow::Result<()> {
        self.ensure_causal_parent(event).await?;
        sqlx::query(
            r#"
            INSERT INTO normalized_events (
                norm_event_id, version, causal_parent_id, event_type, headline, body,
                occurred_at, symbol, sector, source, region, impact_level, impact_category
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            ON CONFLICT (norm_event_id, version) DO NOTHING
            "#,
        )
        .bind(&event.event_id)
        .bind(event.version)
        .bind(&event.causal_parent_id)
        .bind(&event.event_type)
        .bind(&event.headline)
        .bind(&event.body)
        .bind(event.occurred_at)
        .bind(&event.symbol)
        .bind(&event.sector)
        .bind(&event.source)
        .bind(&event.region)
        .bind(&event.impact_level)
        .bind(&event.impact_category)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_causal_parent(&self, event: &NormalizedEvent) -> anyhow::Result<()> {
        let Some(parent_id) = event.causal_parent_id.as_ref() else {
            return Ok(());
        };
        let source = event.source.as_deref().unwrap_or("unknown");
        let payload = serde_json::json!({
            "event_id": parent_id,
            "normalized_event_id": event.event_id,
            "headline": event.headline,
            "source": source,
        });
        let hash_seed = format!("raw-parent:{parent_id}");
        let hash = Uuid::new_v5(&Uuid::NAMESPACE_URL, hash_seed.as_bytes()).to_string();

        sqlx::query(
            r#"
            INSERT INTO raw_events (event_id, source, payload_json, hash)
            VALUES ($1,$2,$3,$4)
            ON CONFLICT (event_id) DO NOTHING
            "#,
        )
        .bind(parent_id)
        .bind(source)
        .bind(payload)
        .bind(hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_score_projection(
        &self,
        event: &NormalizedEvent,
        score: &ScoreOutput,
    ) -> anyhow::Result<()> {
        let event_class = serde_json::to_value(score.event_class)?
            .as_str()
            .unwrap_or("GENERAL")
            .to_string();

        sqlx::query(
            r#"
            UPDATE normalized_events
            SET score = $3,
                event_class = $4,
                matched_rules = $5,
                rule_results = $6
            WHERE norm_event_id = $1 AND version = $2
            "#,
        )
        .bind(&event.event_id)
        .bind(event.version)
        .bind(score.score)
        .bind(event_class)
        .bind(serde_json::to_value(&score.matched_rules)?)
        .bind(serde_json::to_value(&score.rule_results)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_rule_traces(
        &self,
        event: &NormalizedEvent,
        score: &ScoreOutput,
    ) -> anyhow::Result<()> {
        for result in &score.rule_results {
            let trace_seed = format!("{}:{}:{}", event.event_id, event.version, result.rule_id);
            let trace_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, trace_seed.as_bytes()).to_string();
            sqlx::query(
                r#"
                INSERT INTO rule_traces (
                    trace_id, event_id, event_version, rule_id, matched, weight,
                    confidence, contribution, reason
                )
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                ON CONFLICT (trace_id) DO NOTHING
                "#,
            )
            .bind(trace_id)
            .bind(&event.event_id)
            .bind(event.version)
            .bind(&result.rule_id)
            .bind(result.matched)
            .bind(result.weight)
            .bind(result.confidence)
            .bind(result.contribution)
            .bind(&result.reason)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn save_price_bar(&self, bar: &PriceBar) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO price_bars (
                symbol, date, open, high, low, close, adj_close, volume, source
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            ON CONFLICT (symbol, date) DO NOTHING
            "#,
        )
        .bind(&bar.symbol)
        .bind(bar.date)
        .bind(bar.open)
        .bind(bar.high)
        .bind(bar.low)
        .bind(bar.close)
        .bind(bar.adj_close)
        .bind(bar.volume)
        .bind(&bar.source)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_decision(&self, decision: &Decision) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO decisions (
                decision_id, parent_event_id, parent_event_version, action, total_score,
                confidence, position_size, thesis, reasons, symbol, sector, entry_price,
                quantity, target_price, stop_loss, timing, exchange, execution_ready
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
            ON CONFLICT (decision_id) DO NOTHING
            "#,
        )
        .bind(&decision.decision_id)
        .bind(&decision.parent_event_id)
        .bind(decision.parent_event_version)
        .bind(decision.action.as_str())
        .bind(decision.total_score)
        .bind(decision.confidence)
        .bind(decision.position_size)
        .bind(&decision.thesis)
        .bind(&decision.reasons)
        .bind(&decision.symbol)
        .bind(&decision.sector)
        .bind(decision.entry_price)
        .bind(decision.quantity.map(|quantity| quantity as i64))
        .bind(decision.target_price)
        .bind(decision.stop_loss)
        .bind(&decision.timing)
        .bind(&decision.exchange)
        .bind(decision.execution_ready)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_decision_input(
        &self,
        input: &DecisionInput,
        decision: &Decision,
        model_version: &str,
    ) -> anyhow::Result<()> {
        let event_json = serde_json::to_value(&input.event)?;
        let score_json = serde_json::to_value(&input.score)?;
        let facts_json = serde_json::to_value(&input.facts)?;
        let thresholds_json = serde_json::to_value(input.thresholds)?;
        let input_hash = input_hash(input);

        sqlx::query(
            r#"
            INSERT INTO decision_inputs (
                decision_id, model_version, input_hash, event_json, score_json,
                facts_json, thresholds_json
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            ON CONFLICT (decision_id) DO NOTHING
            "#,
        )
        .bind(&decision.decision_id)
        .bind(model_version)
        .bind(input_hash)
        .bind(event_json)
        .bind(score_json)
        .bind(facts_json)
        .bind(thresholds_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_decision_audit(
        &self,
        input: &DecisionInput,
        decision: &Decision,
        model_version: &str,
    ) -> anyhow::Result<()> {
        self.save_normalized_event(&input.event).await?;
        self.save_score_projection(&input.event, &input.score)
            .await?;
        self.save_rule_traces(&input.event, &input.score).await?;
        self.save_decision(decision).await?;
        self.save_decision_input(input, decision, model_version)
            .await?;
        Ok(())
    }

    pub async fn save_payment_order(&self, order: &PaymentOrderRecord) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO payment_orders (
                provider_order_id, provider, account_id, receipt, amount_paise,
                currency, status, test_mode
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            ON CONFLICT (provider_order_id) DO UPDATE
            SET status = EXCLUDED.status
            "#,
        )
        .bind(&order.provider_order_id)
        .bind(&order.provider)
        .bind(&order.account_id)
        .bind(&order.receipt)
        .bind(order.amount_paise)
        .bind(&order.currency)
        .bind(&order.status)
        .bind(order.test_mode)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_payment_event(&self, event: &PaymentEventRecord) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO payment_events (
                event_id, provider, event_type, provider_order_id,
                provider_payment_id, verified, payload_json, received_at
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            ON CONFLICT (event_id) DO NOTHING
            "#,
        )
        .bind(&event.event_id)
        .bind(&event.provider)
        .bind(&event.event_type)
        .bind(&event.provider_order_id)
        .bind(&event.provider_payment_id)
        .bind(event.verified)
        .bind(&event.payload_json)
        .bind(event.received_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn recent_payment_events(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<PaymentEventRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                event_id,
                provider,
                event_type,
                provider_order_id,
                provider_payment_id,
                verified,
                payload_json,
                received_at
            FROM payment_events
            ORDER BY received_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(PaymentEventRecord {
                    event_id: row.try_get("event_id")?,
                    provider: row.try_get("provider")?,
                    event_type: row.try_get("event_type")?,
                    provider_order_id: row.try_get("provider_order_id")?,
                    provider_payment_id: row.try_get("provider_payment_id")?,
                    verified: row.try_get("verified")?,
                    payload_json: row.try_get("payload_json")?,
                    received_at: row.try_get("received_at")?,
                })
            })
            .collect()
    }
}
