//! Configuração lida de `~/.claude/hud.toml` (ou do caminho em $HUD_CONFIG).
//! Todos os campos são opcionais: ausência => default embutido.
//! Qualquer erro de leitura/parse => defaults (nunca crash).

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    /// true => medidor em ASCII `[#----]` em vez de bloco Unicode `▓░`.
    pub ascii_mode: bool,
    /// Largura (em caracteres) do medidor de contexto.
    pub gauge_width: usize,
    /// Separador entre segmentos.
    pub separator: String,
    /// Limiar (% de contexto) para carga MÉDIO. Use decimal (ex: 50.0).
    pub load_medio: f64,
    /// Limiar (% de contexto) para carga ALTO. Use decimal (ex: 80.0).
    pub load_alto: f64,
    /// Ordem e ativação dos segmentos (esquerda → direita).
    pub segments: Vec<String>,
    pub colors: Colors,
    /// Mapa id-do-modelo → rótulo bonito.
    pub model_names: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Colors {
    pub ok: u8,
    pub warn: u8,
    pub crit: u8,
    pub dim: u8,
}

impl Default for Colors {
    fn default() -> Self {
        // Cores ANSI 256: verde / amarelo / vermelho / cinza.
        Colors {
            ok: 2,
            warn: 3,
            crit: 1,
            dim: 8,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ascii_mode: false,
            gauge_width: 16,
            separator: "  ·  ".to_string(),
            load_medio: 50.0,
            load_alto: 80.0,
            segments: [
                "model",
                "effort",
                "context_gauge",
                "tokens",
                "load",
                "cost",
                "rate_limit",
                "branch",
                "timer",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            colors: Colors::default(),
            model_names: default_model_names(),
        }
    }
}

fn default_model_names() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("claude-opus-4-8".to_string(), "Opus 4.8".to_string());
    m.insert("claude-sonnet-4-6".to_string(), "Sonnet 4.6".to_string());
    m.insert("claude-haiku-4-5-20251001".to_string(), "Haiku 4.5".to_string());
    m.insert("claude-fable-5".to_string(), "Fable 5".to_string());
    m
}

pub fn load() -> Config {
    if let Some(path) = config_path() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str::<Config>(&text) {
                return cfg;
            }
        }
    }
    Config::default()
}

fn config_path() -> Option<PathBuf> {
    if let Some(p) = env::var_os("HUD_CONFIG") {
        return Some(PathBuf::from(p));
    }
    home_dir().map(|h| h.join(".claude").join("hud.toml"))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
}
