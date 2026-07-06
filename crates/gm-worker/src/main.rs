use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Smoke-check worker configuration and domain availability.
    Check,
    /// Apply database migrations to a PostgreSQL database.
    Migrate {
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
        #[arg(long, default_value = "migrations")]
        migrations: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let args = Args::parse();
    match args.command {
        Command::Check => {
            let registry = gm_domain::RuleRegistry::builtin();
            tracing::info!(rule_count = registry.rules().len(), "worker ready");
        }
        Command::Migrate {
            database_url,
            migrations,
        } => {
            let store = gm_persistence::PgStore::connect(&database_url).await?;
            store.run_migrations(&migrations).await?;
            tracing::info!(path = %migrations.display(), "database migrations applied");
        }
    }
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
