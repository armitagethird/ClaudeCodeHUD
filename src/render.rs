//! Montagem da linha do HUD: medidor, badge de carga, formatação e ajuste de
//! largura. Funções puras (testáveis) + montagem final.

use crate::config::Config;
use crate::input::Input;

// ---------------------------------------------------------------------------
// Núcleo puro (sem cor / sem I/O) — fácil de testar.
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum Load {
    Leve,
    Medio,
    Alto,
}

pub fn classify_load(pct: f64, medio: f64, alto: f64) -> Load {
    if pct >= alto {
        Load::Alto
    } else if pct >= medio {
        Load::Medio
    } else {
        Load::Leve
    }
}

/// Medidor `▓▓▓░░░` (ou `###---` em ascii) com `width` caracteres.
pub fn render_gauge(pct: f64, width: usize, ascii: bool) -> String {
    let pct = pct.clamp(0.0, 100.0);
    let filled = (((pct / 100.0) * width as f64).round() as usize).min(width);
    let (fc, ec) = if ascii { ('#', '-') } else { ('▓', '░') };
    let mut s = String::with_capacity(width * 3);
    for _ in 0..filled {
        s.push(fc);
    }
    for _ in filled..width {
        s.push(ec);
    }
    s
}

/// 850 → "850", 8500 → "8k", 200000 → "200k", 1_000_000 → "1M", 1_500_000 → "1.5M".
pub fn human_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        let s = format!("{:.1}", n as f64 / 1_000_000.0);
        let s = s.trim_end_matches(".0");
        format!("{}M", s)
    } else if n >= 1000 {
        format!("{}k", n / 1000)
    } else {
        n.to_string()
    }
}

/// 45000ms → "45s", 720000 → "12m", 3_600_000 → "1h", 3_660_000 → "1h1m".
pub fn human_duration(ms: u64) -> String {
    let secs = ms / 1000;
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if m == 0 {
            format!("{}h", h)
        } else {
            format!("{}h{}m", h, m)
        }
    }
}

/// Largura visível (colunas), ignorando sequências ANSI `\x1b[...m`.
pub fn visible_width(s: &str) -> usize {
    let mut w = 0usize;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for n in chars.by_ref() {
                if n == 'm' {
                    break;
                }
            }
        } else {
            w += 1;
        }
    }
    w
}

// ---------------------------------------------------------------------------
// Tema / cor.
// ---------------------------------------------------------------------------

pub struct Theme {
    pub enabled: bool,
    pub ok: u8,
    pub warn: u8,
    pub crit: u8,
    pub dim: u8,
}

impl Theme {
    pub fn from_config(cfg: &Config, enabled: bool) -> Self {
        Theme {
            enabled,
            ok: cfg.colors.ok,
            warn: cfg.colors.warn,
            crit: cfg.colors.crit,
            dim: cfg.colors.dim,
        }
    }

    fn paint(&self, code: u8, s: &str) -> String {
        if self.enabled && !s.is_empty() {
            format!("\x1b[38;5;{}m{}\x1b[0m", code, s)
        } else {
            s.to_string()
        }
    }

    fn load_color(&self, l: &Load) -> u8 {
        match l {
            Load::Leve => self.ok,
            Load::Medio => self.warn,
            Load::Alto => self.crit,
        }
    }

    fn pct_color(&self, pct: f64) -> u8 {
        if pct >= 80.0 {
            self.crit
        } else if pct >= 50.0 {
            self.warn
        } else {
            self.ok
        }
    }
}

// ---------------------------------------------------------------------------
// Segmentos + ajuste de largura.
// ---------------------------------------------------------------------------

pub struct Segment {
    pub display: String,
    /// Prioridade de corte: menor cai primeiro quando a linha não cabe.
    /// >= PROTECT nunca é descartado.
    pub drop: u32,
}

const PROTECT: u32 = 100;

