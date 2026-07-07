pub mod context;
pub mod event_study;
pub mod features;
pub mod indicators;
pub mod model;
pub mod risk;
pub mod rules;
pub mod scoring;
pub mod stochastic;
pub mod taxonomy;
pub mod types;

pub use context::{MacroContext, MacroInputs, build_macro_context};
pub use features::{FeatureVector, compute_features};
pub use model::{
    CandidateAction, DECISION_MODEL_VERSION, EventStudyEvidence, ModelReport, build_model_report,
    input_hash,
};
pub use rules::{RuleDefinition, RuleRegistry, RuleResult};
pub use scoring::{AsOfFacts, DecisionInput, DecisionThresholds, decide, score_event};
pub use stochastic::{PredictionRecord, gbm_flow_prediction};
pub use taxonomy::{EventClass, classify};
pub use types::{Action, Decision, NormalizedEvent, PriceBar};
