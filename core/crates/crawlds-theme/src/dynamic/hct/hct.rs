//! HCT (Hue-Chroma-Tone) color space implementation
//!
//! HCT combines hue, chroma, and tone into a perceptually uniform color space
//! that maps directly to Material Design 3 color system.

use std::f32::consts::PI;

use super::lab::{lab_to_rgb, rgb_to_lab};

const MIN_CHROMA_THRESHOLD: f32 = 0.1;

#[derive(Clone, Copy, Debug)]
pub struct Hct {
    hue: f32,
    chroma: f32,
    tone: f32,
    argb: u32,
}

impl Hct {
    pub fn from(hue: f32, chroma: f32, tone: f32) -> Self {
        let argb = Self::get_argb(hue, chroma, tone);
        Self {
            hue,
            chroma,
            tone,
            argb,
        }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let (l, a, bb) = rgb_to_lab(r, g, b);
        let hue = Self::sanitize_degrees((a.atan2(bb) * 180.0 / PI).rem_euclid(360.0));
        let chroma = (a.powi(2) + bb.powi(2)).sqrt();
        let argb = Self::rgb_to_argb(r, g, b);
        Self {
            hue,
            chroma,
            tone: l,
            argb,
        }
    }

    pub fn from_hue_and_tone(hue: f32, tone: f32) -> Self {
        Self::from(hue, 0.0, tone)
    }

    #[inline]
    fn sanitize_degrees(degrees: f32) -> f32 {
        let d = degrees % 360.0;
        if d < 0.0 {
            d + 360.0
        } else {
            d
        }
    }

    #[inline]
    #[allow(dead_code)]
    fn angle_diff(a: f32, b: f32) -> f32 {
        let diff = (a - b + 180.0).rem_euclid(360.0) - 180.0;
        if diff < -180.0 {
            diff + 360.0
        } else {
            diff
        }
    }

    #[inline]
    #[allow(dead_code)]
    fn set_alpha(argb: u32, alpha: u8) -> u32 {
        (argb & 0x00FFFFFF) | ((alpha as u32) << 24)
    }

    #[inline]
    fn rgb_to_argb(r: u8, g: u8, b: u8) -> u32 {
        0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    fn from_argb(argb: u32) -> (u8, u8, u8, u8) {
        (
            ((argb >> 16) & 0xFF) as u8,
            ((argb >> 8) & 0xFF) as u8,
            (argb & 0xFF) as u8,
            ((argb >> 24) & 0xFF) as u8,
        )
    }

    fn argb_from_rgb(r: u8, g: u8, b: u8) -> u32 {
        (255u32 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    #[inline]
    #[allow(dead_code)]
    fn is_bounded(x: f32, l: f32, u: f32) -> bool {
        x >= l - 0.002 && x <= u + 0.002
    }

    fn hue_to_radian(hue: f32) -> f32 {
        hue * PI / 180.0
    }

    fn get_argb(hue: f32, chroma: f32, tone: f32) -> u32 {
        if chroma < MIN_CHROMA_THRESHOLD {
            return Self::get_argb_for_tone(tone);
        }

        let hue_r = Self::hue_to_radian(hue);
        let cs = hue_r.cos();
        let ss = hue_r.sin();

        let n = (tone + 10.0) / 100.0;
        let nn = n * n;
        let nnn = nn * n;

        let _a = 50.0 * ((83.084 - 57.129 * n + 9.0 * nn - 0.1268 * nnn).sqrt() - 5.873) * cs;
        let _b = 50.0 * ((83.084 - 57.129 * n + 9.0 * nn - 0.1268 * nnn).sqrt() - 5.873) * ss;

        let mut low = 0.0_f32;
        let mut high = chroma;

        for _ in 0..80 {
            let mid = (low + high) / 2.0;
            let argb = Self::from_linear_solve(mid, mid, hue, tone, cs, ss);
            if argb == 0 {
                high = mid;
                continue;
            }

            let (r, g, b, _) = Self::from_argb(argb);
            let (_, ca, cb) = rgb_to_lab(r, g, b);
            let actual_chroma = (ca * ca + cb * cb).sqrt();
            let error = actual_chroma - mid;

            if error.abs() < 0.01 {
                return argb;
            }

            if actual_chroma > mid {
                high = mid;
            } else {
                low = mid;
            }
        }

        Self::from_linear_solve((low + high) / 2.0, (low + high) / 2.0, hue, tone, cs, ss)
    }

    fn from_linear_solve(
        chroma: f32,
        expected_chroma: f32,
        hue: f32,
        tone: f32,
        cs: f32,
        ss: f32,
    ) -> u32 {
        let _hue_r = Self::hue_to_radian(hue);
        let n = (tone + 10.0) / 100.0;
        let nn = n * n;
        let nnn = nn * n;

        let target_l =
            (50.0 - 5.873 * (83.084 - 57.129 * n + 9.0 * nn - 0.1268 * nnn).sqrt()).max(0.0);

        let a = chroma * cs;
        let b = chroma * ss;

        let l_start = target_l.floor() as i32;
        let l_end = target_l.ceil() as i32;

        for l in l_start..=l_end {
            let ll = l as f32;
            let (r, g, b) = lab_to_rgb(ll, a, b);

            let (r_l, _g_l, bb_l) = rgb_to_lab(r, g, b);
            let chroma_at_l = (r_l * r_l + bb_l * bb_l).sqrt();

            if chroma_at_l >= expected_chroma - 0.4 {
                return Self::argb_from_rgb(r, g, b);
            }
        }

        0
    }

    fn get_argb_for_tone(tone: f32) -> u32 {
        let (r, g, b) = lab_to_rgb(tone, 0.0, 0.0);
        Self::argb_from_rgb(r, g, b)
    }

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        let (r, g, b, _) = Self::from_argb(self.argb);
        (r, g, b)
    }

    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn get_hue(&self) -> f32 {
        self.hue
    }

    pub fn get_chroma(&self) -> f32 {
        self.chroma
    }

    pub fn get_tone(&self) -> f32 {
        self.tone
    }

    pub fn to_argb(&self) -> u32 {
        self.argb
    }
}

impl Default for Hct {
    fn default() -> Self {
        Self::from(0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hct_from_rgb() {
        let hct = Hct::from_rgb(66, 135, 255);
        assert!(hct.get_hue() >= 0.0 && hct.get_hue() <= 360.0);
        assert!(hct.get_tone() >= 0.0 && hct.get_tone() <= 100.0);
    }

    #[test]
    fn test_hct_roundtrip() {
        let hct = Hct::from(220.0, 50.0, 50.0);
        let (r, g, b) = hct.to_rgb();
        let hct2 = Hct::from_rgb(r, g, b);

        let hue_diff = (hct.get_hue() - hct2.get_hue()).abs();
        assert!(hue_diff < 1.0 || hue_diff > 359.0);
    }

    #[test]
    fn test_hct_hex() {
        let hct = Hct::from(220.0, 50.0, 50.0);
        let hex = hct.to_hex();
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
    }
}
