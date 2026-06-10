//! Detecção de cor. Respeita a convenção NO_COLOR (https://no-color.org).

/// Cor habilitada a menos que a env var NO_COLOR esteja presente (qualquer valor).
pub fn color_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}
