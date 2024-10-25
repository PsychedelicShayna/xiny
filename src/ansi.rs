pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

pub fn colorize(input: &str, color: Color) -> String {
    let color_code = match color {
        Color::Black => "30",
        Color::Red => "31",
        Color::Green => "32",
        Color::Yellow => "33",
        Color::Blue => "34",
        Color::Magenta => "35",
        Color::Cyan => "36",
        Color::White => "37",
        Color::BrightBlack => "90",
        Color::BrightRed => "91",
        Color::BrightGreen => "92",
        Color::BrightYellow => "93",
        Color::BrightBlue => "94",
        Color::BrightMagenta => "95",
        Color::BrightCyan => "96",
        Color::BrightWhite => "97",
    };

    // Construct the colorized string with ANSI escape codes
    format!("\x1b[{}m{}\x1b[0m", color_code, input)
}
