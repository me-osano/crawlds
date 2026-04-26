use super::TerminalTheme;

pub fn generate(theme: &TerminalTheme) -> String {
    let c = &theme.colors;
    format!(
        "color0 {}\n\
         color1 {}\n\
         color2 {}\n\
         color3 {}\n\
         color4 {}\n\
         color5 {}\n\
         color6 {}\n\
         color7 {}\n\
         color8 {}\n\
         color9 {}\n\
         color10 {}\n\
         color11 {}\n\
         color12 {}\n\
         color13 {}\n\
         color14 {}\n\
         color15 {}\n\
         cursor {}\n\
         cursor_text_color {}\n\
         background {}\n\
         foreground {}\n\
         selection_foreground {}\n\
         selection_background {}",
        c.normal.black.strip_prefix('#').unwrap_or(&c.normal.black),
        c.normal.red.strip_prefix('#').unwrap_or(&c.normal.red),
        c.normal.green.strip_prefix('#').unwrap_or(&c.normal.green),
        c.normal
            .yellow
            .strip_prefix('#')
            .unwrap_or(&c.normal.yellow),
        c.normal.blue.strip_prefix('#').unwrap_or(&c.normal.blue),
        c.normal
            .magenta
            .strip_prefix('#')
            .unwrap_or(&c.normal.magenta),
        c.normal.cyan.strip_prefix('#').unwrap_or(&c.normal.cyan),
        c.normal.white.strip_prefix('#').unwrap_or(&c.normal.white),
        c.bright.black.strip_prefix('#').unwrap_or(&c.bright.black),
        c.bright.red.strip_prefix('#').unwrap_or(&c.bright.red),
        c.bright.green.strip_prefix('#').unwrap_or(&c.bright.green),
        c.bright
            .yellow
            .strip_prefix('#')
            .unwrap_or(&c.bright.yellow),
        c.bright.blue.strip_prefix('#').unwrap_or(&c.bright.blue),
        c.bright
            .magenta
            .strip_prefix('#')
            .unwrap_or(&c.bright.magenta),
        c.bright.cyan.strip_prefix('#').unwrap_or(&c.bright.cyan),
        c.bright.white.strip_prefix('#').unwrap_or(&c.bright.white),
        c.cursor.strip_prefix('#').unwrap_or(&c.cursor),
        c.cursor_text.strip_prefix('#').unwrap_or(&c.cursor_text),
        c.background.strip_prefix('#').unwrap_or(&c.background),
        c.foreground.strip_prefix('#').unwrap_or(&c.foreground),
        c.selection_fg.strip_prefix('#').unwrap_or(&c.selection_fg),
        c.selection_bg.strip_prefix('#').unwrap_or(&c.selection_bg),
    )
}
