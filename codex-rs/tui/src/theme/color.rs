use ratatui::style::Color;

use crate::terminal_palette::StdoutColorLevel;
use crate::terminal_palette::best_color_for_level;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemeColor {
    TerminalForeground,
    TerminalBackground,
    Transparent,
    Rgb(u8, u8, u8),
    Ansi(Color),
}

impl ThemeColor {
    pub(crate) fn to_ratatui(self, level: StdoutColorLevel) -> Option<Color> {
        match self {
            ThemeColor::TerminalForeground
            | ThemeColor::TerminalBackground
            | ThemeColor::Transparent => None,
            ThemeColor::Rgb(r, g, b) => Some(best_color_for_level((r, g, b), level)),
            ThemeColor::Ansi(color) => Some(color),
        }
    }
}

pub(crate) fn parse_color(value: &str) -> Result<ThemeColor, String> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex(hex).map_err(|err| format!("invalid hex color `{value}`: {err}"));
    }
    match value.to_ascii_lowercase().as_str() {
        "terminal.foreground" | "terminal.fg" => Ok(ThemeColor::TerminalForeground),
        "terminal.background" | "terminal.bg" => Ok(ThemeColor::TerminalBackground),
        "transparent" | "none" => Ok(ThemeColor::Transparent),
        "black" => Ok(ThemeColor::Ansi(Color::Black)),
        "red" => Ok(ThemeColor::Ansi(Color::Red)),
        "green" => Ok(ThemeColor::Ansi(Color::Green)),
        "yellow" => Ok(ThemeColor::Ansi(Color::Yellow)),
        "blue" => Ok(ThemeColor::Ansi(Color::Blue)),
        "magenta" => Ok(ThemeColor::Ansi(Color::Magenta)),
        "cyan" => Ok(ThemeColor::Ansi(Color::Cyan)),
        "gray" | "grey" => Ok(ThemeColor::Ansi(Color::Gray)),
        "white" => Ok(ThemeColor::Ansi(Color::White)),
        "dark-gray" | "dark-grey" => Ok(ThemeColor::Ansi(Color::DarkGray)),
        "light-red" => Ok(ThemeColor::Ansi(Color::LightRed)),
        "light-green" => Ok(ThemeColor::Ansi(Color::LightGreen)),
        "light-yellow" => Ok(ThemeColor::Ansi(Color::LightYellow)),
        "light-blue" => Ok(ThemeColor::Ansi(Color::LightBlue)),
        "light-magenta" => Ok(ThemeColor::Ansi(Color::LightMagenta)),
        "light-cyan" => Ok(ThemeColor::Ansi(Color::LightCyan)),
        other => Err(format!("unknown color `{other}`")),
    }
}

fn parse_hex(hex: &str) -> Result<ThemeColor, &'static str> {
    if hex.len() != 6 {
        return Err("expected 6 hex digits");
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "invalid red channel")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "invalid green channel")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "invalid blue channel")?;
    Ok(ThemeColor::Rgb(r, g, b))
}
