use super::TerminalTheme;

pub fn generate(theme: &TerminalTheme) -> String {
    let c = &theme.colors;
    format!(
        "return {{\n\
         foreground = '{}'\n\
         background = '{}'\n\
         \n\
         cursor_bg = '{}'\n\
         cursor_fg = '{}'\n\
         \n\
         black = '{}'\n\
         red = '{}'\n\
         green = '{}'\n\
         yellow = '{}'\n\
         blue = '{}'\n\
         magenta = '{}'\n\
         cyan = '{}'\n\
         white = '{}'\n\
         \n\
         bright_black = '{}'\n\
         bright_red = '{}'\n\
         bright_green = '{}'\n\
         bright_yellow = '{}'\n\
         bright_blue = '{}'\n\
         bright_magenta = '{}'\n\
         bright_cyan = '{}'\n\
         bright_white = '{}'\n\
         \n\
         selection_bg = '{}'\n\
         selection_fg = '{}'\n\
         }}",
        c.foreground,
        c.background,
        c.cursor,
        c.cursor_text,
        c.normal.black,
        c.normal.red,
        c.normal.green,
        c.normal.yellow,
        c.normal.blue,
        c.normal.magenta,
        c.normal.cyan,
        c.normal.white,
        c.bright.black,
        c.bright.red,
        c.bright.green,
        c.bright.yellow,
        c.bright.blue,
        c.bright.magenta,
        c.bright.cyan,
        c.bright.white,
        c.selection_bg,
        c.selection_fg,
    )
}
