use std::{hint::black_box, time::Instant};

use chrono::{DateTime, Utc};
use gm_domain::{
    AsOfFacts, DecisionInput, DecisionThresholds, EventStudyEvidence, NormalizedEvent,
    RuleRegistry, decide, score_event,
};

const DEFAULT_P95_MAX_MICROS: u128 = 25_000;

#[test]
fn decision_path_p95_stays_inside_release_budget() {
    let registry = RuleRegistry::builtin();
    let event = NormalizedEvent {
        event_id: "perf-earnings".to_string(),
        version: 1,
        causal_parent_id: Some("raw-perf-earnings".to_string()),
        event_type: Some("EARNINGS".to_string()),
        headline: "Quarterly earnings beat estimates".to_string(),
        body: "Profit rose and revenue grew higher than expected.".to_string(),
        occurred_at: DateTime::parse_from_rfc3339("2026-07-06T09:15:00Z")
            .expect("valid fixture timestamp")
            .with_timezone(&Utc),
        symbol: Some("RELIANCE".to_string()),
        sector: Some("Oil & Gas".to_string()),
        source: Some("NSE".to_string()),
        region: Some("IN".to_string()),
        impact_level: Some("HIGH".to_string()),
        impact_category: Some("EARNINGS".to_string()),
    };
    let facts = AsOfFacts {
        entry_price: Some(1000.0),
        exchange: Some("NSE".to_string()),
        event_study: Some(EventStudyEvidence {
            abnormal_returns: vec![0.012, 0.018, -0.004, 0.021, 0.009],
        }),
        ..AsOfFacts::default()
    };

    for _ in 0..32 {
        let score = score_event(black_box(&event), &registry);
        let input = DecisionInput {
            event: event.clone(),
            score,
            facts: facts.clone(),
            thresholds: DecisionThresholds::default(),
        };
        black_box(decide(input));
    }

    let mut samples = Vec::with_capacity(512);
    for _ in 0..512 {
        let started_at = Instant::now();
        let score = score_event(black_box(&event), &registry);
        let input = DecisionInput {
            event: event.clone(),
            score,
            facts: facts.clone(),
            thresholds: DecisionThresholds::default(),
        };
        black_box(decide(input));
        samples.push(started_at.elapsed().as_micros());
    }

    samples.sort_unstable();
    let p95 = samples[((samples.len() as f64 * 0.95).ceil() as usize).saturating_sub(1)];
    let max_p95 = std::env::var("DECISION_P95_MAX_MICROS")
        .ok()
        .and_then(|value| value.parse::<u128>().ok())
        .unwrap_or(DEFAULT_P95_MAX_MICROS);

    assert!(
        p95 <= max_p95,
        "decision path p95 {p95}us exceeded {max_p95}us; samples={samples:?}"
    );
}
