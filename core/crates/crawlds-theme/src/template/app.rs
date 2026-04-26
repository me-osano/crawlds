//! Template implementations for different applications.
//!
//! Provides template generators for terminals, editors, and other applications.

use crate::dynamic::scheme::SchemeColors;
use crate::template::{DualModeTemplateRenderer, TemplateRenderer};

pub trait ThemeTemplate: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, colors: &SchemeColors) -> String;
    fn render_dual_mode(&self, dark: &SchemeColors, light: &SchemeColors) -> String;
}

pub struct KittyTemplate;

impl ThemeTemplate for KittyTemplate {
    fn name(&self) -> &str {
        "kitty"
    }

    fn render(&self, colors: &SchemeColors) -> String {
        let renderer = TemplateRenderer::new(colors);
        renderer.render(KITTY_TEMPLATE)
    }

    fn render_dual_mode(&self, dark: &SchemeColors, light: &SchemeColors) -> String {
        let renderer = DualModeTemplateRenderer::from_colors(dark, light);
        renderer.render(KITTY_TEMPLATE)
    }
}

pub struct FootTemplate;

impl ThemeTemplate for FootTemplate {
    fn name(&self) -> &str {
        "foot"
    }

    fn render(&self, colors: &SchemeColors) -> String {
        let renderer = TemplateRenderer::new(colors);
        renderer.render(FOOT_TEMPLATE)
    }

    fn render_dual_mode(&self, dark: &SchemeColors, light: &SchemeColors) -> String {
        let renderer = DualModeTemplateRenderer::from_colors(dark, light);
        renderer.render(FOOT_TEMPLATE)
    }
}

pub struct AlacrittyTemplate;

impl ThemeTemplate for AlacrittyTemplate {
    fn name(&self) -> &str {
        "alacritty"
    }

    fn render(&self, colors: &SchemeColors) -> String {
        let renderer = TemplateRenderer::new(colors);
        renderer.render(ALACRITTY_TEMPLATE)
    }

    fn render_dual_mode(&self, dark: &SchemeColors, light: &SchemeColors) -> String {
        let renderer = DualModeTemplateRenderer::from_colors(dark, light);
        renderer.render(ALACRITTY_TEMPLATE)
    }
}

pub struct GtkTemplate;

impl ThemeTemplate for GtkTemplate {
    fn name(&self) -> &str {
        "gtk"
    }

    fn render(&self, colors: &SchemeColors) -> String {
        let renderer = TemplateRenderer::new(colors);
        renderer.render(GTK_TEMPLATE)
    }

    fn render_dual_mode(&self, dark: &SchemeColors, light: &SchemeColors) -> String {
        let renderer = DualModeTemplateRenderer::from_colors(dark, light);
        renderer.render(GTK_TEMPLATE)
    }
}

pub struct ZedTemplate;

impl ThemeTemplate for ZedTemplate {
    fn name(&self) -> &str {
        "zed"
    }

    fn render(&self, colors: &SchemeColors) -> String {
        let renderer = TemplateRenderer::new(colors);
        renderer.render(ZED_TEMPLATE)
    }

    fn render_dual_mode(&self, dark: &SchemeColors, light: &SchemeColors) -> String {
        let renderer = DualModeTemplateRenderer::from_colors(dark, light);
        renderer.render(ZED_TEMPLATE)
    }
}

pub struct CrawldsTemplate;

impl ThemeTemplate for CrawldsTemplate {
    fn name(&self) -> &str {
        "crawlds"
    }

    fn render(&self, colors: &SchemeColors) -> String {
        let renderer = TemplateRenderer::new(colors);
        renderer.render(CRAWLDS_TEMPLATE)
    }

    fn render_dual_mode(&self, dark: &SchemeColors, light: &SchemeColors) -> String {
        let renderer = DualModeTemplateRenderer::from_colors(dark, light);
        renderer.render(CRAWLDS_TEMPLATE)
    }
}

