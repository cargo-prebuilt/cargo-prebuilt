pub enum PossibleColor {
    BrightBlue,
    BrightCyan,
    BrightGreen,
    BrightMagenta,
    BrightPurple,
    BrightRed,
    BrightWhite,
    BrightYellow,
    Green,
}

pub fn set_override(color: bool) {
    #[cfg(feature = "color")]
    {
        owo_colors::set_override(color);
    }

    #[cfg(not(feature = "color"))]
    {}
}

#[cfg(feature = "color")]
pub fn err_color_print(str: &str, color: &PossibleColor) -> String {
    use owo_colors::{OwoColorize, Stream::Stderr};

    match color {
        PossibleColor::BrightBlue => str
            .if_supports_color(Stderr, |t| t.bright_blue())
            .to_string(),
        PossibleColor::BrightCyan => str
            .if_supports_color(Stderr, |t| t.bright_cyan())
            .to_string(),
        PossibleColor::BrightGreen => str
            .if_supports_color(Stderr, |t| t.bright_green())
            .to_string(),
        PossibleColor::BrightMagenta => str
            .if_supports_color(Stderr, |t| t.bright_magenta())
            .to_string(),
        PossibleColor::BrightPurple => str
            .if_supports_color(Stderr, |t| t.bright_purple())
            .to_string(),
        PossibleColor::BrightRed => str
            .if_supports_color(Stderr, |t| t.bright_red())
            .to_string(),
        PossibleColor::BrightWhite => str
            .if_supports_color(Stderr, |t| t.bright_white())
            .to_string(),
        PossibleColor::BrightYellow => str
            .if_supports_color(Stderr, |t| t.bright_yellow())
            .to_string(),
        PossibleColor::Green => str.if_supports_color(Stderr, |t| t.green()).to_string(),
    }
}

#[cfg(not(feature = "color"))]
#[inline(always)]
pub fn err_color_print(str: &str, _color: &PossibleColor) -> String {
    str.to_string()
}
