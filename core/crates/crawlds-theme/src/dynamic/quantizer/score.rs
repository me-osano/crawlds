//! Color scoring algorithm for UI theme suitability
//!
//! Ranks colors based on chroma and proportion for Material Design themes.

use std::collections::HashMap;

use crate::dynamic::hct::Hct;

const TARGET_CHROMA: f32 = 48.0;
const WEIGHT_PROPORTION: f32 = 0.7;
const WEIGHT_CHROMA_ABOVE: f32 = 0.3;
const WEIGHT_CHROMA_BELOW: f32 = 0.1;
const CUTOFF_CHROMA: f32 = 5.0;
const CUTOFF_EXCITED_PROPORTION: f32 = 0.01;
#[allow(dead_code)]
const FALLBACK_COLOR_ARGB: u32 = 0xFF4285F4;

#[inline]
fn sanitize_degrees(degrees: f32) -> i32 {
    let d = degrees % 360.0;
    if d < 0.0 {
        (d + 360.0) as i32
    } else {
        d as i32
    }
}

#[inline]
fn difference_degrees(a: f32, b: f32) -> f32 {
    let diff = (a - b).abs();
    diff.min(360.0 - diff)
}

pub fn score_colors(
    color_to_population: &HashMap<u32, usize>,
    desired: usize,
    fallback_color: u32,
    filter_colors: bool,
) -> Vec<u32> {
    let mut colors_hct: Vec<(u32, Hct)> = Vec::new();
    let mut hue_population = vec![0usize; 360];
    let mut population_sum = 0usize;

    for (&argb, &population) in color_to_population.iter() {
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;

        let hct = Hct::from_rgb(r, g, b);
        colors_hct.push((argb, hct));
        let hue = sanitize_degrees(hct.get_hue()) as usize;
        hue_population[hue] += population;
        population_sum += population;
    }

    if colors_hct.is_empty() || population_sum == 0 {
        return vec![fallback_color];
    }

    let population_sum_f = population_sum as f32;
    let mut hue_excited_proportions = vec![0.0f32; 360];
    for hue in 0..360 {
        let proportion = hue_population[hue] as f32 / population_sum_f;
        for offset in -14..16 {
            let neighbor_hue = ((hue as i32 + offset).rem_euclid(360)) as usize;
            hue_excited_proportions[neighbor_hue] += proportion;
        }
    }

    let mut scored_hct: Vec<(u32, crate::dynamic::hct::Hct, f32)> = Vec::new();
    for (argb, hct) in &colors_hct {
        let hue = sanitize_degrees(hct.get_hue()) as usize;
        let proportion = hue_excited_proportions[hue];

        if filter_colors {
            if hct.get_chroma() < CUTOFF_CHROMA {
                continue;
            }
            if proportion <= CUTOFF_EXCITED_PROPORTION {
                continue;
            }
        }

        let proportion_score = proportion * 100.0 * WEIGHT_PROPORTION;

        let chroma_weight = if hct.get_chroma() < TARGET_CHROMA {
            WEIGHT_CHROMA_BELOW
        } else {
            WEIGHT_CHROMA_ABOVE
        };
        let chroma_score = (hct.get_chroma() - TARGET_CHROMA) * chroma_weight;

        let score = proportion_score + chroma_score;
        scored_hct.push((*argb, *hct, score));
    }

    if scored_hct.is_empty() {
        return vec![fallback_color];
    }

    scored_hct.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut chosen_colors: Vec<(u32, crate::dynamic::hct::Hct)> = Vec::new();

    for diff_degrees in (15..=90).rev() {
        chosen_colors.clear();
        for (argb, hct, _) in &scored_hct {
            let mut is_duplicate = false;
            for (_, chosen_hct) in &chosen_colors {
                if difference_degrees(hct.get_hue(), chosen_hct.get_hue()) < diff_degrees as f32 {
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                chosen_colors.push((*argb, *hct));
            }

            if chosen_colors.len() >= desired {
                break;
            }
        }

        if chosen_colors.len() >= desired {
            break;
        }
    }

    if chosen_colors.is_empty() {
        return vec![fallback_color];
    }

    chosen_colors.into_iter().map(|(argb, _)| argb).collect()
}

pub fn extract_source_color(pixels: &[(u8, u8, u8)], fallback_color: u32) -> u32 {
    if pixels.is_empty() {
        return fallback_color;
    }

    let wu_result = super::wu::quantize_wu(pixels, 128);
    let starting_clusters: Vec<u32> = wu_result;

    let color_to_count = quantize_wsmeans(pixels, 128, &starting_clusters);

    let mut filtered: HashMap<u32, usize> = HashMap::new();
    for (&argb, &count) in color_to_count.iter() {
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        let hct = Hct::from_rgb(r, g, b);
        if hct.get_chroma() >= 5.0 {
            filtered.insert(argb, count);
        }
    }

    if filtered.is_empty() {
        filtered = color_to_count;
    }

    let ranked = score_colors(&filtered, 4, fallback_color, true);

    ranked.into_iter().next().unwrap_or(fallback_color)
}

#[allow(dead_code)]
fn argb_from_rgb(r: u8, g: u8, b: u8) -> u32 {
    (255u32 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[allow(dead_code)]
fn rgb_from_argb(argb: u32) -> (u8, u8, u8) {
    (
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
    )
}

use super::wsmeans::quantize_wsmeans;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_colors() {
        let mut colors = HashMap::new();
        colors.insert(argb_from_rgb(255, 0, 0), 100);
        colors.insert(argb_from_rgb(0, 255, 0), 80);

        let result = score_colors(&colors, 2, argb_from_rgb(66, 135, 255), true);
        assert!(!result.is_empty());
    }
}
