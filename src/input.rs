//! Structs que espelham o JSON que o Claude Code envia no stdin.
//!
//! Regra anti-bug central: TODO campo é `Option`. Campo ausente => segmento
//! omitido, nunca crash. Campos desconhecidos são ignorados pelo serde, então
//! mudanças de schema do Claude Code não quebram o HUD.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Input {
    pub cwd: Option<String>,
    pub model: Option<Model>,
    pub workspace: Option<Workspace>,
    pub cost: Option<Cost>,
    pub context_window: Option<ContextWindow>,
    pub effort: Option<Effort>,
    pub rate_limits: Option<RateLimits>,
    #[allow(dead_code)]
    pub exceeds_200k_tokens: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub current_dir: Option<String>,
    #[allow(dead_code)]
    pub project_dir: Option<String>,
    #[allow(dead_code)]
    pub git_worktree: Option<String>,
    #[allow(dead_code)]
    pub repo: Option<Repo>,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    #[allow(dead_code)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Cost {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ContextWindow {
    pub total_input_tokens: Option<u64>,
    #[allow(dead_code)]
    pub total_output_tokens: Option<u64>,
    pub context_window_size: Option<u64>,
    pub used_percentage: Option<f64>,
    #[allow(dead_code)]
    pub remaining_percentage: Option<f64>,
    #[allow(dead_code)]
    pub current_usage: Option<CurrentUsage>,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUsage {
    #[allow(dead_code)]
    pub input_tokens: Option<u64>,
    #[allow(dead_code)]
    pub output_tokens: Option<u64>,
    #[allow(dead_code)]
    pub cache_creation_input_tokens: Option<u64>,
    #[allow(dead_code)]
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Effort {
    pub level: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RateLimits {
    pub five_hour: Option<RateWindow>,
    pub seven_day: Option<RateWindow>,
}

#[derive(Debug, Deserialize)]
pub struct RateWindow {
    pub used_percentage: Option<f64>,
    #[allow(dead_code)]
    pub resets_at: Option<i64>,
}
