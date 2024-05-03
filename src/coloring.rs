#[macro_export]
macro_rules! color {
    (bright_blue, $s:expr) => {{
        use owo_colors::OwoColorize;
        $s.if_supports_color(owo_colors::Stream::Stderr, |t| t.bright_blue())
    }};
    (bright_cyan, $s:expr) => {{
        use owo_colors::OwoColorize;
        $s.if_supports_color(owo_colors::Stream::Stderr, |t| t.bright_cyan())
    }};
    (bright_green, $s:expr) => {{
        use owo_colors::OwoColorize;
        $s.if_supports_color(owo_colors::Stream::Stderr, |t| t.bright_green())
    }};
    (bright_purple, $s:expr) => {{
        use owo_colors::OwoColorize;
        $s.if_supports_color(owo_colors::Stream::Stderr, |t| t.bright_purple())
    }};
    (bright_red, $s:expr) => {{
        use owo_colors::OwoColorize;
        $s.if_supports_color(owo_colors::Stream::Stderr, |t| t.bright_red())
    }};
    (bright_white, $s:expr) => {{
        use owo_colors::OwoColorize;
        $s.if_supports_color(owo_colors::Stream::Stderr, |t| t.bright_white())
    }};
    (bright_yellow, $s:expr) => {{
        use owo_colors::OwoColorize;
        $s.if_supports_color(owo_colors::Stream::Stderr, |t| t.bright_yellow())
    }};
    (green, $s:expr) => {{
        use owo_colors::OwoColorize;
        $s.if_supports_color(owo_colors::Stream::Stderr, |t| t.green())
    }};
}

pub fn set_override(color: bool) {
    owo_colors::set_override(color);
}

#[cfg(test)]
mod test {
    #[test]
    fn colors_compile() {
        _ = color!(bright_blue, "");
        _ = color!(bright_cyan, "");
        _ = color!(bright_green, "");
        _ = color!(bright_purple, "");
        _ = color!(bright_red, "");
        _ = color!(bright_white, "");
        _ = color!(bright_yellow, "");
        _ = color!(green, "");
    }
}
