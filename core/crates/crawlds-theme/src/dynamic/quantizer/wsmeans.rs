//! WSMeans (Weighted K-Means) quantizer
//!
//! Refines Wu quantizer output using weighted k-means in Lab color space.

use std::collections::HashMap;

use super::wu::argb_from_rgb;
use crate::dynamic::hct::lab::{lab_to_rgb, rgb_to_lab};

const LCG_MASK: u64 = (1 << 48) - 1;

fn lab_distance_squared(a: (f32, f32, f32), b: (f32, f32, f32)) -> f32 {
    let dl = a.0 - b.0;
    let da = a.1 - b.1;
    let db = a.2 - b.2;
    dl * dl + da * da + db * db
}

fn rgb_from_argb(argb: u32) -> (u8, u8, u8) {
    (
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
    )
}

pub fn quantize_wsmeans(
    pixels: &[(u8, u8, u8)],
    max_colors: usize,
    starting_clusters: &[u32],
) -> HashMap<u32, usize> {
    let mut pixel_to_count: HashMap<u32, usize> = HashMap::new();
    let mut unique_pixels: Vec<u32> = Vec::new();
    let mut points: Vec<(f32, f32, f32)> = Vec::new();

    for (r, g, b) in pixels {
        let argb = argb_from_rgb(*r, *g, *b);
        if let Some(count) = pixel_to_count.get_mut(&argb) {
            *count += 1;
        } else {
            unique_pixels.push(argb);
            points.push(rgb_to_lab(*r, *g, *b));
            pixel_to_count.insert(argb, 1);
        }
    }

    let cluster_count = max_colors.min(points.len());
    if cluster_count == 0 {
        return HashMap::new();
    }

    let mut clusters: Vec<(f32, f32, f32)> = starting_clusters
        .iter()
        .take(cluster_count)
        .map(|&argb| {
            let (r, g, b) = rgb_from_argb(argb);
            rgb_to_lab(r, g, b)
        })
        .collect();

    let additional_needed = cluster_count - clusters.len();
    if additional_needed > 0 {
        let mut rng = LcgRandom::new(0x42688);
        let mut indices: Vec<usize> = Vec::new();
        for _ in 0..additional_needed {
            let mut index = rng.next_range(points.len() as u32) as usize;
            while indices.contains(&index) {
                index = rng.next_range(points.len() as u32) as usize;
            }
            indices.push(index);
        }
        for index in indices {
            clusters.push(points[index]);
        }
    }

    let mut cluster_indices: Vec<usize> = (0..points.len()).map(|i| i % cluster_count).collect();
    let mut pixel_count_sums = vec![0usize; cluster_count];

    for _iteration in 0..10 {
        let mut points_moved = 0;

        for i in 0..cluster_count {
            for j in (i + 1)..cluster_count {
                let _dist = lab_distance_squared(clusters[i], clusters[j]);
            }
        }

        for i in 0..points.len() {
            let point = points[i];
            let prev_idx = cluster_indices[i];
            let prev_dist = lab_distance_squared(point, clusters[prev_idx]);

            let mut min_dist = prev_dist;
            let mut new_idx = prev_idx;

            for j in 0..cluster_count {
                let dist = lab_distance_squared(point, clusters[j]);
                if dist < min_dist {
                    min_dist = dist;
                    new_idx = j;
                }
            }

            if new_idx != prev_idx {
                points_moved += 1;
                cluster_indices[i] = new_idx;
            }
        }

        if points_moved == 0 {
            break;
        }

        let mut component_l = vec![0.0f32; cluster_count];
        let mut component_a = vec![0.0f32; cluster_count];
        let mut component_b = vec![0.0f32; cluster_count];

        for i in 0..points.len() {
            let cidx = cluster_indices[i];
            let pt = points[i];
            let count = *pixel_to_count.get(&unique_pixels[i]).unwrap_or(&1);
            pixel_count_sums[cidx] += count;
            component_l[cidx] += pt.0 * count as f32;
            component_a[cidx] += pt.1 * count as f32;
            component_b[cidx] += pt.2 * count as f32;
        }

        for i in 0..cluster_count {
            let count = pixel_count_sums[i];
            if count == 0 {
                clusters[i] = (0.0, 0.0, 0.0);
            } else {
                clusters[i] = (
                    component_l[i] / count as f32,
                    component_a[i] / count as f32,
                    component_b[i] / count as f32,
                );
            }
        }
    }

    let mut cluster_populations: HashMap<u32, usize> = HashMap::new();
    for i in 0..cluster_count {
        let count = pixel_count_sums[i];
        if count == 0 {
            continue;
        }

        let (lab_l, lab_a, lab_b) = clusters[i];
        let (r, g, b) = lab_to_rgb(lab_l, lab_a, lab_b);
        let argb = argb_from_rgb(r, g, b);

        if cluster_populations.contains_key(&argb) {
            continue;
        }

        cluster_populations.insert(argb, count);
    }

    cluster_populations
}

struct LcgRandom {
    seed: u64,
}

impl LcgRandom {
    fn new(seed: u32) -> Self {
        Self {
            seed: ((seed as u64 ^ 0x5DEECE66Du64) & LCG_MASK),
        }
    }

    fn next(&mut self, bits: u32) -> u32 {
        self.seed = (self.seed.wrapping_mul(0x5DEECE66Du64).wrapping_add(0xBu64)) & LCG_MASK;
        (self.seed >> (48 - bits)) as u32
    }

    fn next_range(&mut self, range: u32) -> u32 {
        if range & (range - 1) == 0 {
            return ((range as u64 * self.next(31) as u64) >> 31) as u32;
        }
        loop {
            let bits = self.next(31);
            let val = bits % range;
if bits >= val {
                return val;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsmeans() {
        let pixels = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        let starting = vec![argb_from_rgb(255, 0, 0)];
        let result = quantize_wsmeans(&pixels, 3, &starting);
        assert!(!result.is_empty());
    }
}
