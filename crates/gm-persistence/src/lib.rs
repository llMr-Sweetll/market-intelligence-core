use std::path::Path;

use anyhow::Context;
use gm_domain::{Decision, NormalizedEvent, PriceBar};
use sqlx::{PgPool, postgres::PgPoolOptions};

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

    pub async fn run_migrations(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let migrator = sqlx::migrate::Migrator::new(path.as_ref()).await?;
        migrator.run(&self.pool).await?;
        Ok(())
    }

    pub async fn save_normalized_event(&self, event: &NormalizedEvent) -> anyhow::Result<()> {
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
}
