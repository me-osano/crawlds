//! CIELAB color space conversion utilities
//!
//! Provides RGB ↔ XYZ ↔ LAB conversions with D65 illuminant.

const REF_X: f32 = 95.047;
const REF_Y: f32 = 100.000;
const REF_Z: f32 = 108.883;

const LAB_E: f32 = 0.008856;
const LAB_K: f32 = 903.3;

#[inline]
fn pivot_rgb(n: f32) -> f32 {
    if n > 0.04045 {
        ((n + 0.055) / 1.055).powf(2.4) * 100.0
    } else {
        n * 100.0 / 12.92
    }
}

#[inline]
fn pivot_xyz(n: f32) -> f32 {
    if n > LAB_E {
        n.cbrt() * 100.0
    } else {
        (LAB_K * n) / 100.0
    }
}

#[inline]
fn delinearize(n: f32) -> f32 {
    if n >= 0.0031308 {
        1.055 * n.powf(1.0 / 2.4) - 0.055
    } else {
        n * 12.92
    }
}

/// Convert sRGB (0-255) to LAB (L*, a*, b*)
pub fn rgb_to_lab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let r = pivot_rgb(r);
    let g = pivot_rgb(g);
    let b = pivot_rgb(b);

    let x = r * 0.4124 + g * 0.3576 + b * 0.1805;
    let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
    let z = r * 0.0193 + g * 0.1192 + b * 0.9505;

    let x = x / REF_X;
    let y = y / REF_Y;
    let z = z / REF_Z;

    let x = pivot_xyz(x);
    let y = pivot_xyz(y);
    let z = pivot_xyz(z);

    let l = 116.0 * y - 16.0;
    let a = 500.0 * (x - y);
    let b = 200.0 * (y - z);

    (l, a, b)
}

/// Convert LAB (L*, a*, b*) to sRGB (0-255)
pub fn lab_to_rgb(l: f32, a: f32, b: f32) -> (u8, u8, u8) {
    let mut y = (l + 16.0) / 116.0;
    let mut x = a / 500.0 + y;
    let mut z = y - b / 200.0;

    let y2 = y * y * y;
    let x2 = x * x * x;
    let z2 = z * z * z;

    y = if y2 > LAB_E {
        y2
    } else {
        (116.0 * y - 16.0) / LAB_K
    };
    x = if x2 > LAB_E {
        x2
    } else {
        (116.0 * x - 16.0) / LAB_K
    };
    z = if z2 > LAB_E {
        z2
    } else {
        (116.0 * z - 16.0) / LAB_K
    };

    x *= REF_X;
    y *= REF_Y;
    z *= REF_Z;

    let r = x * 3.2406 + y * -1.5372 + z * -0.4986;
    let g = x * -0.9689 + y * 1.8758 + z * 0.0415;
    let bb = x * 0.0557 + y * -0.2040 + z * 1.0570;

    let r = delinearize(r / 100.0).clamp(0.0, 1.0);
    let g = delinearize(g / 100.0).clamp(0.0, 1.0);
    let bb = delinearize(bb / 100.0).clamp(0.0, 1.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (bb * 255.0).round() as u8,
    )
}

/// Calculate squared Euclidean distance between two LAB colors
#[inline]
pub fn lab_distance((l1, a1, b1): (f32, f32, f32), (l2, a2, b2): (f32, f32, f32)) -> f32 {
    let dl = l1 - l2;
    let da = a1 - a2;
    let db = b1 - b2;
    dl * dl + da * da + db * db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_lab_roundtrip() {
        let (r, g, b) = (66, 135, 255);
        let lab = rgb_to_lab(r, g, b);
        let (r2, g2, b2) = lab_to_rgb(lab.0, lab.1, lab.2);

        assert!(r.abs_diff(r2) <= 1);
        assert!(g.abs_diff(g2) <= 1);
        assert!(b.abs_diff(b2) <= 1);
    }
}
