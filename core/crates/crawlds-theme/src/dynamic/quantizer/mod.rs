//! Color Quantizer Module
//!
//! Provides Wu and WSMeans color quantization algorithms for extracting
//! color palettes from images.

pub mod score;
pub mod wsmeans;
pub mod wu;

pub use score::{extract_source_color, score_colors};
pub use wsmeans::quantize_wsmeans;
pub use wu::{quantize_wu, QuantizerWu};
