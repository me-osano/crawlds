pub mod generator;
pub mod hct;
pub mod quantizer;
pub mod scheme;
pub mod terminal;

pub use generator::{GeneratorConfig, ThemeGenerator};
pub use hct::{Hct, TonalPalette};
pub use quantizer::{extract_source_color, quantize_wsmeans, quantize_wu, score_colors};
pub use scheme::{SchemeContent, SchemeMonochrome, SchemeRainbow, SchemeTonalSpot};
