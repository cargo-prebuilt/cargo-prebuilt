#[cfg(feature = "color")]
use std::sync::atomic::{AtomicBool, Ordering};

pub enum PossibleColor {
    BrightBlue,
    BrightCyan,
    BrightGreen,
    BrightMagenta,
    BrightPurple,
    BrightRed,
    Green,
}

#[cfg(feature = "color")]
static COLOR_SUPPORT_ERR: AtomicBool = AtomicBool::new(false);

pub fn from_stream() {
    #[cfg(feature = "color")]
    {
        match supports_color::on(supports_color::Stream::Stderr) {
            Some(_) => COLOR_SUPPORT_ERR.store(true, Ordering::Relaxed),
            None => COLOR_SUPPORT_ERR.store(false, Ordering::Relaxed),
        }
    }
}

pub fn set_override(color: bool) {
    #[cfg(feature = "color")]
    {
        COLOR_SUPPORT_ERR.store(color, Ordering::Relaxed)
    }

    #[cfg(not(feature = "color"))]
    let _ = color;
}

#[cfg(feature = "color")]
pub fn err_color_print(str: &str, color: PossibleColor) -> String {
    use owo_colors::OwoColorize;

    if !COLOR_SUPPORT_ERR.load(Ordering::Relaxed) {
        return str.to_string();
    }

    match color {
        PossibleColor::BrightBlue => str.bright_blue().to_string(),
        PossibleColor::BrightCyan => str.bright_cyan().to_string(),
        PossibleColor::BrightGreen => str.bright_green().to_string(),
        PossibleColor::BrightMagenta => str.bright_magenta().to_string(),
        PossibleColor::BrightPurple => str.bright_purple().to_string(),
        PossibleColor::BrightRed => str.bright_red().to_string(),
        PossibleColor::Green => str.green().to_string(),
    }
}

#[cfg(not(feature = "color"))]
#[inline(always)]
pub fn err_color_print(str: &str, _color: PossibleColor) -> String {
    str.to_string()
}
