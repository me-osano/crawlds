//! CAM16 color appearance model (simplified)
//!
//! A simplified CAM16 implementation focused on providing chroma information
//! for theme generation.

use crate::dynamic::hct::Hct;

#[derive(Clone, Copy, Debug)]
pub struct Cam16 {
    pub hue: f32,
    pub chroma: f32,
}

impl Cam16 {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Result<Self, &'static str> {
        let hct = Hct::from_rgb(r, g, b);
        Ok(Self {
            hue: hct.get_hue(),
            chroma: hct.get_chroma(),
        })
    }
}
