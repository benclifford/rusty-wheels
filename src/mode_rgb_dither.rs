use crate::leds;
use crate::structs::{FrameState, Mode, RGB24};
use std::io;

use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Sub;

use palette::encoding::pixel::Pixel;
use palette::Hsv;
use palette::Srgb;

use rand::Rng;

/// a value in the space we are dithering
#[derive(Copy, Clone)]
struct V {
    v: (f32, f32, f32),
}

impl Add for V {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let (x, y, z) = other.v;
        let (a, b, c) = self.v;
        V {
            v: (a + x, b + y, c + z),
        }
    }
}

impl AddAssign for V {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for V {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let (x, y, z) = other.v;
        let (a, b, c) = self.v;
        V {
            v: (a - x, b - y, c - z),
        }
    }
}

impl Mul<f32> for V {
    type Output = Self;
    fn mul(self, other: f32) -> Self {
        let (a, b, c) = self.v;
        V {
            v: (a * other, b * other, c * other),
        }
    }
}

struct Dither {
    /// This will contain the errors propagated from the previous frame
    prev_errors: [V; 23],
    /// This will contain the errors propagated to the next frame
    next_errors: [V; 23],
    /// pre-step will render into here
    rgb: [RGB24; 23],
    /// selection of pixel colours that can be used
    available_colours: Vec<(f32, f32, f32)>,
}

impl<const LEDS: usize> Mode<LEDS> for Dither {
    fn render(
        &self,
        side: leds::Side,
        leds: &mut leds::WheelLEDs<LEDS>,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            leds.set(side, led, self.rgb[led]);
        }
        Ok(())
    }

    fn pre_step(&mut self, frame: &FrameState) -> io::Result<()> {
        let bounded_pos = frame.spin_pos.min(1.0);

        let mut row_accum_error: V = V { v: (0.0, 0.0, 0.0) };
        self.next_errors = [V { v: (0.0, 0.0, 0.0) }; 23];

        for led in 0..23 {
            let hue = (bounded_pos * 360.0).min(360.0);

            let value = ((led as f32) / 23.0).powf(2.0) * 0.9 + 0.1;
            // don't push value too high - if can't render full intensity, colour choosing alg locks on red
            let hsv: Hsv = Hsv::from_components((hue, 1.0, value));

            let srgb = Srgb::from(hsv);

            let pixels: [f32; 3] = srgb.into_linear().into_format().into_raw();

            let r = pixels[0];
            let g = pixels[1];
            let b = pixels[2];

            let intensity: V = V { v: (r, g, b) };

            let corrected_intensity = intensity + row_accum_error + self.prev_errors[led];

            let render_amount = find_closest_colour(corrected_intensity.v, &self.available_colours);

            let total_error = corrected_intensity - render_amount;

            row_accum_error = total_error * (7.0 / 16.0);

            let lower_accum_error = total_error * (3.0 / 16.0);
            let mid_accum_error = total_error * (5.0 / 16.0);
            let higher_accum_error = total_error * (1.0 / 16.0);

            // is it right to discard the lower and higher errors at the end,
            // or should they be absorbed into other errors?
            if led > 0 {
                self.next_errors[led - 1] += lower_accum_error;
            }
            self.next_errors[led] += mid_accum_error;
            if led < 22 {
                self.next_errors[led + 1] += higher_accum_error;
            }

            const GAMMA: f32 = 2.0;
            let v = render_amount;
            let (r, g, b) = v.v;
            let colour = (
                (255.0 * (r.powf(GAMMA))) as u8,
                (255.0 * (g.powf(GAMMA))) as u8,
                (255.0 * (b.powf(GAMMA))) as u8,
            );
            self.rgb[led] = colour;
        }

        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        self.prev_errors = self.next_errors;
        Ok(())
    }
}

pub fn create_dither<const LEDS: usize>() -> Box<dyn Mode<LEDS>> {
    let num_colours = rand::thread_rng().gen_range(2, 6);

    let base_degrees = rand::thread_rng().gen_range(0.0, 360.0);
    let step_degrees = 360.0 / (num_colours as f32);

    let mut colour_vec: Vec<(f32, f32, f32)> = Vec::new();
    colour_vec.push((0.0, 0.0, 0.0));

    for n in 0..num_colours {
        let hue = (base_degrees + step_degrees * (n as f32)) % 360.0;
        let hsv: Hsv = Hsv::from_components((hue, 1.0, 1.0));

        let srgb = Srgb::from(hsv);

        let pixels: [f32; 3] = srgb.into_linear().into_format().into_raw();

        let r = pixels[0];
        let g = pixels[1];
        let b = pixels[2];

        colour_vec.push((r, g, b));
    }

    Box::new(Dither {
        prev_errors: [V { v: (0.0, 0.0, 0.0) }; 23],
        next_errors: [V { v: (0.0, 0.0, 0.0) }; 23],
        rgb: [(0, 0, 0); 23],
        available_colours: colour_vec,
    })
}

fn find_closest_colour(rgb: (f32, f32, f32), available: &[(f32, f32, f32)]) -> V {
    let (r, g, b) = rgb;
    let mut best_distance = 1000.0; // "very big"
    let mut best_colour = (0.0, 0.0, 0.0);
    for (r2, g2, b2) in available.iter() {
        let distance = ((r - r2).powf(2.0) + (g - g2).powf(2.0) + (b - b2).powf(2.0)).sqrt();
        if distance < best_distance {
            best_colour = (*r2, *g2, *b2);
            best_distance = distance;
        }
    }
    V { v: best_colour }
}
