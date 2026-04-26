//! TonalPalette for Material Design 3 color system
//!
//! A TonalPalette is a color palette with a fixed hue and varying chroma and tone.

use std::cell::RefCell;
use std::collections::HashMap;

use super::hct::Hct;

const CACHE_SIZE: usize = 200;

#[derive(Clone)]
pub struct TonalPalette {
    hue: f32,
    chroma: f32,
    cache: RefCell<HashMap<i32, u32>>,
}

impl TonalPalette {
    pub fn from_hct(hct: &Hct) -> Self {
        Self {
            hue: hct.get_hue(),
            chroma: hct.get_chroma(),
            cache: RefCell::new(HashMap::with_capacity(CACHE_SIZE)),
        }
    }

    pub fn new(hue: f32, chroma: f32) -> Self {
        Self {
            hue,
            chroma,
            cache: RefCell::new(HashMap::with_capacity(CACHE_SIZE)),
        }
    }

    pub fn tone(&self, tone: f32) -> u32 {
        let key = (tone * 100.0).round() as i32;
        let mut cache = self.cache.borrow_mut();
        if let Some(&argb) = cache.get(&key) {
            return argb;
        }
        let hct = Hct::from(self.hue, self.chroma, tone);
        let argb = hct.to_argb();
        if cache.len() < CACHE_SIZE {
            cache.insert(key, argb);
        }
        argb
    }

    pub fn tone_hex(&self, tone: f32) -> String {
        let argb = self.tone(tone);
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn get_hue(&self) -> f32 {
        self.hue
    }

    pub fn get_chroma(&self) -> f32 {
        self.chroma
    }
}

impl Default for TonalPalette {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl std::fmt::Debug for TonalPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TonalPalette")
            .field("hue", &self.hue)
            .field("chroma", &self.chroma)
            .field("cache_size", &self.cache.borrow().len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tonal_palette() {
        let palette = TonalPalette::new(220.0, 48.0);

        assert_eq!(palette.tone(40.0), palette.tone(40.0));
        assert_eq!(palette.get_hue(), 220.0);
        assert_eq!(palette.get_chroma(), 48.0);
    }

    #[test]
    fn test_tone_hex() {
        let palette = TonalPalette::new(220.0, 48.0);
        let hex = palette.tone_hex(50.0);
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
    }
}
