CREATE TABLE decision_inputs (
    decision_id TEXT PRIMARY KEY REFERENCES decisions(decision_id) ON DELETE CASCADE,
    model_version TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    event_json JSONB NOT NULL,
    score_json JSONB NOT NULL,
    facts_json JSONB NOT NULL,
    thresholds_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ix_decision_inputs_hash ON decision_inputs(input_hash);
CREATE INDEX ix_rule_traces_event ON rule_traces(event_id, event_version);
