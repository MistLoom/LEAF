use serde::{Deserialize, Serialize};

/// 挂在 Kb 上的场景配置；工具仅允许单次调用覆盖显式字段。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioConfig {
    #[serde(default = "default_gate_threshold")]
    pub gate_threshold: f64,
    #[serde(default = "default_dup_threshold")]
    pub dup_threshold: f64,
    #[serde(default = "default_lambda")]
    pub decay_lambda: f64,
    #[serde(default = "default_granularity")]
    pub granularity: String,
}

fn default_gate_threshold() -> f64 {
    0.0
}

fn default_dup_threshold() -> f64 {
    0.85
}

fn default_lambda() -> f64 {
    0.0
}

fn default_granularity() -> String {
    "section".into()
}

impl Default for ScenarioConfig {
    fn default() -> Self {
        Self {
            gate_threshold: default_gate_threshold(),
            dup_threshold: default_dup_threshold(),
            decay_lambda: default_lambda(),
            granularity: default_granularity(),
        }
    }
}
