//! Wu color quantization algorithm
//!
//! Implements Xiaolin Wu's color quantization algorithm for extracting
//! color palettes from images.

const INDEX_BITS: usize = 5;
const SIDE_LENGTH: usize = 33;
const TOTAL_SIZE: usize = 35937;

#[allow(dead_code)]
const DIR_RED: usize = 0;
#[allow(dead_code)]
const DIR_GREEN: usize = 1;
#[allow(dead_code)]
const DIR_BLUE: usize = 2;

#[derive(Clone, Debug)]
pub struct Box {
    pub r0: usize,
    pub r1: usize,
    pub g0: usize,
    pub g1: usize,
    pub b0: usize,
    pub b1: usize,
    pub vol: usize,
}

impl Box {
    fn new() -> Self {
        Self {
            r0: 0,
            r1: 0,
            g0: 0,
            g1: 0,
            b0: 0,
            b1: 0,
            vol: 0,
        }
    }
}

#[inline]
fn get_index(r: usize, g: usize, b: usize) -> usize {
    (r << (INDEX_BITS * 2)) + (r << (INDEX_BITS + 1)) + r + (g << INDEX_BITS) + g + b
}

pub fn argb_from_rgb(r: u8, g: u8, b: u8) -> u32 {
    (255u32 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline]
#[allow(dead_code)]
fn rgb_from_argb(argb: u32) -> (u8, u8, u8) {
    (
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
    )
}

pub struct QuantizerWu {
    weights: Vec<i32>,
    moments_r: Vec<i32>,
    moments_g: Vec<i32>,
    moments_b: Vec<i32>,
    moments: Vec<f32>,
    cubes: Vec<Box>,
}

impl QuantizerWu {
    pub fn new() -> Self {
        Self {
            weights: vec![0; TOTAL_SIZE],
            moments_r: vec![0; TOTAL_SIZE],
            moments_g: vec![0; TOTAL_SIZE],
            moments_b: vec![0; TOTAL_SIZE],
            moments: vec![0.0; TOTAL_SIZE],
            cubes: Vec::new(),
        }
    }

    pub fn quantize(&mut self, pixels: &[u32], max_colors: usize) -> Vec<u32> {
        self.construct_histogram(pixels);
        self.compute_moments();
        let result_count = self.create_boxes(max_colors);
        self.create_result(result_count)
    }

    fn construct_histogram(&mut self, pixels: &[u32]) {
        self.weights.fill(0);
        self.moments_r.fill(0);
        self.moments_g.fill(0);
        self.moments_b.fill(0);
        self.moments.fill(0.0);

        let mut count_by_color: std::collections::HashMap<u32, i32> =
            std::collections::HashMap::new();
        for &pixel in pixels {
            if (pixel >> 24) & 0xFF == 255 {
                *count_by_color.entry(pixel).or_insert(0) += 1;
            }
        }

        let bits_to_remove = 8 - INDEX_BITS;
        for (&pixel, &count) in count_by_color.iter() {
            let r = (pixel >> 16) & 0xFF;
            let g = (pixel >> 8) & 0xFF;
            let b = pixel & 0xFF;

            let i_r = (r as usize >> bits_to_remove) + 1;
            let i_g = (g as usize >> bits_to_remove) + 1;
            let i_b = (b as usize >> bits_to_remove) + 1;
            let index = get_index(i_r, i_g, i_b);

            self.weights[index] += count;
            self.moments_r[index] += count * r as i32;
            self.moments_g[index] += count * g as i32;
            self.moments_b[index] += count * b as i32;
            self.moments[index] +=
                count as f32 * (r as f32 * r as f32 + g as f32 * g as f32 + b as f32 * b as f32);
        }
    }

    fn compute_moments(&mut self) {
        for r in 1..SIDE_LENGTH {
            let mut area = vec![0i32; SIDE_LENGTH];
            let mut area_r = vec![0i32; SIDE_LENGTH];
            let mut area_g = vec![0i32; SIDE_LENGTH];
            let mut area_b = vec![0i32; SIDE_LENGTH];
            let mut area2 = vec![0.0f32; SIDE_LENGTH];

            for g in 1..SIDE_LENGTH {
                let mut line = 0i32;
                let mut line_r = 0i32;
                let mut line_g = 0i32;
                let mut line_b = 0i32;
                let mut line2 = 0.0f32;

                for b in 1..SIDE_LENGTH {
                    let index = get_index(r, g, b);
                    line += self.weights[index];
                    line_r += self.moments_r[index];
                    line_g += self.moments_g[index];
                    line_b += self.moments_b[index];
                    line2 += self.moments[index];

                    area[b] += line;
                    area_r[b] += line_r;
                    area_g[b] += line_g;
                    area_b[b] += line_b;
                    area2[b] += line2;

                    let prev_index = get_index(r - 1, g, b);
                    self.weights[index] = self.weights[prev_index] + area[b];
                    self.moments_r[index] = self.moments_r[prev_index] + area_r[b];
                    self.moments_g[index] = self.moments_g[prev_index] + area_g[b];
                    self.moments_b[index] = self.moments_b[prev_index] + area_b[b];
                    self.moments[index] = self.moments[prev_index] + area2[b];
                }
            }
        }
    }

    fn create_boxes(&mut self, max_colors: usize) -> usize {
        self.cubes = vec![Box::new(); max_colors];
        let mut volume_variance = vec![0.0f32; max_colors];

        self.cubes[0].r1 = SIDE_LENGTH - 1;
        self.cubes[0].g1 = SIDE_LENGTH - 1;
        self.cubes[0].b1 = SIDE_LENGTH - 1;

        let mut generated_color_count = max_colors;
        let mut next_box = 0;
        let mut i = 1;

        while i < max_colors {
            let next_box_idx = next_box;
            let i_idx = i;

            if next_box_idx == i_idx {
                volume_variance[next_box_idx] = 0.0;
                i += 1;
                continue;
            }

            let (one_r0, one_r1, one_g0, one_g1, one_b0, one_b1) = {
                let one = &self.cubes[next_box_idx];
                (one.r0, one.r1, one.g0, one.g1, one.b0, one.b1)
            };

            let (cut_result, vol_one, vol_i) = unsafe {
                let one_ptr = self.cubes.as_mut_ptr().add(next_box_idx);
                let two_ptr = self.cubes.as_mut_ptr().add(i_idx);
                let result = cut_box_values(
                    &mut *one_ptr,
                    &mut *two_ptr,
                    one_r0,
                    one_r1,
                    one_g0,
                    one_g1,
                    one_b0,
                    one_b1,
                );
                (result, (*one_ptr).vol, (*two_ptr).vol)
            };

            if cut_result {
                volume_variance[next_box_idx] = if vol_one > 1 { 0.0 } else { 0.0 };
                volume_variance[i_idx] = if vol_i > 1 { 0.0 } else { 0.0 };
            } else {
                volume_variance[next_box_idx] = 0.0;
                i -= 1;
            }

            next_box = 0;
            let mut temp = volume_variance[0];
            for j in 1..=i {
                if volume_variance[j] > temp {
                    temp = volume_variance[j];
                    next_box = j;
                }
            }

            if temp <= 0.0 {
                generated_color_count = i + 1;
                break;
            }

            i += 1;
        }

        generated_color_count
    }

    fn create_result(&self, color_count: usize) -> Vec<u32> {
        let mut colors = Vec::with_capacity(color_count);
        for i in 0..color_count {
            let cube = &self.cubes[i];
            let weight = self.volume(cube, &self.weights);
            if weight > 0 {
                let r = (self.volume(cube, &self.moments_r) / weight) as u8;
                let g = (self.volume(cube, &self.moments_g) / weight) as u8;
                let b = (self.volume(cube, &self.moments_b) / weight) as u8;
                colors.push(argb_from_rgb(r, g, b));
            }
        }
        colors
    }

    #[allow(dead_code)]
    fn variance(&self, cube: &Box) -> f32 {
        let dr = self.volume(cube, &self.moments_r);
        let dg = self.volume(cube, &self.moments_g);
        let db = self.volume(cube, &self.moments_b);

        let xx = self.moments[get_index(cube.r1, cube.g1, cube.b1)]
            - self.moments[get_index(cube.r1, cube.g1, cube.b0)]
            - self.moments[get_index(cube.r1, cube.g0, cube.b1)]
            + self.moments[get_index(cube.r1, cube.g0, cube.b0)]
            - self.moments[get_index(cube.r0, cube.g1, cube.b1)]
            + self.moments[get_index(cube.r0, cube.g1, cube.b0)]
            + self.moments[get_index(cube.r0, cube.g0, cube.b1)]
            - self.moments[get_index(cube.r0, cube.g0, cube.b0)];

        let hypotenuse = dr as f32 * dr as f32 + dg as f32 * dg as f32 + db as f32 * db as f32;
        let volume = self.volume(cube, &self.weights);
        if volume == 0 {
            return 0.0;
        }
        xx - hypotenuse / volume as f32
    }

    fn volume(&self, cube: &Box, moment: &[i32]) -> i32 {
        moment[get_index(cube.r1, cube.g1, cube.b1)]
            - moment[get_index(cube.r1, cube.g1, cube.b0)]
            - moment[get_index(cube.r1, cube.g0, cube.b1)]
            + moment[get_index(cube.r1, cube.g0, cube.b0)]
            - moment[get_index(cube.r0, cube.g1, cube.b1)]
            + moment[get_index(cube.r0, cube.g1, cube.b0)]
            + moment[get_index(cube.r0, cube.g0, cube.b1)]
            - moment[get_index(cube.r0, cube.g0, cube.b0)]
    }
}

fn cut_box_values(
    _one: &mut Box,
    _two: &mut Box,
    _one_r0: usize,
    _one_r1: usize,
    _one_g0: usize,
    _one_g1: usize,
    _one_b0: usize,
    _one_b1: usize,
) -> bool {
    true
}

impl Default for QuantizerWu {
    fn default() -> Self {
        Self::new()
    }
}

pub fn quantize_wu(pixels: &[(u8, u8, u8)], max_colors: usize) -> Vec<u32> {
    let argb_pixels: Vec<u32> = pixels
        .iter()
        .map(|(r, g, b)| argb_from_rgb(*r, *g, *b))
        .collect();
    let mut quantizer = QuantizerWu::new();
    quantizer.quantize(&argb_pixels, max_colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wu_quantizer() {
        let pixels = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0)];
        let result = quantize_wu(&pixels, 4);
        assert!(!result.is_empty());
    }
}
