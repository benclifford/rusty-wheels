use crate::leds;
use crate::structs::{FrameState, Mode};
use std::io;

struct Dither {
    /// This will contain the errors propagated to the next frame
    errors: [f32; 23],
}

impl Mode for Dither {
    fn render(
        &self,
        side: usize,
        leds: &mut leds::WheelLEDs,
        frame: &FrameState,
    ) -> io::Result<()> {
        // fade from 0 up to full intensity around the wheel

        let intensity = frame.spin_pos;

        let mut accum_error = 0.0;

        for led in 0..23 {
            let corrected_intensity = intensity + accum_error;

            let render_amount = if corrected_intensity > 0.5 { 1.0 } else { 0.0 };

            accum_error = corrected_intensity - render_amount;

            // println!("Accumulated error {}", accum_error);

            let colour = (
                (255.0 * render_amount) as u8,
                (64.0 * render_amount) as u8,
                0,
            );

            leds.set(side, led, colour);
        }

        Ok(())
    }

    //    fn step(&mut self, frame: &FrameState) -> io::Result<()> {
    //        self.last_spin_pos = frame.spin_pos;
    //        Ok(())
    //    }
}

pub fn create_dither() -> Box<dyn Mode> {
    Box::new(Dither { errors: [0.0; 23] })
}
