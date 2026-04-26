use super::TerminalTheme;

pub fn generate(theme: &TerminalTheme) -> String {
    let c = &theme.colors;
    format!(
        "[[colors.primary]]\n\
         background = '{bg}'\n\
         foreground = '{fg}'\n\
         \n\
         [[colors.normal]]\n\
         black = '{black}'\n\
         red = '{red}'\n\
         green = '{green}'\n\
         yellow = '{yellow}'\n\
         blue = '{blue}'\n\
         magenta = '{magenta}'\n\
         cyan = '{cyan}'\n\
         white = '{white}'\n\
         \n\
         [[colors.bright]]\n\
         black = '{bright_black}'\n\
         red = '{bright_red}'\n\
         green = '{bright_green}'\n\
         yellow = '{bright_yellow}'\n\
         blue = '{bright_blue}'\n\
         magenta = '{bright_magenta}'\n\
         cyan = '{bright_cyan}'\n\
         white = '{bright_white}'\n\
         \n\
         [[colors.cursor]]\n\
         text = '{cursor_text}'\n\
         cursor = '{cursor}'",
        bg = c.background,
        fg = c.foreground,
        black = c.normal.black,
        red = c.normal.red,
        green = c.normal.green,
        yellow = c.normal.yellow,
        blue = c.normal.blue,
        magenta = c.normal.magenta,
        cyan = c.normal.cyan,
        white = c.normal.white,
        bright_black = c.bright.black,
        bright_red = c.bright.red,
        bright_green = c.bright.green,
        bright_yellow = c.bright.yellow,
        bright_blue = c.bright.blue,
        bright_magenta = c.bright.magenta,
        bright_cyan = c.bright.cyan,
        bright_white = c.bright.white,
        cursor_text = c.cursor_text,
        cursor = c.cursor,
    )
}