pub fn get_template(name: &str) -> Option<Box<dyn ThemeTemplate>> {
    match name {
        "kitty" => Some(Box::new(KittyTemplate)),
        "foot" => Some(Box::new(FootTemplate)),
        "alacritty" => Some(Box::new(AlacrittyTemplate)),
        "gtk" => Some(Box::new(GtkTemplate)),
        "zed" => Some(Box::new(ZedTemplate)),
        "crawlds" => Some(Box::new(CrawldsTemplate)),
        _ => None,
    }
}

pub fn list_templates() -> Vec<&'static str> {
    vec!["kitty", "foot", "alacritty", "gtk", "zed", "crawlds"]
}

const KITTY_TEMPLATE: &str = r#"# Theme: CrawlDS
foreground {{colors.on_surface.default.hex}}
background {{colors.surface.default.hex}}
selection_foreground {{colors.on_surface.default.hex}}
selection_background {{colors.primary_container.default.hex}}

color0 {{colors.surface_container_lowest.default.hex}}
color1 {{colors.error.default.hex}}
color2 {{colors.primary.default.hex}}
color3 {{colors.tertiary.default.hex}}
color4 {{colors.secondary.default.hex}}
color5 {{colors.secondary.default.hex | darken}}
color6 {{colors.tertiary.default.hex | lighten}}
color7 {{colors.surface_container_high.default.hex}}
color8 {{colors.surface_container.default.hex}}
color9 {{colors.error.default.hex | lighten}}
color10 {{colors.primary.default.hex | lighten}}
color11 {{colors.tertiary.default.hex | lighten}}
color12 {{colors.secondary.default.hex | lighten}}
color13 {{colors.secondary.default.hex | lighten}}
color14 {{colors.tertiary.default.hex | lighten}}
color15 {{colors.on_surface.default.hex}}

cursor {{colors.primary.default.hex}}
cursor_blink yes
font {{font_family}} {{font_size}}
"#;

const FOOT_TEMPLATE: &str = r#"[colors]
background = {{colors.surface.default.hex}}
foreground = {{colors.on_surface.default.hex}}
searchbox_no_match_fg = {{colors.error.default.hex}}
searchbox_match_fg = {{colors.surface.default.hex}}
selected_bg = {{colors.secondary_container.default.hex}}
selected_fg = {{colors.on_secondary.default.hex}}
scrollbar_inactive = {{colors.surface_variant.default.hex}}
scrollbar_active = {{colors.on_surface_variant.default.hex}}
transient_bg = {{colors.surface_container_high.default.hex}}

# Basic 16 colors
black = {{colors.surface_container_lowest.default.hex}}
red = {{colors.error.default.hex}}
green = {{colors.primary.default.hex}}
yellow = {{colors.tertiary.default.hex}}
blue = {{colors.secondary.default.hex}}
magenta = {{colors.tertiary.default.hex | darken}}
cyan = {{colors.secondary.default.hex | darken}}
white = {{colors.surface_container_high.default.hex}}

bright.black = {{colors.surface_container.default.hex}}
bright.red = {{colors.error.default.hex | lighten}}
bright.green = {{colors.primary.default.hex | lighten}}
bright.yellow = {{colors.tertiary.default.hex | lighten}}
bright.blue = {{colors.secondary.default.hex | lighten}}
bright.magenta = {{colors.tertiary.default.hex | lighten}}
bright.cyan = {{colors.secondary.default.hex | lighten}}
bright.white = {{colors.on_surface.default.hex}}

[csd]
border_width = 0
child_border_width = 1
padding = 2

[cursor]
style = beam
"#;

