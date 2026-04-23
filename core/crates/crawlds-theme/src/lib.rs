mod dynamic;
mod error;
mod r#static;
mod types;

pub use dynamic::generator::{generate_terminal_colors, SCHEMA_VERSION};
pub use error::{ThemeError, ThemeResult};
pub use types::{
    TerminalColorSet, TerminalColors, ThemeColors, ThemeData, ThemeMetadata, ThemeSchemeType,
    ThemeVariant,
};

pub use dynamic::{DynamicThemeGenerator, Matugen};
pub use r#static::{load_theme_from_file, ThemeManager};

pub use r#static::loader::{RawTerminalColors, RawTheme, RawThemeColors, RawThemeVariant};
