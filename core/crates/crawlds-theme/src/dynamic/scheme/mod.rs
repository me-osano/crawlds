//! Material Design 3 Scheme Module
//!
//! Provides M3 color scheme implementations.

pub mod content;
pub mod faithful;
pub mod fruit_salad;
pub mod monochrome;
pub mod muted;
pub mod rainbow;
pub mod tonal_spot;
pub mod tones;
pub mod vibrant;

pub use content::SchemeContent;
pub use faithful::SchemeFaithful;
pub use fruit_salad::SchemeFruitSalad;
pub use monochrome::SchemeMonochrome;
pub use muted::SchemeMuted;
pub use rainbow::SchemeRainbow;
pub use tonal_spot::{Scheme, SchemeColors, SchemeTonalSpot};
pub use tones::{
    ToneValues, DARK_TONES, LIGHT_TONES, MONOCHROME_DARK_TONES, MONOCHROME_LIGHT_TONES,
    MUTED_DARK_TONES, MUTED_LIGHT_TONES,
};
pub use vibrant::SchemeVibrant;
