use crate::leds;
use crate::structs::{FrameState, Mode};
use std::io;

struct Dither {
    /// This will contain the errors propagated from the previous frame
    prev_errors: [f32; 23],
    /// This will contain the errors propagated to the next frame
    next_errors: [f32; 23],
    /// pre-step will render into here
    rgb: [(u8, u8, u8); 23],
}

impl Mode for Dither {
    fn render(
        &self,
        side: usize,
        leds: &mut leds::WheelLEDs,
        frame: &FrameState,
    ) -> io::Result<()> {

    for led in 0..23 {
        leds.set(side, led, self.rgb[led]);
    }
    Ok(())
}

    fn pre_step(&mut self, frame: &FrameState) -> io::Result<()> {
        // fade from 0 up to full intensity around the wheel
        let intensity = frame.spin_pos.min(1.0);
        // let intensity = 0.3;

        let mut row_accum_error = 0.0;
        self.next_errors = [0.0; 23];

        for led in 0..23 {
            let corrected_intensity = intensity + row_accum_error + self.prev_errors[led];

            let render_amount = if corrected_intensity > 0.5 { 1.0 } else { 0.0 };

            let total_error = corrected_intensity - render_amount;

            row_accum_error = total_error * (7.0 / 16.0);

            /*
            println!("intensity = {}", intensity);
            println!("corrected intensity = {}", corrected_intensity);
            println!("render_amount = {}", render_amount);
            println!("total error (corr. int - render_amt)= {}", total_error);
            println!("row_accum_error = {}", row_accum_error);
            */

            let lower_accum_error = total_error * (3.0 / 16.0);
            let mid_accum_error = total_error * (5.0 / 16.0);
            let higher_accum_error = total_error * (1.0 / 16.0);

            // is it right to discard the lower and higher errors at the end,
            // or should they be absorbed into other errors?
            if led > 0 {
                self.next_errors[led-1] += lower_accum_error;
            }
            self.next_errors[led] += mid_accum_error;
            if led < 22 {
                self.next_errors[led+1] += higher_accum_error;
            }

            let colour = (
                (255.0 * render_amount) as u8,
                (64.0 * render_amount) as u8,
                0,
            );
            self.rgb[led] = colour;
        }

        Ok(())
    }

    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
        self.prev_errors = self.next_errors;
        Ok(())
    }
}

pub fn create_dither() -> Box<dyn Mode> {
    Box::new(Dither {
        prev_errors: [0.0; 23],
        next_errors: [0.0; 23],
        rgb: [(0, 0, 0); 23]
    })
}
