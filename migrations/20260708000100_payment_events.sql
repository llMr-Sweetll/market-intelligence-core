CREATE TABLE payment_orders (
    provider_order_id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    account_id TEXT NOT NULL,
    receipt TEXT NOT NULL UNIQUE,
    amount_paise BIGINT NOT NULL,
    currency TEXT NOT NULL,
    status TEXT NOT NULL,
    test_mode BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ix_payment_orders_account_created ON payment_orders(account_id, created_at DESC);

CREATE TABLE payment_events (
    event_id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    event_type TEXT NOT NULL,
    provider_order_id TEXT,
    provider_payment_id TEXT,
    verified BOOLEAN NOT NULL,
    payload_json JSONB NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ix_payment_events_received ON payment_events(received_at DESC);
CREATE INDEX ix_payment_events_order ON payment_events(provider_order_id);