/// Remove segmentos de menor prioridade até caber em `cols`, preservando os
/// protegidos (model + medidor de contexto) e a ordem original.
pub fn fit_width(segs: Vec<Segment>, sep: &str, cols: Option<usize>) -> String {
    let sep_w = sep.chars().count();
    let mut kept = segs;
    if let Some(max) = cols {
        loop {
            let vis: usize = kept.iter().map(|s| visible_width(&s.display)).sum::<usize>()
                + sep_w * kept.len().saturating_sub(1);
            if vis <= max || kept.len() <= 1 {
                break;
            }
            let cand = kept
                .iter()
                .enumerate()
                .filter(|(_, s)| s.drop < PROTECT)
                .min_by_key(|(_, s)| s.drop)
                .map(|(i, _)| i);
            match cand {
                Some(i) => {
                    kept.remove(i);
                }
                None => break,
            }
        }
    }
    kept.iter()
        .map(|s| s.display.as_str())
        .collect::<Vec<_>>()
        .join(sep)
}

fn columns() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
}

fn model_name(input: &Input, cfg: &Config) -> Option<String> {
    let m = input.model.as_ref()?;
    if let Some(id) = m.id.as_deref() {
        if let Some(name) = cfg.model_names.get(id) {
            return Some(name.clone());
        }
    }
    m.display_name.clone().or_else(|| m.id.clone())
}

fn used_pct(input: &Input) -> Option<f64> {
    input.context_window.as_ref().and_then(|c| c.used_percentage)
}

fn context_gauge(input: &Input, cfg: &Config, theme: &Theme) -> Option<String> {
    let pct = used_pct(input)?;
    let load = classify_load(pct, cfg.load_medio, cfg.load_alto);
    let color = theme.load_color(&load);
    let bar = render_gauge(pct, cfg.gauge_width, cfg.ascii_mode);
    Some(format!(
        "{} {} {}",
        theme.paint(theme.dim, "ctx"),
        theme.paint(color, &bar),
        theme.paint(color, &format!("{:.0}%", pct))
    ))
}

fn tokens(input: &Input) -> Option<String> {
    let cw = input.context_window.as_ref()?;
    let used = cw.total_input_tokens?;
    let size = cw.context_window_size.unwrap_or(200_000);
    Some(format!("{}/{}", human_tokens(used), human_tokens(size)))
}

fn load_badge(input: &Input, cfg: &Config, theme: &Theme) -> Option<String> {
    let pct = used_pct(input)?;
    let load = classify_load(pct, cfg.load_medio, cfg.load_alto);
    let (txt, color) = match load {
        Load::Leve => ("LEVE", theme.ok),
        Load::Medio => ("MÉDIO", theme.warn),
        Load::Alto => ("ALTO", theme.crit),
    };
    Some(format!(
        "{} {}",
        theme.paint(theme.dim, "carga"),
        theme.paint(color, txt)
    ))
}

