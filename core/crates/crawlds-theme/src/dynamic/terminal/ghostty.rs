use super::TerminalTheme;

pub fn generate(theme: &TerminalTheme) -> String {
    let c = &theme.colors;
    format!(
        "[colors]\n\
         background = {bg}\n\
         foreground = {fg}\n\
         \n\
         palette0 = {black}\n\
         palette1 = {red}\n\
         palette2 = {green}\n\
         palette3 = {yellow}\n\
         palette4 = {blue}\n\
         palette5 = {magenta}\n\
         palette6 = {cyan}\n\
         palette7 = {white}\n\
         palette8 = {bright_black}\n\
         palette9 = {bright_red}\n\
         palette10 = {bright_green}\n\
         palette11 = {bright_yellow}\n\
         palette12 = {bright_blue}\n\
         palette13 = {bright_magenta}\n\
         palette14 = {bright_cyan}\n\
         palette15 = {bright_white}\n\
         \n\
         selection-background = {selection_bg}\n\
         selection-foreground = {selection_fg}\n\
         \n\
         cursor = {cursor}\n\
         cursor-fg = {cursor_text}",
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
        selection_bg = c.selection_bg,
        selection_fg = c.selection_fg,
        cursor = c.cursor,
        cursor_text = c.cursor_text,
    )
}
