CREATE TABLE raw_events (
    event_id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    payload_json JSONB NOT NULL,
    hash TEXT NOT NULL UNIQUE,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE normalized_events (
    norm_event_id TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    causal_parent_id TEXT REFERENCES raw_events(event_id),
    event_type TEXT,
    headline TEXT NOT NULL,
    body TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    symbol TEXT,
    sector TEXT,
    score DOUBLE PRECISION NOT NULL DEFAULT 0,
    event_class TEXT,
    matched_rules JSONB NOT NULL DEFAULT '[]',
    rule_results JSONB NOT NULL DEFAULT '[]',
    source TEXT,
    region TEXT,
    impact_level TEXT,
    impact_category TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (norm_event_id, version)
);

CREATE INDEX ix_normalized_events_symbol_created ON normalized_events(symbol, created_at);

CREATE TABLE macro_signals (
    signal_id TEXT PRIMARY KEY,
    signal_type TEXT NOT NULL,
    symbol TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    change_percent DOUBLE PRECISION NOT NULL,
    change_absolute DOUBLE PRECISION NOT NULL,
    currency TEXT,
    source TEXT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata_json JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE price_bars (
    symbol TEXT NOT NULL,
    date DATE NOT NULL,
    open DOUBLE PRECISION NOT NULL,
    high DOUBLE PRECISION NOT NULL,
    low DOUBLE PRECISION NOT NULL,
    close DOUBLE PRECISION NOT NULL,
    adj_close DOUBLE PRECISION,
    volume BIGINT NOT NULL DEFAULT 0,
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (symbol, date)
);

CREATE TABLE feature_snapshots (
    symbol TEXT NOT NULL,
    as_of DATE NOT NULL,
    n_bars INTEGER NOT NULL DEFAULT 0,
    features JSONB NOT NULL DEFAULT '{}',
    model_version TEXT NOT NULL DEFAULT 'v1',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (symbol, as_of)
);

CREATE TABLE prediction_records (
    symbol TEXT NOT NULL,
    as_of DATE NOT NULL,
    horizon INTEGER NOT NULL,
    model_version TEXT NOT NULL DEFAULT 'gbm-flow-v1',
    seed BIGINT NOT NULL DEFAULT 0,
    n_bars INTEGER NOT NULL DEFAULT 0,
    mu DOUBLE PRECISION NOT NULL,
    sigma DOUBLE PRECISION NOT NULL,
    flow_adjustment DOUBLE PRECISION NOT NULL DEFAULT 0,
    expected_return DOUBLE PRECISION NOT NULL,
    forecast_std DOUBLE PRECISION NOT NULL,
    quantiles JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (symbol, as_of, horizon)
);

CREATE TABLE decisions (
    decision_id TEXT PRIMARY KEY,
    portfolio_id TEXT NOT NULL DEFAULT 'DEFAULT',
    parent_event_id TEXT NOT NULL,
    parent_event_version INTEGER NOT NULL,
    action TEXT NOT NULL,
    total_score DOUBLE PRECISION NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    position_size DOUBLE PRECISION NOT NULL,
    thesis TEXT NOT NULL,
    reasons JSONB NOT NULL DEFAULT '[]',
    symbol TEXT,
    sector TEXT,
    entry_price DOUBLE PRECISION,
    quantity INTEGER,
    target_price DOUBLE PRECISION,
    stop_loss DOUBLE PRECISION,
    timing TEXT,
    exchange TEXT,
    execution_ready BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE rule_traces (
    trace_id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL,
    event_version INTEGER NOT NULL DEFAULT 1,
    rule_id TEXT NOT NULL,
    matched BOOLEAN NOT NULL,
    weight DOUBLE PRECISION NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    contribution DOUBLE PRECISION NOT NULL,
    reason TEXT NOT NULL,
    evaluated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
