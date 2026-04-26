//! Template Renderer Module
//!
//! Full-featured template rendering with filters.

pub mod apps;
pub mod apply;
pub mod renderer;

pub use apps::{get_template, list_templates, ThemeTemplate};
pub use apply::{apply_template, TemplateApplicator};
pub use renderer::{
    load_template, render_template, render_template_dual_mode, ColorFilters,
    DualModeTemplateRenderer, TemplateRenderer,
};