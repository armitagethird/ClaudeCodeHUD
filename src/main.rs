//! HUD — statusline para Claude Code.
//!
//! Princípio central anti-bug: ler o JSON do stdin, formatar uma linha, imprimir.
//! Nunca dá panic visível; em QUALQUER erro imprime fallback vazio e sai com 0,
//! para que a barra de status do Claude Code jamais quebre.

mod color;
mod config;
mod git;
mod input;
mod render;

use std::io::{Read, Write};
use std::path::Path;

fn main() {
    // run() devolve None em qualquer falha (stdin malformado, etc.).
    // unwrap_or_default() => string vazia => statusline em branco, nunca crash.
    let line = run().unwrap_or_default();
    let mut out = std::io::stdout();
    let _ = out.write_all(line.as_bytes());
    let _ = out.flush();
}

fn run() -> Option<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).ok()?;

    let parsed: input::Input = serde_json::from_str(&buf).ok()?;
    let cfg = config::load();
    let theme = render::Theme::from_config(&cfg, color::color_enabled());

    // Branch lida de .git/HEAD a partir do cwd (sem subprocess).
    let cwd = parsed
        .cwd
        .clone()
        .or_else(|| parsed.workspace.as_ref().and_then(|w| w.current_dir.clone()));
    let branch = cwd.as_deref().and_then(|c| git::current_branch(Path::new(c)));

    Some(render::render_line(&parsed, &cfg, &theme, &branch))
}
