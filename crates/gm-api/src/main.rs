use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use gm_api::{ApiConfig, build_app};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "GM_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "GM_PORT", default_value_t = 8000)]
    port: u16,
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,
    #[arg(long, env = "GM_MIGRATIONS", default_value = "migrations")]
    migrations: PathBuf,
    #[arg(long, env = "GM_SKIP_MIGRATIONS", default_value_t = false)]
    skip_migrations: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let args = Args::parse();
    let app = build_app(ApiConfig {
        database_url: args.database_url,
        migrations: args.migrations,
        run_migrations: !args.skip_migrations,
    })
    .await?;

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
