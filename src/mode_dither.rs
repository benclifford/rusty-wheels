use crate::leds;
use crate::structs::{FrameState, Mode, RGB24};
use std::io;

struct Dither {
    /// This will contain the errors propagated from the previous frame
    prev_errors: [f32; 23],
    /// This will contain the errors propagated to the next frame
    next_errors: [f32; 23],
    /// pre-step will render into here
    rgb: [RGB24; 23],
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
        let scaled_pos = (bounded_pos * 2.0) % 1.0;
        let intensity = if scaled_pos < 0.5 {
            scaled_pos
        } else {
            1.0 - scaled_pos
        };

        let mut row_accum_error = 0.0;
        self.next_errors = [0.0; 23];

        for led in 0..23 {
            let corrected_intensity = intensity + row_accum_error + self.prev_errors[led];

            let render_amount = if corrected_intensity > 0.66 {
                1.0
            } else if corrected_intensity > 0.33 {
                0.5
            } else {
                0.0
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
            let gamma_corrected_render_amount = render_amount.clamp(0.0, 1.0).powf(GAMMA);
            let colour = (
                (255.0 * gamma_corrected_render_amount) as u8,
                (255.0 * gamma_corrected_render_amount) as u8,
                0,
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
    Box::new(Dither {
        prev_errors: [0.0; 23],
        next_errors: [0.0; 23],
        rgb: [(0, 0, 0); 23],
    })
}
