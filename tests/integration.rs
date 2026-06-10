//! Testes de integração: alimentam JSON via stdin no binário compilado e
//! verificam a saída + ausência de crash. Cobrem a garantia anti-bug central.

use std::io::Write;
use std::process::{Command, Stdio};

/// Roda o binário com `stdin` e (opcionalmente) uma largura COLUMNS.
/// NO_COLOR=1 garante saída sem ANSI (asserts estáveis).
/// HUD_CONFIG aponta p/ caminho inexistente => defaults (ignora hud.toml real).
fn run_hud(stdin: &str, columns: Option<&str>) -> (String, i32) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hud"));
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.env("NO_COLOR", "1");
    cmd.env("HUD_CONFIG", "definitely-nonexistent-hud-config.toml");
    match columns {
        Some(c) => {
            cmd.env("COLUMNS", c);
        }
        None => {
            cmd.env_remove("COLUMNS");
        }
    }
    let mut child = cmd.spawn().expect("falha ao spawnar o binário hud");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("fixture ausente: {path}"))
}

#[test]
fn full_renders_core_and_extras() {
    let (out, code) = run_hud(&fixture("full.json"), None);
    assert_eq!(code, 0, "saída: {out}");
    assert!(out.contains("Opus 4.8"), "saída: {out}");
    assert!(out.contains("high"), "saída: {out}");
    assert!(out.contains("42%"), "saída: {out}");
    assert!(out.contains("84k/200k"), "saída: {out}");
    assert!(out.contains("LEVE"), "saída: {out}"); // 42% < 50% => LEVE
    assert!(out.contains("$0.23"), "saída: {out}");
    assert!(out.contains("12m"), "saída: {out}"); // timer
}

#[test]
fn minimal_shows_only_model() {
    let (out, code) = run_hud(&fixture("minimal.json"), None);
    assert_eq!(code, 0, "saída: {out}");
    assert!(out.contains("Sonnet 4.6"), "saída: {out}");
    assert!(!out.contains('%'), "saída: {out}");
}

#[test]
fn missing_context_omits_gauge() {
    let (out, code) = run_hud(&fixture("no_context.json"), None);
    assert_eq!(code, 0, "saída: {out}");
    assert!(out.contains("Opus 4.8"), "saída: {out}");
    assert!(out.contains("high"), "saída: {out}");
    assert!(!out.contains("ctx"), "saída: {out}");
}

#[test]
fn garbage_stdin_never_crashes() {
    let (out, code) = run_hud("}{ not json at all", None);
    assert_eq!(code, 0);
    assert!(out.trim().is_empty(), "saída: {out}");
}

#[test]
fn empty_stdin_is_safe() {
    let (out, code) = run_hud("", None);
    assert_eq!(code, 0);
    assert!(out.trim().is_empty(), "saída: {out}");
}

#[test]
fn narrow_width_drops_low_priority_keeps_core() {
    let (out, code) = run_hud(&fixture("full.json"), Some("40"));
    assert_eq!(code, 0, "saída: {out}");
    assert!(out.contains("Opus 4.8"), "saída: {out}");
    assert!(out.contains("42%"), "saída: {out}"); // medidor protegido
    assert!(!out.contains("12m"), "saída: {out}"); // timer descartado
}
