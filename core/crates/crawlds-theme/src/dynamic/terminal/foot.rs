use super::TerminalTheme;

pub fn generate(theme: &TerminalTheme) -> String {
    let c = &theme.colors;
    format!(
        "[colors]\n\
         background = {}\n\
         foreground = {}\n\
         \n\
         regular0 = {}\n\
         regular1 = {}\n\
         regular2 = {}\n\
         regular3 = {}\n\
         regular4 = {}\n\
         regular5 = {}\n\
         regular6 = {}\n\
         regular7 = {}\n\
         \n\
         bright0 = {}\n\
         bright1 = {}\n\
         bright2 = {}\n\
         bright3 = {}\n\
         bright4 = {}\n\
         bright5 = {}\n\
         bright6 = {}\n\
         bright7 = {}\n\
         \n\
         selection-foreground = {}\n\
         selection-background = {}\n\
         \n\
         cursor = {}\n\
         cursor-foreground = {}",
        c.background.strip_prefix('#').unwrap_or(&c.background),
        c.foreground.strip_prefix('#').unwrap_or(&c.foreground),
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
        c.selection_fg.strip_prefix('#').unwrap_or(&c.selection_fg),
        c.selection_bg.strip_prefix('#').unwrap_or(&c.selection_bg),
        c.cursor.strip_prefix('#').unwrap_or(&c.cursor),
        c.cursor_text.strip_prefix('#').unwrap_or(&c.cursor_text),
    )
}