const ALACRITTY_TEMPLATE: &str = r#"# Theme: CrawlDS
colors:
  primary:
    background: '{{colors.surface.default.hex}}'
    foreground: '{{colors.on_surface.default.hex}}'
  cursor:
    text: '{{colors.primary.default.hex}}'
    cursor: '{{colors.primary.default.hex}}'
  search:
    matches:
      foreground: '{{colors.on_surface.default.hex}}'
      background: '{{colors.secondary_container.default.hex}}'
    focused_match:
      foreground: '{{colors.on_primary.default.hex}}'
      background: '{{colors.primary.default.hex}}'
  line_indicator:
    foreground: '{{colors.primary.default.hex}}'
    background: '{{colors.primary_container.default.hex}}'
  selection:
    text: '{{colors.on_surface.default.hex}}'
    background: '{{colors.secondary_container.default.hex}}'
  normal:
    black: '{{colors.surface_container_lowest.default.hex}}'
    red: '{{colors.error.default.hex}}'
    green: '{{colors.primary.default.hex}}'
    yellow: '{{colors.tertiary.default.hex}}'
    blue: '{{colors.secondary.default.hex}}'
    magenta: '{{colors.tertiary.default.hex | darken}}'
    cyan: '{{colors.secondary.default.hex | darken}}'
    white: '{{colors.surface_container_high.default.hex}}'
  bright:
    black: '{{colors.surface_container.default.hex}}'
    red: '{{colors.error.default.hex | lighten}}'
    green: '{{colors.primary.default.hex | lighten}}'
    yellow: '{{colors.tertiary.default.hex | lighten}}'
    blue: '{{colors.secondary.default.hex | lighten}}'
    magenta: '{{colors.tertiary.default.hex | lighten}}'
    cyan: '{{colors.secondary.default.hex | lighten}}'
    white: '{{colors.on_surface.default.hex}}'
  dim:
    black: '{{colors.surface.default.hex}}'
"#;

const GTK_TEMPLATE: &str = r#"@define-color theme_fg_color {{colors.on_surface.default.hex}};
@define-color theme_bg_color {{colors.surface.default.hex}};
@define-color theme_selected_bg_color {{colors.secondary_container.default.hex}};
@define-color theme_selected_fg_color {{colors.on_secondary.default.hex}};
@define-color theme_primary_color {{colors.primary.default.hex}};
@define-color theme_secondary_color {{colors.secondary.default.hex}};
@define-color theme_tertiary_color {{colors.tertiary.default.hex}};
@define-color theme_tooltip_bg_color {{colors.surface_container_high.default.hex}};
@define-color theme_tooltip_fg_color {{colors.on_surface.default.hex}};
@define-color theme_error_color {{colors.error.default.hex}};
"#;

const ZED_TEMPLATE: &str = r#"{
  "theme": {
    "name": "CrawlDS",
    "dark": {
      "background": "{{colors.surface.dark.hex}}",
      "foreground": "{{colors.on_surface.dark.hex}}",
      "accent": "{{colors.primary.dark.hex}}",
      "selection": "{{colors.secondary_container.dark.hex}}",
      "caret": "{{colors.primary.dark.hex}}",
      "link": "{{colors.primary.dark.hex}}",
      "syntax": {
        "keyword": "{{colors.tertiary.dark.hex}}",
        "string": "{{colors.primary.dark.hex}}",
        "number": "{{colors.secondary.dark.hex}}",
        "comment": "{{colors.on_surface_variant.dark.hex}}",
        "type": "{{colors.tertiary.dark.hex}}",
        "function": "{{colors.primary.dark.hex}}",
        "variable": "{{colors.on_surface.dark.hex}}"
      }
    }
  }
}
"#;

const CRAWLDS_TEMPLATE: &str = r#"{
  "name": "CrawlDS",
  "colors": {
    "primary": "{{colors.primary.default.hex}}",
    "onPrimary": "{{colors.on_primary.default.hex}}",
    "secondary": "{{colors.secondary.default.hex}}",
    "onSecondary": "{{colors.on_secondary.default.hex}}",
    "tertiary": "{{colors.tertiary.default.hex}}",
    "onTertiary": "{{colors.on_tertiary.default.hex}}",
    "error": "{{colors.error.default.hex}}",
    "onError": "{{colors.on_error.default.hex}}",
    "surface": "{{colors.surface.default.hex}}",
    "onSurface": "{{colors.on_surface.default.hex}}",
    "surfaceVariant": "{{colors.surface_variant.default.hex}}",
    "onSurfaceVariant": "{{colors.on_surface_variant.default.hex}}",
    "outline": "{{colors.outline.default.hex}}",
    "hover": "{{colors.hover.default.hex}}"
  }
}
# vim:ft=json
"#;
