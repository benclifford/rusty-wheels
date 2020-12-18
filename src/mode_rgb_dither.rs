use crate::leds;
use crate::structs::{FrameState, Mode};
use std::io;

use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Sub;

use palette::encoding::pixel::Pixel;
use palette::Hsv;
use palette::Srgb;

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
    fn add_assign(&mut self, other: Self) -> () {
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
    rgb: [(u8, u8, u8); 23],
}

impl Mode for Dither {
    fn render(
        &self,
        side: usize,
        leds: &mut leds::WheelLEDs,
        _frame: &FrameState,
    ) -> io::Result<()> {
        for led in 0..23 {
            leds.set(side, led, self.rgb[led]);
        }
        Ok(())
    }

    fn pre_step(&mut self, frame: &FrameState) -> io::Result<()> {
        let bounded_pos = frame.spin_pos.min(1.0);

        let hue = (bounded_pos * 360.0).min(360.0);
        // don't push value too high - if can't render full intensity, colour choosing alg locks on red
        let hsv: Hsv = Hsv::from_components((hue, 1.0, 0.3));

        let srgb = Srgb::from(hsv);

        let pixels: [u8; 3] = srgb.into_linear().into_format().into_raw();

        let r = (pixels[0] as f32) / 256.0;
        let g = (pixels[1] as f32) / 256.0;
        let b = (pixels[2] as f32) / 256.0;

        let intensity: V = V { v: (r, g, b) };

        let mut row_accum_error: V = V { v: (0.0, 0.0, 0.0) };
        self.next_errors = [V { v: (0.0, 0.0, 0.0) }; 23];

        for led in 0..23 {
            let corrected_intensity = intensity + row_accum_error + self.prev_errors[led];

            let (r, g, b) = corrected_intensity.v;
            let render_amount = if r > 0.5 {
                V { v: (1.0, 0.0, 0.0) }
            } else if g > 0.5 {
                V { v: (0.0, 1.0, 0.0) }
            } else if b > 0.5 {
                V { v: (0.0, 0.0, 1.0) }
            } else {
                V { v: (0.0, 0.0, 0.0) }
            };

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
            // TODO: put GAMMA correction back in?
            let colour = ((255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8);
            self.rgb[led] = colour;
        }

        Ok(())
    }

    fn step(&mut self, _frame: &FrameState) -> io::Result<()> {
        self.prev_errors = self.next_errors;
        Ok(())
    }
}

pub fn create_dither() -> Box<dyn Mode> {
    Box::new(Dither {
        prev_errors: [V { v: (0.0, 0.0, 0.0) }; 23],
        next_errors: [V { v: (0.0, 0.0, 0.0) }; 23],
        rgb: [(0, 0, 0); 23],
    })
}
