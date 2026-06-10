//! Branch git lendo `.git/HEAD` diretamente — sem spawnar o `git`.
//! Isso elimina locks travados / lentidão (o bug clássico das statuslines).

use std::path::{Path, PathBuf};

/// Sobe diretórios a partir de `start` até achar `.git/HEAD` e devolve a branch.
/// HEAD destacado (sha cru) ou repositório ausente => None (segmento omitido).
pub fn current_branch(start: &Path) -> Option<String> {
    let head = find_head(start)?;
    let content = std::fs::read_to_string(head).ok()?;
    parse_head(&content)
}

fn find_head(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        let head = d.join(".git").join("HEAD");
        if head.is_file() {
            return Some(head);
        }
        // `.git` pode ser um arquivo (worktree/submódulo): "gitdir: <caminho>".
        let gitfile = d.join(".git");
        if gitfile.is_file() {
            if let Ok(text) = std::fs::read_to_string(&gitfile) {
                if let Some(p) = text.lines().next().and_then(|l| l.strip_prefix("gitdir:")) {
                    let h = PathBuf::from(p.trim()).join("HEAD");
                    if h.is_file() {
                        return Some(h);
                    }
                }
            }
        }
        dir = d.parent();
    }
    None
}

/// "ref: refs/heads/feature/x" => "feature/x"; sha cru (detached) => None.
pub fn parse_head(content: &str) -> Option<String> {
    let line = content.lines().next()?.trim();
    let rest = line.strip_prefix("ref:")?.trim();
    let branch = rest.strip_prefix("refs/heads/").unwrap_or(rest);
    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::parse_head;

    #[test]
    fn simple_branch() {
        assert_eq!(parse_head("ref: refs/heads/main\n").as_deref(), Some("main"));
    }

    #[test]
    fn nested_branch_keeps_slashes() {
        assert_eq!(
            parse_head("ref: refs/heads/feature/cool-thing\n").as_deref(),
            Some("feature/cool-thing")
        );
    }

    #[test]
    fn detached_head_is_none() {
        assert_eq!(parse_head("9f1a2b3c4d5e6f\n"), None);
    }

    #[test]
    fn empty_is_none() {
        assert_eq!(parse_head(""), None);
    }
}
