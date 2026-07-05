use serde::{Deserialize, Serialize};

use crate::types::{Action, Decision};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskPolicy {
    pub max_position_notional: f64,
    pub allow_short_sell: bool,
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            max_position_notional: 100_000.0,
            allow_short_sell: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskDecision {
    pub status: RiskStatus,
    pub reasons: Vec<String>,
    pub triggered_rules: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskStatus {
    Passed,
    Rejected,
}

pub fn validate_pre_trade(decision: &Decision, policy: &RiskPolicy) -> RiskDecision {
    let mut reasons = Vec::new();
    let mut triggered_rules = Vec::new();

    if decision.action == Action::Hold {
        reasons.push("HOLD decisions are not executable".to_string());
        triggered_rules.push("NON_EXECUTABLE_HOLD".to_string());
    }

    if decision.action == Action::Sell && !policy.allow_short_sell {
        reasons.push("Short selling is disabled by policy".to_string());
        triggered_rules.push("SHORT_SELL_DISABLED".to_string());
    }

    if let (Some(quantity), Some(entry_price)) = (decision.quantity, decision.entry_price) {
        let notional = quantity as f64 * entry_price;
        if notional > policy.max_position_notional {
            reasons.push(format!(
                "Order notional {:.2} exceeds max {:.2}",
                notional, policy.max_position_notional
            ));
            triggered_rules.push("MAX_POSITION_NOTIONAL".to_string());
        }
    } else {
        reasons.push("Decision lacks executable quantity or entry price".to_string());
        triggered_rules.push("MISSING_EXECUTION_FACTS".to_string());
    }

    RiskDecision {
        status: if reasons.is_empty() {
            RiskStatus::Passed
        } else {
            RiskStatus::Rejected
        },
        reasons,
        triggered_rules,
    }
}