fn rate_limit(input: &Input, theme: &Theme) -> Option<String> {
    let rl = input.rate_limits.as_ref()?;
    let mut parts: Vec<String> = Vec::new();
    if let Some(p) = rl.five_hour.as_ref().and_then(|w| w.used_percentage) {
        parts.push(format!(
            "5h {}",
            theme.paint(theme.pct_color(p), &format!("{:.0}%", p))
        ));
    }
    if let Some(p) = rl.seven_day.as_ref().and_then(|w| w.used_percentage) {
        parts.push(format!(
            "7d {}",
            theme.paint(theme.pct_color(p), &format!("{:.0}%", p))
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

/// Monta a linha completa do HUD na ordem de `cfg.segments`, omitindo segmentos
/// sem dados, e ajusta à largura do terminal.
pub fn render_line(input: &Input, cfg: &Config, theme: &Theme, branch: &Option<String>) -> String {
    let mut segs: Vec<Segment> = Vec::new();
    for name in &cfg.segments {
        let (drop, text): (u32, Option<String>) = match name.as_str() {
            "model" => (101, model_name(input, cfg)),
            "effort" => (7, input.effort.as_ref().and_then(|e| e.level.clone())),
            "context_gauge" => (PROTECT, context_gauge(input, cfg, theme)),
            "tokens" => (5, tokens(input)),
            "load" => (6, load_badge(input, cfg, theme)),
            "cost" => (
                4,
                input
                    .cost
                    .as_ref()
                    .and_then(|c| c.total_cost_usd)
                    .map(|v| format!("${:.2}", v)),
            ),
            "rate_limit" => (3, rate_limit(input, theme)),
            "branch" => (2, branch.clone()),
            "timer" => (
                1,
                input
                    .cost
                    .as_ref()
                    .and_then(|c| c.total_duration_ms)
                    .map(human_duration),
            ),
            _ => (50, None),
        };
        if let Some(t) = text {
            if !t.is_empty() {
                segs.push(Segment { display: t, drop });
            }
        }
    }
    fit_width(segs, &cfg.separator, columns())
}

// ---------------------------------------------------------------------------
// Testes unitários das funções puras.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_filled_counts() {
        let f = |s: &str| s.chars().filter(|&c| c == '▓').count();
        assert_eq!(f(&render_gauge(0.0, 16, false)), 0);
        assert_eq!(f(&render_gauge(42.0, 16, false)), 7); // round(6.72) = 7
        assert_eq!(f(&render_gauge(100.0, 16, false)), 16);
        assert_eq!(render_gauge(100.0, 16, false).chars().count(), 16);
    }

    #[test]
    fn gauge_ascii_mode() {
        assert_eq!(render_gauge(50.0, 10, true), "#####-----");
    }

    #[test]
    fn gauge_clamps_out_of_range() {
        assert_eq!(render_gauge(250.0, 4, true), "####");
        assert_eq!(render_gauge(-10.0, 4, true), "----");
    }

    #[test]
    fn load_classification_at_thresholds() {
        assert_eq!(classify_load(49.0, 50.0, 80.0), Load::Leve);
        assert_eq!(classify_load(50.0, 50.0, 80.0), Load::Medio);
        assert_eq!(classify_load(80.0, 50.0, 80.0), Load::Alto);
        assert_eq!(classify_load(81.0, 50.0, 80.0), Load::Alto);
    }

    #[test]
    fn tokens_formatting() {
        assert_eq!(human_tokens(850), "850");
        assert_eq!(human_tokens(8500), "8k");
        assert_eq!(human_tokens(200_000), "200k");
        assert_eq!(human_tokens(1_000_000), "1M");
        assert_eq!(human_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn duration_formatting() {
        assert_eq!(human_duration(45_000), "45s");
        assert_eq!(human_duration(720_000), "12m");
        assert_eq!(human_duration(3_600_000), "1h");
        assert_eq!(human_duration(3_660_000), "1h1m");
    }

    #[test]
    fn visible_width_ignores_ansi() {
        assert_eq!(visible_width("\x1b[38;5;2mok\x1b[0m"), 2);
        assert_eq!(visible_width("ctx ▓▓░░ 42%"), 12);
    }

    #[test]
    fn fit_drops_lowest_priority_first_preserving_protected() {
        let segs = vec![
            Segment { display: "MODEL".into(), drop: 101 },
            Segment { display: "GAUGE".into(), drop: 100 },
            Segment { display: "effort".into(), drop: 7 },
            Segment { display: "timer".into(), drop: 1 },
        ];
        let out = fit_width(segs, " ", Some(17));
        assert!(out.contains("MODEL"));
        assert!(out.contains("GAUGE"));
        assert!(!out.contains("timer")); // drop=1 cai primeiro
        assert!(!out.contains("effort")); // drop=7 cai em seguida
    }

    #[test]
    fn fit_no_columns_keeps_everything() {
        let segs = vec![
            Segment { display: "a".into(), drop: 1 },
            Segment { display: "b".into(), drop: 2 },
        ];
        assert_eq!(fit_width(segs, " | ", None), "a | b");
    }
}
